use std::path::PathBuf;
use std::sync::Mutex;

use rusqlite::{params, Connection};
use tauri::{AppHandle, Manager};

use crate::error::{AppError, AppResult};
use crate::locale::LocalePack;

pub mod migrations;

pub struct Db {
    pub conn: Mutex<Connection>,
    pub path: PathBuf,
    /// `true` when this process created a brand-new database. The caller seeds
    /// a fresh DB from the active locale pack; existing DBs are left untouched.
    pub fresh: bool,
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

    let fresh = migrations::apply(&conn)?;

    Ok(Db {
        conn: Mutex::new(conn),
        path,
        fresh,
    })
}

/// Seed a **fresh** database from the active locale pack. Categories are matched
/// by their stable `key`: an existing seeded row (created by the SQL migrations)
/// is renamed to the pack's `name`; a key the migrations didn't seed is inserted.
/// Seed rules are inserted mapped by category key, skipping patterns already
/// present. Safe to call only on a fresh DB (never overwrites user edits).
pub fn seed_from_pack(conn: &Connection, pack: &LocalePack) -> AppResult<()> {
    for c in &pack.categories {
        let updated = conn.execute(
            "UPDATE categories
                SET name = ?2, color_token = ?3, kind = ?4, is_investment = ?5
              WHERE key = ?1",
            params![c.key, c.name, c.color_token, c.kind, c.is_investment],
        )?;
        if updated == 0 {
            conn.execute(
                "INSERT INTO categories (key, name, color_token, kind, is_investment)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![c.key, c.name, c.color_token, c.kind, c.is_investment],
            )?;
        }
    }

    for r in &pack.rules.seed_rules {
        // A seeded rule starts with a single snippet; more are added from the
        // Rules screen.
        let inserted = conn.execute(
            "INSERT INTO rules (category_id, priority, due_day, display_name)
             SELECT c.id, ?1, NULL, ?2 FROM categories c
             WHERE c.key = ?3
               AND NOT EXISTS (
                   SELECT 1 FROM rule_patterns WHERE pattern = ?4
               )",
            params![r.priority, r.display_name, r.category, r.pattern],
        )?;
        if inserted > 0 {
            conn.execute(
                "INSERT INTO rule_patterns (rule_id, pattern) VALUES (?1, ?2)",
                params![conn.last_insert_rowid(), r.pattern],
            )?;
        }
    }

    Ok(())
}
