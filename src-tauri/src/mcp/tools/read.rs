//! The six read tools.
//!
//! Each one is shaped around a question someone actually asks ("how did my
//! month go?"), not around a table. That is why `get_month_summary` answers
//! with four aggregations at once: an agent that needs four round-trips to
//! describe a month spends its context on plumbing.

use rusqlite::Connection;
use serde_json::{json, Value};

use crate::commands::bills::validate_month;
use crate::commands::transactions::TransactionFilters;
use crate::commands::{categories, rules, summary, transactions};
use crate::error::{AppError, AppResult};
use crate::locale::LocalePack;
use crate::mcp::config::McpConfig;

fn opt_str(args: &Value, key: &str) -> Option<String> {
    args.get(key).and_then(Value::as_str).map(str::to_string)
}

fn opt_u32(args: &Value, key: &str) -> Option<u32> {
    args.get(key).and_then(Value::as_u64).map(|n| n as u32)
}

/// A month argument reaches SQL as a `LIKE` prefix, so it is validated with the
/// same strict check the rest of the app uses — never a looser one here.
fn checked_month(args: &Value) -> AppResult<Option<String>> {
    match opt_str(args, "month") {
        Some(m) => {
            validate_month(&m)?;
            Ok(Some(m))
        }
        None => Ok(None),
    }
}

pub fn list_transactions(conn: &Connection, cfg: &McpConfig, args: &Value) -> AppResult<Value> {
    let month = checked_month(args)?;
    let filters = TransactionFilters {
        account_id: None,
        month,
        category_id: args.get("category_id").and_then(Value::as_i64),
        q: opt_str(args, "q"),
        limit: opt_u32(args, "limit"),
    };
    let mut rows = transactions::list(conn, &filters)?;

    if let Some(cutoff) = cfg.cutoff() {
        rows.retain(|t| t.date >= cutoff);
    }
    if args.get("uncategorized_only").and_then(Value::as_bool) == Some(true) {
        rows.retain(|t| t.category_id.is_none());
    }

    Ok(serde_json::to_value(rows)?)
}

pub fn get_month_summary(
    conn: &Connection,
    pack: &LocalePack,
    _cfg: &McpConfig,
    args: &Value,
) -> AppResult<Value> {
    let month = checked_month(args)?;
    let m = month.as_deref();
    Ok(json!({
        "kpis": summary::kpis(conn, m)?,
        "by_category": summary::by_category(conn, m)?,
        "income_sources": summary::income(conn, pack, m)?,
        "investments": summary::investments(conn, m)?,
    }))
}

pub fn get_trend(conn: &Connection, cfg: &McpConfig, args: &Value) -> AppResult<Value> {
    let asked = opt_u32(args, "months_back").unwrap_or(12);
    // The window is a ceiling, not a suggestion: asking for 60 months inside a
    // 12-month window returns 12.
    let months_back = if cfg.window_months == 0 {
        asked
    } else {
        asked.min(cfg.window_months)
    };
    Ok(serde_json::to_value(summary::by_month(conn, months_back)?)?)
}

pub fn list_bills(conn: &Connection, _cfg: &McpConfig, args: &Value) -> AppResult<Value> {
    let month = opt_str(args, "month")
        .ok_or_else(|| AppError::Invalid("month is required".into()))?;
    validate_month(&month)?;
    Ok(serde_json::to_value(rules::calendar_events_with_conn(conn, &month)?)?)
}

pub fn list_categories(conn: &Connection, _cfg: &McpConfig, _args: &Value) -> AppResult<Value> {
    Ok(serde_json::to_value(categories::all(conn)?)?)
}

pub fn list_rules(conn: &Connection, _cfg: &McpConfig, _args: &Value) -> AppResult<Value> {
    Ok(serde_json::to_value(rules::all(conn)?)?)
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
        crate::db::seed_from_pack(&conn, &crate::locale::LocalePack::embedded_pt_br()).unwrap();
        conn.execute("INSERT INTO accounts (name) VALUES ('Checking')", []).unwrap();
        conn
    }

    fn tx(conn: &Connection, date: &str, amount: &str, description: &str) -> i64 {
        conn.execute(
            "INSERT INTO transactions (account_id, date, amount, description)
             VALUES (1, ?1, ?2, ?3)",
            rusqlite::params![date, amount, description],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    fn cfg(window_months: u32) -> McpConfig {
        let mut c = McpConfig {
            enabled: true,
            tools: Default::default(),
            window_months,
        };
        for (name, _) in crate::mcp::config::TOOL_DEFAULTS {
            c.tools.insert((*name).to_string(), true);
        }
        c
    }

    #[test]
    fn list_transactions_returns_what_is_there() {
        let conn = seeded();
        tx(&conn, "2026-09-01", "-42.00", "MERCADO");

        let out = list_transactions(&conn, &cfg(0), &json!({})).unwrap();

        let rows = out.as_array().expect("an array of transactions");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["description"], "MERCADO");
        assert_eq!(rows[0]["amount"], "-42.00", "money stays a decimal string");
    }

    /// The whole point of the window: what falls outside it does not exist for
    /// the agent, no matter what it asks for.
    #[test]
    fn list_transactions_hides_what_is_outside_the_window() {
        let conn = seeded();
        tx(&conn, "2019-01-05", "-10.00", "ANCIENT");
        tx(&conn, "2026-09-01", "-42.00", "RECENT");

        let out = list_transactions(&conn, &cfg(12), &json!({})).unwrap();

        let descriptions: Vec<&str> = out
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["description"].as_str().unwrap())
            .collect();
        assert!(descriptions.contains(&"RECENT"));
        assert!(!descriptions.contains(&"ANCIENT"), "the window is not decorative");
    }

    #[test]
    fn no_window_shows_the_whole_history() {
        let conn = seeded();
        tx(&conn, "2019-01-05", "-10.00", "ANCIENT");

        let out = list_transactions(&conn, &cfg(0), &json!({})).unwrap();

        assert_eq!(out.as_array().unwrap().len(), 1);
    }

    #[test]
    fn list_transactions_honours_a_limit() {
        let conn = seeded();
        for i in 1..=5 {
            tx(&conn, &format!("2026-09-0{i}"), "-1.00", "X");
        }

        let out = list_transactions(&conn, &cfg(0), &json!({ "limit": 2 })).unwrap();

        assert_eq!(out.as_array().unwrap().len(), 2);
    }

    /// One call instead of four: the shape is the reason these tools exist.
    #[test]
    fn get_month_summary_answers_in_one_payload() {
        let conn = seeded();
        tx(&conn, "2026-09-01", "-42.00", "MERCADO");
        tx(&conn, "2026-09-02", "3000.00", "SALARY");
        let pack = crate::locale::LocalePack::embedded_pt_br();

        let out = get_month_summary(&conn, &pack, &cfg(0), &json!({ "month": "2026-09" })).unwrap();

        assert!(out.get("kpis").is_some());
        assert!(out.get("by_category").is_some());
        assert!(out.get("income_sources").is_some());
        assert!(out.get("investments").is_some());
        assert_eq!(out["kpis"]["expense"], "42.00");
    }

    #[test]
    fn get_month_summary_needs_a_real_month() {
        let conn = seeded();
        let pack = crate::locale::LocalePack::embedded_pt_br();

        let out = get_month_summary(&conn, &pack, &cfg(0), &json!({ "month": "setembro" }));

        assert!(out.is_err(), "a malformed month must not reach SQL as a LIKE prefix");
    }

    #[test]
    fn get_trend_returns_a_month_series() {
        let conn = seeded();
        tx(&conn, "2026-09-01", "-42.00", "MERCADO");

        let out = get_trend(&conn, &cfg(0), &json!({ "months_back": 3 })).unwrap();

        assert!(out.as_array().is_some());
    }

    #[test]
    fn list_categories_comes_from_the_active_pack() {
        let conn = seeded();

        let out = list_categories(&conn, &cfg(0), &json!({})).unwrap();

        assert!(!out.as_array().unwrap().is_empty());
        assert!(out.as_array().unwrap()[0].get("name").is_some());
    }

    #[test]
    fn list_rules_is_empty_until_there_are_rules() {
        let conn = Connection::open_in_memory().unwrap();
        migrations::apply(&conn).unwrap();

        let out = list_rules(&conn, &cfg(0), &json!({})).unwrap();

        assert!(out.as_array().unwrap().is_empty());
    }

    #[test]
    fn list_bills_needs_a_real_month() {
        let conn = seeded();

        assert!(list_bills(&conn, &cfg(0), &json!({ "month": "2026-9" })).is_err());
    }
}
