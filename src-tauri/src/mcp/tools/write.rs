//! The three write tools.
//!
//! Each one is off until someone turns it on, and each is bounded by the same
//! data window the read tools obey. A tool that could change what the agent is
//! not allowed to see would turn the window into decoration.

use rusqlite::Connection;
use serde_json::{json, Value};

use crate::commands::{bills, rules, transactions};
use crate::domain::rule::NewRule;
use crate::error::{AppError, AppResult};
use crate::mcp::config::McpConfig;

fn ids(args: &Value) -> AppResult<Vec<i64>> {
    let raw = args
        .get("transaction_ids")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::Invalid("transaction_ids is required".into()))?;
    if raw.is_empty() {
        return Err(AppError::Invalid("transaction_ids must not be empty".into()));
    }
    raw.iter()
        .map(|v| v.as_i64().ok_or_else(|| AppError::Invalid("transaction ids must be integers".into())))
        .collect()
}

/// Every id must be inside the window BEFORE anything is written: a batch that
/// applied its valid half would leave a state nobody asked for.
fn assert_within_window(conn: &Connection, cfg: &McpConfig, ids: &[i64]) -> AppResult<()> {
    let Some(cutoff) = cfg.cutoff() else { return Ok(()) };
    for id in ids {
        let date: Option<String> = conn
            .query_row("SELECT date FROM transactions WHERE id = ?1", [id], |r| r.get(0))
            .ok();
        match date {
            Some(d) if d >= cutoff => {}
            Some(_) => {
                return Err(AppError::Invalid(format!(
                    "transaction {id} is older than the configured data window"
                )))
            }
            None => return Err(AppError::Invalid(format!("transaction {id} not found"))),
        }
    }
    Ok(())
}

pub fn categorize(conn: &Connection, cfg: &McpConfig, args: &Value) -> AppResult<Value> {
    let targets = ids(args)?;
    assert_within_window(conn, cfg, &targets)?;

    let category_id = match args.get("category_id") {
        Some(Value::Null) | None => None,
        Some(v) => Some(
            v.as_i64()
                .ok_or_else(|| AppError::Invalid("category_id must be an integer or null".into()))?,
        ),
    };

    for id in &targets {
        transactions::set_category(conn, *id, category_id)?;
    }
    Ok(json!({ "updated": targets.len() }))
}

pub fn create_rule(conn: &mut Connection, _cfg: &McpConfig, args: &Value) -> AppResult<Value> {
    let patterns: Vec<String> = args
        .get("patterns")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_str).map(str::to_string).collect())
        .unwrap_or_default();

    let input = NewRule {
        patterns,
        category_id: args
            .get("category_id")
            .and_then(Value::as_i64)
            .ok_or_else(|| AppError::Invalid("category_id is required".into()))?,
        priority: args.get("priority").and_then(Value::as_i64).unwrap_or(0) as i32,
        due_day: args.get("due_day").and_then(Value::as_i64).map(|d| d as i32),
        pay_lead_months: 0,
        display_name: args.get("display_name").and_then(Value::as_str).map(str::to_string),
    };

    // `rules::create` already rejects an empty pattern list and an impossible
    // due day — the tool does not re-validate what the shared function owns.
    Ok(serde_json::to_value(rules::create(conn, input)?)?)
}

pub fn settle_bill(conn: &Connection, _cfg: &McpConfig, args: &Value) -> AppResult<Value> {
    let rule_id = args
        .get("rule_id")
        .and_then(Value::as_i64)
        .ok_or_else(|| AppError::Invalid("rule_id is required".into()))?;
    let due_month = args
        .get("due_month")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError::Invalid("due_month is required".into()))?;
    bills::validate_month(due_month)?;

    let transaction_id = args.get("transaction_id").and_then(Value::as_i64);
    bills::settle(conn, rule_id, due_month, transaction_id)?;
    Ok(json!({ "settled": true }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;
    use crate::mcp::config::McpConfig;
    use rusqlite::Connection;
    use serde_json::json;

    fn seeded() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();
        migrations::apply(&conn).unwrap();
        conn.execute(
            "INSERT INTO categories (name, color_token, kind) VALUES ('Market', NULL, 'expense')",
            [],
        )
        .unwrap();
        conn.execute("INSERT INTO accounts (name) VALUES ('Checking')", []).unwrap();
        conn
    }

    fn tx(conn: &Connection, date: &str) -> i64 {
        conn.execute(
            "INSERT INTO transactions (account_id, date, amount, description)
             VALUES (1, ?1, '-42.00', 'MERCADO')",
            rusqlite::params![date],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    fn cfg(window_months: u32) -> McpConfig {
        let mut c = McpConfig { enabled: true, tools: Default::default(), window_months };
        for (name, _) in crate::mcp::config::TOOL_DEFAULTS {
            c.tools.insert((*name).to_string(), true);
        }
        c
    }

    fn category_of(conn: &Connection, id: i64) -> Option<i64> {
        conn.query_row("SELECT category_id FROM transactions WHERE id = ?1", [id], |r| r.get(0))
            .unwrap()
    }

    #[test]
    fn categorize_sets_the_category() {
        let conn = seeded();
        let t = tx(&conn, "2026-09-01");

        categorize(&conn, &cfg(0), &json!({ "transaction_ids": [t], "category_id": 1 })).unwrap();

        assert_eq!(category_of(&conn, t), Some(1));
    }

    #[test]
    fn categorize_handles_a_batch() {
        let conn = seeded();
        let a = tx(&conn, "2026-09-01");
        let b = tx(&conn, "2026-09-02");

        let out =
            categorize(&conn, &cfg(0), &json!({ "transaction_ids": [a, b], "category_id": 1 }))
                .unwrap();

        assert_eq!(out["updated"], 2);
        assert_eq!(category_of(&conn, a), Some(1));
        assert_eq!(category_of(&conn, b), Some(1));
    }

    /// The window is the whole reason writes are bounded: a tool that can touch
    /// what the agent cannot see is a back door into the data it was denied.
    #[test]
    fn categorize_refuses_a_transaction_outside_the_window() {
        let conn = seeded();
        let old = tx(&conn, "2019-01-05");

        let out = categorize(&conn, &cfg(12), &json!({ "transaction_ids": [old], "category_id": 1 }));

        assert!(out.is_err(), "outside the window must be refused, not silently skipped");
        assert_eq!(category_of(&conn, old), None, "and nothing may be written");
    }

    /// All or nothing: a batch that half-applied would leave the user with a
    /// state neither they nor the agent asked for.
    #[test]
    fn a_batch_with_one_bad_id_writes_nothing() {
        let conn = seeded();
        let good = tx(&conn, "2026-09-01");
        let old = tx(&conn, "2019-01-05");

        let out = categorize(
            &conn,
            &cfg(12),
            &json!({ "transaction_ids": [good, old], "category_id": 1 }),
        );

        assert!(out.is_err());
        assert_eq!(category_of(&conn, good), None, "the valid half must not land either");
    }

    #[test]
    fn categorize_needs_at_least_one_id() {
        let conn = seeded();
        assert!(categorize(&conn, &cfg(0), &json!({ "transaction_ids": [] })).is_err());
    }

    #[test]
    fn create_rule_persists_and_returns_it() {
        let mut conn = seeded();

        let out = create_rule(
            &mut conn,
            &cfg(0),
            &json!({ "patterns": ["MERCADO"], "category_id": 1, "display_name": "Market" }),
        )
        .unwrap();

        assert_eq!(out["patterns"][0], "MERCADO");
        let count: i64 =
            conn.query_row("SELECT COUNT(*) FROM rules", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn create_rule_rejects_an_empty_pattern_list() {
        let mut conn = seeded();
        assert!(create_rule(&mut conn, &cfg(0), &json!({ "patterns": [], "category_id": 1 })).is_err());
    }

    #[test]
    fn settle_bill_records_the_occurrence() {
        let mut conn = seeded();
        create_rule(&mut conn, &cfg(0), &json!({ "patterns": ["MERCADO"], "category_id": 1 }))
            .unwrap();
        let rule: i64 = conn.query_row("SELECT id FROM rules", [], |r| r.get(0)).unwrap();

        settle_bill(&conn, &cfg(0), &json!({ "rule_id": rule, "due_month": "2026-09" })).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM bill_settlements", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn settle_bill_needs_a_real_month() {
        let conn = seeded();
        assert!(settle_bill(&conn, &cfg(0), &json!({ "rule_id": 1, "due_month": "set/26" })).is_err());
    }
}
