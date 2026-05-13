use std::path::PathBuf;
use std::sync::Mutex;

use rusqlite::Connection;
use tauri::{AppHandle, Manager};

use crate::error::{AppError, AppResult};

pub mod migrations;

pub struct Db {
    pub conn: Mutex<Connection>,
    pub path: PathBuf,
}

pub fn init(app: &AppHandle) -> AppResult<Db> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Path(e.to_string()))?;

    std::fs::create_dir_all(&dir)?;
    let path = dir.join("finan.db");

    let conn = Connection::open(&path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;

    migrations::apply(&conn)?;

    Ok(Db {
        conn: Mutex::new(conn),
        path,
    })
}
