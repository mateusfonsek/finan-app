use rusqlite::params;
use tauri::State;

use crate::db::Db;
use crate::domain::category::{Category, NewCategory};
use crate::error::{AppError, AppResult};

#[tauri::command]
#[specta::specta]
pub fn list_categories(db: State<'_, Db>) -> AppResult<Vec<Category>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    let mut stmt = conn.prepare(
        "SELECT id, name, color_token, kind, created_at FROM categories ORDER BY kind, name",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Category {
            id: row.get(0)?,
            name: row.get(1)?,
            color_token: row.get(2)?,
            kind: row.get(3)?,
            created_at: row.get(4)?,
        })
    })?;
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(AppError::from)
}

#[tauri::command]
#[specta::specta]
pub fn create_category(db: State<'_, Db>, input: NewCategory) -> AppResult<Category> {
    if !matches!(input.kind.as_str(), "expense" | "income" | "transfer") {
        return Err(AppError::Invalid(format!(
            "invalid kind '{}' (must be expense|income|transfer)",
            input.kind
        )));
    }
    let conn = db.conn.lock().expect("db mutex poisoned");
    conn.execute(
        "INSERT INTO categories (name, color_token, kind) VALUES (?1, ?2, ?3)",
        params![input.name, input.color_token, input.kind],
    )?;
    let id = conn.last_insert_rowid();
    conn.query_row(
        "SELECT id, name, color_token, kind, created_at FROM categories WHERE id = ?1",
        params![id],
        |row| {
            Ok(Category {
                id: row.get(0)?,
                name: row.get(1)?,
                color_token: row.get(2)?,
                kind: row.get(3)?,
                created_at: row.get(4)?,
            })
        },
    )
    .map_err(AppError::from)
}

#[cfg(test)]
mod tests {
    use crate::db::migrations;
    use rusqlite::{params, Connection};

    fn fresh_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        migrations::apply(&conn).unwrap();
        conn
    }

    #[test]
    fn seed_categories_listed() {
        let conn = fresh_conn();
        let mut stmt = conn
            .prepare("SELECT name, kind FROM categories ORDER BY name")
            .unwrap();
        let rows: Vec<(String, String)> = stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        assert_eq!(rows.len(), 9);
        assert!(rows.iter().any(|(n, _)| n == "Mercado"));
        assert!(rows.iter().any(|(n, k)| n == "Renda" && k == "income"));
    }

    #[test]
    fn create_category_inserts_row() {
        let conn = fresh_conn();
        conn.execute(
            "INSERT INTO categories (name, color_token, kind) VALUES (?1, ?2, ?3)",
            params!["Pets", "--color-cat-outros", "expense"],
        )
        .unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM categories", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 10);
    }

    #[test]
    fn create_category_rejects_duplicate_name() {
        let conn = fresh_conn();
        let r = conn.execute(
            "INSERT INTO categories (name, color_token, kind) VALUES (?1, ?2, ?3)",
            params!["Mercado", "--color-cat-mercado", "expense"],
        );
        assert!(r.is_err(), "UNIQUE constraint on name should reject");
    }
}
