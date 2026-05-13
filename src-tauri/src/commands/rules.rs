use rusqlite::params;
use tauri::State;

use crate::db::Db;
use crate::domain::rule::{NewRule, Rule, UpdateRule};
use crate::error::{AppError, AppResult};

#[tauri::command]
#[specta::specta]
pub fn list_rules(db: State<'_, Db>) -> AppResult<Vec<Rule>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    let mut stmt = conn.prepare(
        "SELECT id, pattern, category_id, priority, created_at
         FROM rules
         ORDER BY priority DESC, created_at DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Rule {
            id: row.get(0)?,
            pattern: row.get(1)?,
            category_id: row.get(2)?,
            priority: row.get(3)?,
            created_at: row.get(4)?,
        })
    })?;
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(AppError::from)
}

#[tauri::command]
#[specta::specta]
pub fn create_rule(db: State<'_, Db>, input: NewRule) -> AppResult<Rule> {
    if input.pattern.trim().is_empty() {
        return Err(AppError::Invalid("pattern must not be empty".into()));
    }
    let conn = db.conn.lock().expect("db mutex poisoned");
    conn.execute(
        "INSERT INTO rules (pattern, category_id, priority) VALUES (?1, ?2, ?3)",
        params![input.pattern.trim(), input.category_id, input.priority],
    )?;
    let id = conn.last_insert_rowid();
    fetch_rule(&conn, id)
}

#[tauri::command]
#[specta::specta]
pub fn update_rule(db: State<'_, Db>, rule_id: i64, input: UpdateRule) -> AppResult<Rule> {
    if input.pattern.trim().is_empty() {
        return Err(AppError::Invalid("pattern must not be empty".into()));
    }
    let conn = db.conn.lock().expect("db mutex poisoned");
    let changed = conn.execute(
        "UPDATE rules SET pattern = ?1, category_id = ?2, priority = ?3 WHERE id = ?4",
        params![
            input.pattern.trim(),
            input.category_id,
            input.priority,
            rule_id
        ],
    )?;
    if changed == 0 {
        return Err(AppError::Invalid(format!("rule {rule_id} not found")));
    }
    fetch_rule(&conn, rule_id)
}

#[tauri::command]
#[specta::specta]
pub fn delete_rule(db: State<'_, Db>, rule_id: i64) -> AppResult<()> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    let changed = conn.execute("DELETE FROM rules WHERE id = ?1", params![rule_id])?;
    if changed == 0 {
        return Err(AppError::Invalid(format!("rule {rule_id} not found")));
    }
    Ok(())
}

/// Run all rules on transactions with `category_id IS NULL` (manual categorization
/// is never overwritten). When multiple rules match the same transaction, the one
/// with the highest priority wins; ties broken by most recent created_at.
/// Returns the count of transactions newly categorized.
#[tauri::command]
#[specta::specta]
pub fn apply_rules_to_uncategorized(
    db: State<'_, Db>,
    account_id: Option<i64>,
) -> AppResult<u32> {
    let mut conn = db.conn.lock().expect("db mutex poisoned");
    apply_rules_internal(&mut conn, account_id)
}

/// Shared engine used both by the public command and by insert_transactions.
pub fn apply_rules_internal(
    conn: &mut rusqlite::Connection,
    account_id: Option<i64>,
) -> AppResult<u32> {
    let scope_filter = match account_id {
        Some(_) => "AND account_id = ?1",
        None => "",
    };
    let sql = format!(
        "UPDATE transactions
         SET category_id = (
             SELECT r.category_id FROM rules r
             WHERE LOWER(transactions.description) LIKE '%' || LOWER(r.pattern) || '%'
             ORDER BY r.priority DESC, r.created_at DESC
             LIMIT 1
         )
         WHERE category_id IS NULL
           AND EXISTS (
               SELECT 1 FROM rules r
               WHERE LOWER(transactions.description) LIKE '%' || LOWER(r.pattern) || '%'
           )
           {scope_filter}",
    );
    let changed = if let Some(id) = account_id {
        conn.execute(&sql, params![id])?
    } else {
        conn.execute(&sql, [])?
    };
    Ok(changed as u32)
}

fn fetch_rule(conn: &rusqlite::Connection, id: i64) -> AppResult<Rule> {
    conn.query_row(
        "SELECT id, pattern, category_id, priority, created_at FROM rules WHERE id = ?1",
        params![id],
        |row| {
            Ok(Rule {
                id: row.get(0)?,
                pattern: row.get(1)?,
                category_id: row.get(2)?,
                priority: row.get(3)?,
                created_at: row.get(4)?,
            })
        },
    )
    .map_err(AppError::from)
}

#[cfg(test)]
mod tests {
    use super::apply_rules_internal;
    use crate::db::migrations;
    use rusqlite::{params, Connection};

    fn fresh_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        migrations::apply(&conn).unwrap();
        conn
    }

    fn category_id(conn: &Connection, name: &str) -> i64 {
        conn.query_row(
            "SELECT id FROM categories WHERE name = ?1",
            params![name],
            |r| r.get(0),
        )
        .unwrap()
    }

    fn insert_account(conn: &Connection) -> i64 {
        conn.execute(
            "INSERT INTO accounts (name, bank, ofx_acctid) VALUES ('test', NULL, 'A1')",
            [],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    fn insert_tx(conn: &Connection, account_id: i64, description: &str, category_id: Option<i64>) -> i64 {
        conn.execute(
            "INSERT INTO transactions (account_id, date, amount, description, category_id, ofx_fitid)
             VALUES (?1, '2026-04-12', '10.00', ?2, ?3, NULL)",
            params![account_id, description, category_id],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    fn insert_rule(conn: &Connection, pattern: &str, cat: i64, priority: i32) -> i64 {
        conn.execute(
            "INSERT INTO rules (pattern, category_id, priority) VALUES (?1, ?2, ?3)",
            params![pattern, cat, priority],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    #[test]
    fn applies_rule_case_insensitive() {
        let mut conn = fresh_conn();
        let acc = insert_account(&conn);
        let transporte = category_id(&conn, "Transporte");
        insert_rule(&conn, "uber", transporte, 0);
        let tx_id = insert_tx(&conn, acc, "UBER * TRIP 12345", None);

        let n = apply_rules_internal(&mut conn, None).unwrap();
        assert_eq!(n, 1);

        let cat: Option<i64> = conn
            .query_row(
                "SELECT category_id FROM transactions WHERE id = ?1",
                params![tx_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(cat, Some(transporte));
    }

    #[test]
    fn higher_priority_rule_wins() {
        let mut conn = fresh_conn();
        let acc = insert_account(&conn);
        let transporte = category_id(&conn, "Transporte");
        let outros = category_id(&conn, "Outros");

        insert_rule(&conn, "uber", outros, 0);
        insert_rule(&conn, "uber trip", transporte, 10);

        let tx_id = insert_tx(&conn, acc, "uber trip via app", None);
        apply_rules_internal(&mut conn, None).unwrap();

        let cat: Option<i64> = conn
            .query_row(
                "SELECT category_id FROM transactions WHERE id = ?1",
                params![tx_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(cat, Some(transporte), "priority 10 wins over priority 0");
    }

    #[test]
    fn manual_categorization_is_never_overwritten() {
        let mut conn = fresh_conn();
        let acc = insert_account(&conn);
        let mercado = category_id(&conn, "Mercado");
        let transporte = category_id(&conn, "Transporte");

        insert_rule(&conn, "uber", transporte, 0);
        let tx_id = insert_tx(&conn, acc, "UBER trip", Some(mercado));

        let n = apply_rules_internal(&mut conn, None).unwrap();
        assert_eq!(n, 0, "manual category preserved");

        let cat: Option<i64> = conn
            .query_row(
                "SELECT category_id FROM transactions WHERE id = ?1",
                params![tx_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(cat, Some(mercado));
    }

    #[test]
    fn scope_to_account_id() {
        let mut conn = fresh_conn();
        conn.execute(
            "INSERT INTO accounts (name, bank, ofx_acctid) VALUES ('a1', NULL, 'A1'),
                                                                  ('a2', NULL, 'A2')",
            [],
        )
        .unwrap();
        let a1: i64 = conn
            .query_row("SELECT id FROM accounts WHERE ofx_acctid='A1'", [], |r| r.get(0))
            .unwrap();
        let a2: i64 = conn
            .query_row("SELECT id FROM accounts WHERE ofx_acctid='A2'", [], |r| r.get(0))
            .unwrap();
        let transporte = category_id(&conn, "Transporte");
        insert_rule(&conn, "uber", transporte, 0);
        insert_tx(&conn, a1, "uber a1", None);
        insert_tx(&conn, a2, "uber a2", None);

        let n = apply_rules_internal(&mut conn, Some(a1)).unwrap();
        assert_eq!(n, 1, "scope must limit to account a1");

        let a1_cat: Option<i64> = conn
            .query_row(
                "SELECT category_id FROM transactions WHERE account_id = ?1",
                params![a1],
                |r| r.get(0),
            )
            .unwrap();
        let a2_cat: Option<i64> = conn
            .query_row(
                "SELECT category_id FROM transactions WHERE account_id = ?1",
                params![a2],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(a1_cat, Some(transporte));
        assert_eq!(a2_cat, None);
    }

    #[test]
    fn pattern_with_no_match_does_nothing() {
        let mut conn = fresh_conn();
        let acc = insert_account(&conn);
        let transporte = category_id(&conn, "Transporte");
        insert_rule(&conn, "uber", transporte, 0);
        insert_tx(&conn, acc, "padaria do bairro", None);

        let n = apply_rules_internal(&mut conn, None).unwrap();
        assert_eq!(n, 0);
    }
}
