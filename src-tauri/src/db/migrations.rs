use rusqlite::Connection;

use crate::error::AppResult;

const MIGRATIONS: &[(&str, &str)] = &[
    ("0001_init", include_str!("../../migrations/0001_init.sql")),
    (
        "0002_rules",
        include_str!("../../migrations/0002_rules.sql"),
    ),
];

pub fn apply(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _migrations (
            name TEXT PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );",
    )?;

    for (name, sql) in MIGRATIONS {
        let already: bool = conn
            .query_row(
                "SELECT 1 FROM _migrations WHERE name = ?1",
                [name],
                |_row| Ok(true),
            )
            .unwrap_or(false);

        if !already {
            conn.execute_batch(sql)?;
            conn.execute("INSERT INTO _migrations (name) VALUES (?1)", [name])?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn table_exists(conn: &Connection, name: &str) -> bool {
        conn.query_row(
            "SELECT name FROM sqlite_master WHERE type='table' AND name=?1",
            [name],
            |row| row.get::<_, String>(0),
        )
        .is_ok()
    }

    #[test]
    fn applies_init_migration() {
        let conn = Connection::open_in_memory().unwrap();
        apply(&conn).unwrap();

        assert!(table_exists(&conn, "accounts"));
        assert!(table_exists(&conn, "categories"));
        assert!(table_exists(&conn, "transactions"));
        assert!(table_exists(&conn, "_migrations"));
    }

    #[test]
    fn seeds_default_categories() {
        let conn = Connection::open_in_memory().unwrap();
        apply(&conn).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM categories", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 9);

        let renda_kind: String = conn
            .query_row(
                "SELECT kind FROM categories WHERE name='Renda'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(renda_kind, "income");
    }

    #[test]
    fn migration_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        apply(&conn).unwrap();
        apply(&conn).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM categories", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 9, "re-running migrations should not duplicate seeds");
    }

    #[test]
    fn applies_rules_migration() {
        let conn = Connection::open_in_memory().unwrap();
        apply(&conn).unwrap();

        assert!(table_exists(&conn, "rules"));
        let applied: Vec<String> = {
            let mut stmt = conn
                .prepare("SELECT name FROM _migrations ORDER BY name")
                .unwrap();
            stmt.query_map([], |r| r.get::<_, String>(0))
                .unwrap()
                .map(|r| r.unwrap())
                .collect()
        };
        assert_eq!(
            applied,
            vec!["0001_init".to_string(), "0002_rules".to_string()]
        );
    }
}
