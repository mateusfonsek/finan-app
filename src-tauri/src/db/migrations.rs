use rusqlite::Connection;

use crate::error::AppResult;

const MIGRATIONS: &[(&str, &str)] = &[
    ("0001_init", include_str!("../../migrations/0001_init.sql")),
    (
        "0002_rules",
        include_str!("../../migrations/0002_rules.sql"),
    ),
    (
        "0003_rules_due_day",
        include_str!("../../migrations/0003_rules_due_day.sql"),
    ),
    (
        "0004_rules_display_name",
        include_str!("../../migrations/0004_rules_display_name.sql"),
    ),
    (
        "0005_compras_category",
        include_str!("../../migrations/0005_compras_category.sql"),
    ),
    (
        "0006_transfer_category",
        include_str!("../../migrations/0006_transfer_category.sql"),
    ),
    (
        "0007_investments",
        include_str!("../../migrations/0007_investments.sql"),
    ),
    (
        "0008_delete_renda_category",
        include_str!("../../migrations/0008_delete_renda_category.sql"),
    ),
    (
        "0009_investment_green_color",
        include_str!("../../migrations/0009_investment_green_color.sql"),
    ),
    (
        "0010_drop_income_cnpj_rules",
        include_str!("../../migrations/0010_drop_income_cnpj_rules.sql"),
    ),
    (
        "0011_account_kind",
        include_str!("../../migrations/0011_account_kind.sql"),
    ),
    (
        "0012_cc_seed",
        include_str!("../../migrations/0012_cc_seed.sql"),
    ),
    (
        "0013_composite_fitid_unique",
        include_str!("../../migrations/0013_composite_fitid_unique.sql"),
    ),
    (
        "0014_category_keys",
        include_str!("../../migrations/0014_category_keys.sql"),
    ),
    (
        "0015_education_pets_categories",
        include_str!("../../migrations/0015_education_pets_categories.sql"),
    ),
];

/// Applies pending migrations. Returns `true` when this call created a **brand
/// new** database (i.e. `0001_init` had not run before), so the caller can seed
/// it from the active locale pack without clobbering an existing user's data.
pub fn apply(conn: &Connection) -> AppResult<bool> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _migrations (
            name TEXT PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );",
    )?;

    let already_initialized: bool = conn
        .query_row(
            "SELECT 1 FROM _migrations WHERE name = '0001_init'",
            [],
            |_row| Ok(true),
        )
        .unwrap_or(false);
    let fresh_db = !already_initialized;

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

    Ok(fresh_db)
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
        assert_eq!(count, 13);

        // 'Renda' foi removida em 0008. Garantir que não sobrou no final.
        let renda_exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM categories WHERE name = 'Renda'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(renda_exists, 0, "Renda foi removida pela migration 0008");
    }

    #[test]
    fn migration_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        apply(&conn).unwrap();
        apply(&conn).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM categories", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 13, "re-running migrations should not duplicate seeds");

        let rule_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM rules
                 WHERE pattern IN ('Pagamento de fatura','Aplicação RDB','Resgate RDB')",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(rule_count, 3, "seed rules must not duplicate on re-run");
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
            vec![
                "0001_init".to_string(),
                "0002_rules".to_string(),
                "0003_rules_due_day".to_string(),
                "0004_rules_display_name".to_string(),
                "0005_compras_category".to_string(),
                "0006_transfer_category".to_string(),
                "0007_investments".to_string(),
                "0008_delete_renda_category".to_string(),
                "0009_investment_green_color".to_string(),
                "0010_drop_income_cnpj_rules".to_string(),
                "0011_account_kind".to_string(),
                "0012_cc_seed".to_string(),
                "0013_composite_fitid_unique".to_string(),
                "0014_category_keys".to_string(),
                "0015_education_pets_categories".to_string(),
            ]
        );
    }

    #[test]
    fn backfills_category_keys() {
        let conn = Connection::open_in_memory().unwrap();
        apply(&conn).unwrap();

        // Every seeded category must have a stable key after 0014.
        let missing: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM categories WHERE key IS NULL",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(missing, 0, "todas as categorias seedadas têm key");

        let market: String = conn
            .query_row(
                "SELECT key FROM categories WHERE name = 'Mercado'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(market, "market");
    }

    #[test]
    fn reports_fresh_db_only_on_first_apply() {
        let conn = Connection::open_in_memory().unwrap();
        assert!(apply(&conn).unwrap(), "primeira aplicação = DB novo");
        assert!(!apply(&conn).unwrap(), "reaplicar não é DB novo");
    }
}
