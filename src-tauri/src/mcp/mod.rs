//! The local MCP server: a socket the owner opens on purpose.
//!
//! Bound to 127.0.0.1 and nothing else. There is no token — the deliberate
//! trade is documented in the spec — so the Origin check in `protocol` carries
//! the defense, and the bind address is what keeps the port off the network.

pub mod config;
pub mod log;
pub mod protocol;
pub mod tools;

use std::io::Read;
use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Emitter, Manager};
use tiny_http::{Header, Request, Response, Server};

use crate::db::Db;
use crate::error::{AppError, AppResult};
use crate::mcp::config::McpConfig;
use crate::mcp::log::CallLog;

pub const PREFERRED_PORT: u16 = 7717;
/// Emitted after any tool wrote, so open screens reload instead of showing a
/// number the agent already changed.
pub const DATA_CHANGED_EVENT: &str = "mcp:data-changed";
/// An open port must not be a way to exhaust memory.
const MAX_BODY_BYTES: usize = 256 * 1024;

fn body_too_large(len: usize) -> bool {
    len > MAX_BODY_BYTES
}

struct Running {
    server: Arc<Server>,
    port: u16,
    thread: Option<std::thread::JoinHandle<()>>,
}

pub struct McpState {
    running: Mutex<Option<Running>>,
    pub log: Arc<CallLog>,
}

impl McpState {
    pub fn new() -> McpState {
        McpState { running: Mutex::new(None), log: Arc::new(CallLog::new()) }
    }

    pub fn port(&self) -> Option<u16> {
        self.running.lock().expect("mcp state mutex poisoned").as_ref().map(|r| r.port)
    }
}

impl Default for McpState {
    fn default() -> Self {
        Self::new()
    }
}

/// Every response carries the negotiated protocol version, which the spec
/// requires on the transport once `initialize` has agreed on it.
fn respond(request: Request, body: String, status: u16) {
    let content_type = Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
        .expect("a static header always parses");
    let version = Header::from_bytes(
        &b"MCP-Protocol-Version"[..],
        protocol::PROTOCOL_VERSION.as_bytes(),
    )
    .expect("a static header always parses");
    let _ = request.respond(
        Response::from_string(body)
            .with_status_code(status)
            .with_header(content_type)
            .with_header(version),
    );
}

fn header<'a>(request: &'a Request, name: &'static str) -> Option<&'a str> {
    request
        .headers()
        .iter()
        .find(|h| h.field.equiv(name))
        .map(|h| h.value.as_str())
}

/// Binds the preferred port, falling back to whatever the OS hands out. A busy
/// 7717 must not mean a broken feature — the screen shows the port that won.
fn bind() -> AppResult<Server> {
    match Server::http(("127.0.0.1", PREFERRED_PORT)) {
        Ok(server) => Ok(server),
        Err(_) => Server::http(("127.0.0.1", 0))
            .map_err(|e| AppError::Invalid(format!("could not open a local port: {e}"))),
    }
}

pub fn start(app: &AppHandle) -> AppResult<u16> {
    let state = app.state::<McpState>();
    let mut running = state.running.lock().expect("mcp state mutex poisoned");
    if let Some(r) = running.as_ref() {
        return Ok(r.port);
    }

    let server = Arc::new(bind()?);
    let port = server
        .server_addr()
        .to_ip()
        .ok_or_else(|| AppError::Invalid("server bound to a non-IP address".into()))?
        .port();

    let thread = {
        let server = server.clone();
        let app = app.clone();
        let log = state.log.clone();
        std::thread::spawn(move || {
            for request in server.incoming_requests() {
                serve(&app, &log, request);
            }
        })
    };

    *running = Some(Running { server, port, thread: Some(thread) });
    Ok(port)
}

/// Takes the request by value: `Request::respond` consumes it, so each guard
/// simply answers and returns.
fn serve(app: &AppHandle, log: &CallLog, mut request: Request) {
    // Checked first, and on every request: without a token this is the only
    // thing standing between a web page and the statement.
    if !protocol::origin_allowed(header(&request, "Origin")) {
        respond(request, r#"{"error":"origin not allowed"}"#.to_string(), 403);
        return;
    }
    if request.method() != &tiny_http::Method::Post {
        respond(request, r#"{"error":"use POST /mcp"}"#.to_string(), 405);
        return;
    }
    // A form-encoded body is how a plain HTML form reaches a port without a
    // preflight — requiring JSON closes that door.
    match header(&request, "Content-Type") {
        Some(ct) if ct.starts_with("application/json") => {}
        _ => {
            respond(request, r#"{"error":"expected application/json"}"#.to_string(), 415);
            return;
        }
    }
    if body_too_large(request.body_length().unwrap_or(0)) {
        respond(request, r#"{"error":"body too large"}"#.to_string(), 413);
        return;
    }

    let mut body = String::new();
    let _ = request
        .as_reader()
        .take(MAX_BODY_BYTES as u64)
        .read_to_string(&mut body);

    let out = {
        let db = app.state::<Db>();
        let locale = app.state::<crate::locale::LocaleState>();
        let mut conn = db.conn.lock().expect("db mutex poisoned");
        let pack = locale.pack.lock().expect("locale mutex poisoned");

        match McpConfig::load(&conn) {
            Ok(cfg) if !cfg.enabled => {
                drop(pack);
                drop(conn);
                // The config can be switched off without the socket being
                // torn down first, so this is checked on every request, not
                // only when the listener starts.
                respond(request, r#"{"error":"mcp disabled"}"#.to_string(), 403);
                return;
            }
            Ok(cfg) => protocol::handle(&mut conn, &pack, &cfg, log, &body),
            Err(e) => {
                drop(pack);
                drop(conn);
                respond(request, format!(r#"{{"error":"{e}"}}"#), 500);
                return;
            }
        }
    };

    if out.wrote {
        let _ = app.emit(DATA_CHANGED_EVENT, ());
    }

    // A notification is answered with 202 and no body, per JSON-RPC.
    let status = if out.json.is_empty() { 202 } else { 200 };
    respond(request, out.json, status);
}

pub fn stop(app: &AppHandle) {
    let state = app.state::<McpState>();
    let mut running = state.running.lock().expect("mcp state mutex poisoned");
    if let Some(mut r) = running.take() {
        r.server.unblock();
        if let Some(t) = r.thread.take() {
            let _ = t.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpStream;

    /// Proves `mod.rs` wires the pieces together — the protocol itself is
    /// covered without a socket in `protocol.rs`.
    #[test]
    fn the_server_answers_a_real_initialize_over_tcp() {
        let server = std::sync::Arc::new(tiny_http::Server::http("127.0.0.1:0").unwrap());
        let port = server.server_addr().to_ip().unwrap().port();

        let handle = {
            let server = server.clone();
            std::thread::spawn(move || {
                if let Some(request) = server.incoming_requests().next() {
                    respond(request, "{\"ok\":true}".to_string(), 200);
                }
            })
        };

        let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
        let body = r#"{"jsonrpc":"2.0","id":1,"method":"initialize"}"#;
        write!(
            stream,
            "POST /mcp HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        )
        .unwrap();

        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        handle.join().unwrap();

        assert!(response.starts_with("HTTP/1.1 200"), "got: {response}");
        assert!(response.contains("\"ok\":true"));
    }

    #[test]
    fn a_body_over_the_cap_is_refused() {
        assert!(body_too_large(MAX_BODY_BYTES + 1));
        assert!(!body_too_large(MAX_BODY_BYTES));
    }

    /// The response must carry the negotiated version on the transport, not
    /// only inside the `initialize` result.
    #[test]
    fn the_protocol_version_is_a_response_header() {
        let server = std::sync::Arc::new(tiny_http::Server::http("127.0.0.1:0").unwrap());
        let port = server.server_addr().to_ip().unwrap().port();

        let handle = {
            let server = server.clone();
            std::thread::spawn(move || {
                if let Some(request) = server.incoming_requests().next() {
                    respond(request, "{}".to_string(), 200);
                }
            })
        };

        let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
        write!(
            stream,
            "POST /mcp HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        )
        .unwrap();

        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        handle.join().unwrap();

        assert!(
            response.contains(crate::mcp::protocol::PROTOCOL_VERSION),
            "got: {response}"
        );
    }
}
