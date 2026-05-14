use rusqlite::params;
use tauri::State;

use crate::db::Db;
use crate::domain::category::{
    is_default_category, Category, CategoryWithCount, NewCategory, UpdateCategory,
};
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
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(AppError::from)
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

#[tauri::command]
#[specta::specta]
pub fn list_categories_with_count(db: State<'_, Db>) -> AppResult<Vec<CategoryWithCount>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    let mut stmt = conn.prepare(
        "SELECT c.id, c.name, c.color_token, c.kind, c.created_at,
                COUNT(t.id) AS tx_count
         FROM categories c
         LEFT JOIN transactions t ON t.category_id = c.id
         GROUP BY c.id
         ORDER BY c.kind, c.name",
    )?;
    let rows = stmt.query_map([], |row| {
        let name: String = row.get(1)?;
        let is_default = is_default_category(&name);
        Ok(CategoryWithCount {
            id: row.get(0)?,
            name,
            color_token: row.get(2)?,
            kind: row.get(3)?,
            created_at: row.get(4)?,
            transaction_count: row.get::<_, i64>(5)? as u32,
            is_default,
        })
    })?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(AppError::from)
}

#[tauri::command]
#[specta::specta]
pub fn update_category(
    db: State<'_, Db>,
    category_id: i64,
    input: UpdateCategory,
) -> AppResult<Category> {
    if input.name.trim().is_empty() {
        return Err(AppError::Invalid("name must not be empty".into()));
    }
    if !matches!(input.kind.as_str(), "expense" | "income" | "transfer") {
        return Err(AppError::Invalid(format!("invalid kind '{}'", input.kind)));
    }
    let conn = db.conn.lock().expect("db mutex poisoned");
    let changed = conn.execute(
        "UPDATE categories SET name = ?1, color_token = ?2, kind = ?3 WHERE id = ?4",
        params![input.name.trim(), input.color_token, input.kind, category_id],
    )?;
    if changed == 0 {
        return Err(AppError::Invalid(format!(
            "category {category_id} not found"
        )));
    }
    conn.query_row(
        "SELECT id, name, color_token, kind, created_at FROM categories WHERE id = ?1",
        params![category_id],
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

/// Apaga uma categoria custom. Defaults (seeds) são protegidas.
/// Transações que referenciavam a categoria ficam com category_id=NULL.
#[tauri::command]
#[specta::specta]
pub fn delete_category(db: State<'_, Db>, category_id: i64) -> AppResult<()> {
    let mut conn = db.conn.lock().expect("db mutex poisoned");

    let name: String = conn.query_row(
        "SELECT name FROM categories WHERE id = ?1",
        params![category_id],
        |r| r.get(0),
    )?;

    if is_default_category(&name) {
        return Err(AppError::Invalid(format!(
            "categoria padrão '{name}' não pode ser apagada"
        )));
    }

    let tx = conn.transaction()?;
    tx.execute(
        "UPDATE transactions SET category_id = NULL WHERE category_id = ?1",
        params![category_id],
    )?;
    let deleted = tx.execute(
        "DELETE FROM categories WHERE id = ?1",
        params![category_id],
    )?;
    if deleted == 0 {
        return Err(AppError::Invalid(format!(
            "category {category_id} not found"
        )));
    }
    tx.commit()?;
    Ok(())
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

    #[test]
    fn default_category_names_recognized() {
        use crate::domain::category::is_default_category;
        assert!(is_default_category("Mercado"));
        assert!(is_default_category("Renda"));
        assert!(!is_default_category("Pets"));
        assert!(!is_default_category(""));
    }

    #[test]
    fn delete_category_cascades_to_null_in_transactions() {
        let conn = fresh_conn();
        // create a custom category to delete
        conn.execute(
            "INSERT INTO categories (name, color_token, kind) VALUES ('Pets', '--color-cat-outros', 'expense')",
            [],
        )
        .unwrap();
        let cat_id: i64 = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO accounts (name, bank, ofx_acctid) VALUES ('t', NULL, 'A')",
            [],
        )
        .unwrap();
        let acc: i64 = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO transactions (account_id, date, amount, description, category_id, ofx_fitid)
             VALUES (?1, '2026-04-01', '-10', 'd', ?2, NULL)",
            params![acc, cat_id],
        )
        .unwrap();

        // Simulate the command's SQL
        conn.execute(
            "UPDATE transactions SET category_id = NULL WHERE category_id = ?1",
            params![cat_id],
        )
        .unwrap();
        conn.execute("DELETE FROM categories WHERE id = ?1", params![cat_id])
            .unwrap();

        let null_cat_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM transactions WHERE category_id IS NULL",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(null_cat_count, 1);
    }
}
