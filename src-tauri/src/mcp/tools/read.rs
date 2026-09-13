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
    let limit = opt_u32(args, "limit");
    let filters = TransactionFilters {
        account_id: None,
        month,
        category_id: args.get("category_id").and_then(Value::as_i64),
        q: opt_str(args, "q"),
        limit: None,
    };
    let mut rows = transactions::list(conn, &filters)?;

    if let Some(cutoff) = cfg.cutoff() {
        rows.retain(|t| t.date >= cutoff);
    }
    if args.get("uncategorized_only").and_then(Value::as_bool) == Some(true) {
        rows.retain(|t| t.category_id.is_none());
    }
    // `limit` must be the last thing applied: this is a local single-user
    // SQLite database, so fetching a few extra rows to filter in Rust is
    // cheap — but asking SQL to LIMIT before the window/category filters run
    // would silently hand back fewer rows than requested (or than exist).
    if let Some(n) = limit {
        rows.truncate(n as usize);
    }

    Ok(serde_json::to_value(rows)?)
}

/// Oldest month the window still allows, as `YYYY-MM` — the first seven
/// characters of `cutoff()`'s `YYYY-MM-01`. `None` when there is no window.
fn oldest_allowed_month(cfg: &McpConfig) -> Option<String> {
    cfg.cutoff().map(|c| c[..7].to_string())
}

pub fn get_month_summary(
    conn: &Connection,
    pack: &LocalePack,
    cfg: &McpConfig,
    args: &Value,
) -> AppResult<Value> {
    let month = checked_month(args)?;

    // `income_sources` returns a label per counterparty — who pays the user —
    // which is exactly the kind of detail the window promises to hide. The
    // other three aggregations would silently go all-time too, so a missing
    // or too-old `month` is refused rather than quietly ignoring the window.
    if let Some(oldest) = oldest_allowed_month(cfg) {
        match month.as_deref() {
            None => {
                return Err(AppError::Invalid(format!(
                    "month is required: history before {oldest} is outside the allowed window"
                )));
            }
            Some(m) if m < oldest.as_str() => {
                return Err(AppError::Invalid(format!(
                    "month {m} is older than the allowed window (oldest: {oldest})"
                )));
            }
            _ => {}
        }
    }

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

    /// The window filter must run before the limit is applied, or a window
    /// that happens to trim the SQL-ordered result could hand back fewer rows
    /// than requested even though enough recent ones exist.
    #[test]
    fn a_limit_is_applied_after_the_window_filter_not_before() {
        let conn = seeded();
        tx(&conn, "2019-01-05", "-1.00", "ANCIENT1");
        tx(&conn, "2019-01-06", "-1.00", "ANCIENT2");
        for i in 1..=4 {
            tx(&conn, &format!("2026-09-0{i}"), "-1.00", "RECENT");
        }

        let out = list_transactions(&conn, &cfg(12), &json!({ "limit": 2 })).unwrap();

        let rows = out.as_array().unwrap();
        assert_eq!(rows.len(), 2, "there are enough recent rows to fill the limit");
        assert!(
            rows.iter().all(|r| r["description"] == "RECENT"),
            "the window must exclude ancient rows, not just shrink the count"
        );
    }

    /// The bug this guards against: `uncategorized_only` is a Rust-side filter
    /// with no correlation to the SQL `ORDER BY date`, so a SQL-side `LIMIT`
    /// can fill its quota with already-categorized rows before the filter
    /// ever runs, undercounting the answer.
    #[test]
    fn a_limit_is_applied_after_uncategorized_only_not_before() {
        let conn = seeded();
        let mercado: i64 = conn
            .query_row("SELECT id FROM categories WHERE name = 'Mercado'", [], |r| r.get(0))
            .unwrap();
        tx(&conn, "2026-09-05", "-1.00", "CAT1");
        tx(&conn, "2026-09-04", "-1.00", "UNCAT1");
        tx(&conn, "2026-09-03", "-1.00", "CAT2");
        tx(&conn, "2026-09-02", "-1.00", "UNCAT2");
        tx(&conn, "2026-09-01", "-1.00", "CAT3");
        conn.execute(
            "UPDATE transactions SET category_id = ?1 WHERE description IN ('CAT1', 'CAT2', 'CAT3')",
            rusqlite::params![mercado],
        )
        .unwrap();

        let out = list_transactions(
            &conn,
            &cfg(0),
            &json!({ "uncategorized_only": true, "limit": 2 }),
        )
        .unwrap();

        let rows = out.as_array().unwrap();
        assert_eq!(rows.len(), 2, "both uncategorized rows must survive the limit");
        assert!(rows.iter().all(|r| r["category_id"].is_null()));
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

    /// `income_sources` scans full history to detect recurrence and returns a
    /// label per counterparty — an all-time summary would leak exactly what
    /// the window promises to hide.
    #[test]
    fn get_month_summary_without_a_month_is_refused_when_a_window_is_set() {
        let conn = seeded();
        let pack = crate::locale::LocalePack::embedded_pt_br();

        let err = get_month_summary(&conn, &pack, &cfg(12), &json!({})).unwrap_err();

        assert!(
            err.to_string().contains("window"),
            "the error must name the window as the reason: {err}"
        );
    }

    #[test]
    fn get_month_summary_for_a_month_older_than_the_window_is_refused() {
        let conn = seeded();
        let pack = crate::locale::LocalePack::embedded_pt_br();

        let out = get_month_summary(&conn, &pack, &cfg(12), &json!({ "month": "2019-01" }));

        assert!(out.is_err(), "2019-01 is far outside any realistic 12-month window");
    }

    #[test]
    fn get_month_summary_for_a_month_inside_the_window_still_works() {
        let conn = seeded();
        let pack = crate::locale::LocalePack::embedded_pt_br();
        let this_month = chrono::Local::now().format("%Y-%m").to_string();

        let out = get_month_summary(&conn, &pack, &cfg(12), &json!({ "month": this_month })).unwrap();

        assert!(out.get("kpis").is_some());
    }

    /// The escape hatch survives: a window of `0` means no limit, so `month`
    /// stays optional and omitting it means all time, same as before this fix.
    #[test]
    fn get_month_summary_with_no_window_still_allows_no_month() {
        let conn = seeded();
        let pack = crate::locale::LocalePack::embedded_pt_br();

        let out = get_month_summary(&conn, &pack, &cfg(0), &json!({}));

        assert!(out.is_ok(), "no window configured means all-time stays available");
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
