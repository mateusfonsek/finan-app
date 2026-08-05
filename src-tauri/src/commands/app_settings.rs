//! Simple key/value app preferences. Born for automatic import's
//! `watch_enabled`, but deliberately generic — future preferences land here
//! instead of becoming a new column somewhere.

use rusqlite::{params, Connection, OptionalExtension};
use tauri::State;

use crate::db::Db;
use crate::error::{AppError, AppResult};

pub fn get_setting(conn: &Connection, key: &str) -> AppResult<Option<String>> {
    conn.query_row(
        "SELECT value FROM app_settings WHERE key = ?1",
        params![key],
        |row| row.get::<_, String>(0),
    )
    .optional()
    .map_err(AppError::from)
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> AppResult<()> {
    conn.execute(
        "INSERT INTO app_settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn get_app_setting(db: State<'_, Db>, key: String) -> AppResult<Option<String>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    get_setting(&conn, &key)
}

#[tauri::command]
#[specta::specta]
pub fn set_app_setting(db: State<'_, Db>, key: String, value: String) -> AppResult<()> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    set_setting(&conn, &key, &value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;

    fn fresh_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        migrations::apply(&conn).unwrap();
        conn
    }

    #[test]
    fn missing_key_returns_none() {
        let conn = fresh_conn();
        assert_eq!(get_setting(&conn, "watch_enabled").unwrap(), None);
    }

    #[test]
    fn set_then_get_roundtrips() {
        let conn = fresh_conn();
        set_setting(&conn, "watch_enabled", "1").unwrap();
        assert_eq!(
            get_setting(&conn, "watch_enabled").unwrap(),
            Some("1".to_string())
        );
    }

    #[test]
    fn set_overwrites_existing_value() {
        let conn = fresh_conn();
        set_setting(&conn, "watch_enabled", "1").unwrap();
        set_setting(&conn, "watch_enabled", "0").unwrap();
        assert_eq!(
            get_setting(&conn, "watch_enabled").unwrap(),
            Some("0".to_string())
        );
        let rows: i64 = conn
            .query_row("SELECT COUNT(*) FROM app_settings", [], |r| r.get(0))
            .unwrap();
        assert_eq!(rows, 1, "upsert must not duplicate the key");
    }
}
