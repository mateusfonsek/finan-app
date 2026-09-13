//! What the MCP screen reads and writes.

use tauri::{AppHandle, State};

use crate::db::Db;
use crate::error::AppResult;
use crate::mcp::config::McpConfig;
use crate::mcp::log::CallEntry;
use crate::mcp::{self, McpState};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct McpToolState {
    pub name: String,
    /// English, from the registry — it is protocol, not interface.
    pub description: String,
    /// Writes are grouped apart on screen and start off.
    pub write: bool,
    pub enabled: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct McpStatus {
    pub enabled: bool,
    /// `None` while the server is down — a URL for a closed port is a lie.
    pub port: Option<u16>,
    pub url: Option<String>,
    pub window_months: u32,
    pub tools: Vec<McpToolState>,
}

fn status_of(db: &Db, state: &McpState) -> AppResult<McpStatus> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    let cfg = McpConfig::load(&conn)?;
    let port = state.port();

    let tools = mcp::tools::specs()
        .iter()
        .map(|s| McpToolState {
            name: s.name.to_string(),
            description: s.description.to_string(),
            write: s.write,
            enabled: cfg.tool_enabled(s.name),
        })
        .collect();

    Ok(McpStatus {
        enabled: cfg.enabled,
        port,
        url: port.map(|p| format!("http://127.0.0.1:{p}/mcp")),
        window_months: cfg.window_months,
        tools,
    })
}

#[tauri::command]
#[specta::specta]
pub fn mcp_status(db: State<'_, Db>, state: State<'_, McpState>) -> AppResult<McpStatus> {
    status_of(&db, &state)
}

#[tauri::command]
#[specta::specta]
pub fn set_mcp_enabled(
    app: AppHandle,
    db: State<'_, Db>,
    state: State<'_, McpState>,
    enabled: bool,
) -> AppResult<McpStatus> {
    {
        let conn = db.conn.lock().expect("db mutex poisoned");
        mcp::config::set_enabled(&conn, enabled)?;
    }
    if enabled {
        mcp::start(&app)?;
    } else {
        mcp::stop(&app);
    }
    status_of(&db, &state)
}

#[tauri::command]
#[specta::specta]
pub fn set_mcp_tool(
    db: State<'_, Db>,
    state: State<'_, McpState>,
    name: String,
    enabled: bool,
) -> AppResult<McpStatus> {
    {
        let conn = db.conn.lock().expect("db mutex poisoned");
        mcp::config::set_tool(&conn, &name, enabled)?;
    }
    status_of(&db, &state)
}

#[tauri::command]
#[specta::specta]
pub fn set_mcp_window(
    db: State<'_, Db>,
    state: State<'_, McpState>,
    months: u32,
) -> AppResult<McpStatus> {
    {
        let conn = db.conn.lock().expect("db mutex poisoned");
        mcp::config::set_window(&conn, months)?;
    }
    status_of(&db, &state)
}

#[tauri::command]
#[specta::specta]
pub fn mcp_recent_calls(state: State<'_, McpState>) -> AppResult<Vec<CallEntry>> {
    Ok(state.log.entries())
}
