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
use crate::mcp::tools::read::oldest_allowed_month;

/// A pattern this short would match essentially every uncategorized
/// description at once — `create_rule` has no preview and no undo, so a
/// one- or two-character pattern is a single call that recategorizes the
/// agent's entire visible window. The UI keeps its own, more permissive
/// check: it has a preview screen the agent does not.
const MIN_MCP_PATTERN_LEN: usize = 3;

/// Caps how much one `categorize_transactions` call can touch. The window
/// already bounds *which* transactions are reachable; this bounds how much of
/// that reachable set a single call — possibly one step in an agent's retry
/// loop over bad ids — can rewrite at once.
const MAX_CATEGORIZE_BATCH: usize = 200;

fn ids(args: &Value) -> AppResult<Vec<i64>> {
    let raw = args
        .get("transaction_ids")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::Invalid("transaction_ids is required".into()))?;
    if raw.is_empty() {
        return Err(AppError::Invalid("transaction_ids must not be empty".into()));
    }
    if raw.len() > MAX_CATEGORIZE_BATCH {
        return Err(AppError::Invalid(format!(
            "transaction_ids must not exceed {MAX_CATEGORIZE_BATCH} (got: {})",
            raw.len()
        )));
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
        let date: Option<String> = match conn
            .query_row("SELECT date FROM transactions WHERE id = ?1", [id], |r| r.get(0))
        {
            Ok(d) => Some(d),
            // Absence is a normal outcome worth its own message below; any
            // other failure (a locked or corrupt database) must surface as
            // itself, not collapse into "not found".
            Err(rusqlite::Error::QueryReturnedNoRows) => None,
            Err(e) => return Err(e.into()),
        };
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

pub fn create_rule(conn: &mut Connection, cfg: &McpConfig, args: &Value) -> AppResult<Value> {
    let patterns: Vec<String> = args
        .get("patterns")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_str).map(str::to_string).collect())
        .unwrap_or_default();

    if let Some(short) = patterns.iter().find(|p| p.trim().chars().count() < MIN_MCP_PATTERN_LEN) {
        return Err(AppError::Invalid(format!(
            "pattern {short:?} is shorter than {MIN_MCP_PATTERN_LEN} characters — too broad a \
             match for a call with no preview and no undo"
        )));
    }

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

    // `rules::create_with_scope` already rejects an empty pattern list and an
    // impossible due day — the tool does not re-validate what the shared
    // function owns. The backfill it triggers is bounded by the window: the
    // agent's rule categorizes only what it could already see, while the
    // user's own "new rule" form (`rules::create`) still reaches all of it.
    Ok(serde_json::to_value(rules::create_with_scope(conn, input, cfg.cutoff().as_deref())?)?)
}

pub fn settle_bill(conn: &Connection, cfg: &McpConfig, args: &Value) -> AppResult<Value> {
    let rule_id = args
        .get("rule_id")
        .and_then(Value::as_i64)
        .ok_or_else(|| AppError::Invalid("rule_id is required".into()))?;
    let due_month = args
        .get("due_month")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError::Invalid("due_month is required".into()))?;
    bills::validate_month(due_month)?;

    // `bills::settle` upserts on `(rule_id, due_month)`, so a `due_month`
    // outside the window is not just an out-of-bounds read: it is a write that
    // can overwrite — and clear the `transaction_id` of — a settlement the
    // agent was never allowed to see in the first place.
    if let Some(oldest) = oldest_allowed_month(cfg) {
        if due_month < oldest.as_str() {
            return Err(AppError::Invalid(format!(
                "due_month {due_month} is older than the allowed window (oldest: {oldest})"
            )));
        }
    }

    let transaction_id = args.get("transaction_id").and_then(Value::as_i64);
    if let Some(id) = transaction_id {
        assert_within_window(conn, cfg, &[id])?;
    }
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

    /// A missing id and one merely outside the window are different failures:
    /// only one of them could plausibly move inside the window some day, so
    /// collapsing them into one message would hide that from the caller.
    #[test]
    fn categorize_reports_a_missing_id_differently_from_one_outside_the_window() {
        let conn = seeded();
        let old = tx(&conn, "2019-01-05");

        let missing_err =
            categorize(&conn, &cfg(12), &json!({ "transaction_ids": [999], "category_id": 1 }))
                .unwrap_err()
                .to_string();
        let old_err =
            categorize(&conn, &cfg(12), &json!({ "transaction_ids": [old], "category_id": 1 }))
                .unwrap_err()
                .to_string();

        assert!(missing_err.contains("not found"), "{missing_err}");
        assert!(old_err.contains("window"), "{old_err}");
        assert_ne!(missing_err, old_err);
        assert_eq!(category_of(&conn, old), None, "nothing is written either way");
    }

    /// The agent's rule creation is bounded by the same window as everything
    /// else it can touch: an old transaction that matches the new pattern must
    /// stay exactly as it was.
    #[test]
    fn a_rule_created_through_mcp_does_not_recategorize_a_transaction_older_than_the_window() {
        let mut conn = seeded();
        let old = tx(&conn, "2019-01-05");
        let recent = tx(&conn, "2026-09-01");

        create_rule(&mut conn, &cfg(12), &json!({ "patterns": ["MERCADO"], "category_id": 1 }))
            .unwrap();

        assert_eq!(category_of(&conn, old), None, "outside the window: left alone");
        assert_eq!(category_of(&conn, recent), Some(1), "inside the window: backfilled");
    }

    /// The escape hatch survives here too: no window configured means the
    /// agent's rule reaches the whole history, same as the user's own rule form.
    #[test]
    fn with_no_window_configured_a_rule_created_through_mcp_still_backfills_the_whole_history() {
        let mut conn = seeded();
        let old = tx(&conn, "2019-01-05");

        create_rule(&mut conn, &cfg(0), &json!({ "patterns": ["MERCADO"], "category_id": 1 }))
            .unwrap();

        assert_eq!(category_of(&conn, old), Some(1));
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

    /// `settle_bill` is the one write path that takes a transaction id
    /// directly: without this check it would be the back door every other
    /// write closes.
    #[test]
    fn settling_a_bill_with_a_transaction_outside_the_window_is_refused() {
        let mut conn = seeded();
        create_rule(&mut conn, &cfg(0), &json!({ "patterns": ["MERCADO"], "category_id": 1 }))
            .unwrap();
        let rule: i64 = conn.query_row("SELECT id FROM rules", [], |r| r.get(0)).unwrap();
        let old = tx(&conn, "2019-01-05");

        let out = settle_bill(
            &conn,
            &cfg(12),
            &json!({ "rule_id": rule, "due_month": "2026-09", "transaction_id": old }),
        );

        assert!(out.is_err());
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM bill_settlements", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0, "nothing may be written when the transaction is refused");
    }

    #[test]
    fn settling_with_a_transaction_inside_the_window_still_works() {
        let mut conn = seeded();
        create_rule(&mut conn, &cfg(0), &json!({ "patterns": ["MERCADO"], "category_id": 1 }))
            .unwrap();
        let rule: i64 = conn.query_row("SELECT id FROM rules", [], |r| r.get(0)).unwrap();
        let recent = tx(&conn, "2026-09-01");

        settle_bill(
            &conn,
            &cfg(12),
            &json!({ "rule_id": rule, "due_month": "2026-09", "transaction_id": recent }),
        )
        .unwrap();

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

    /// `bills::settle` upserts on `(rule_id, due_month)`: a `due_month`
    /// outside the window would let the agent silently overwrite — and clear
    /// the `transaction_id` of — a settlement it cannot even read.
    #[test]
    fn settle_bill_refuses_a_due_month_outside_the_window() {
        let mut conn = seeded();
        create_rule(&mut conn, &cfg(0), &json!({ "patterns": ["MERCADO"], "category_id": 1 }))
            .unwrap();
        let rule: i64 = conn.query_row("SELECT id FROM rules", [], |r| r.get(0)).unwrap();

        let out = settle_bill(&conn, &cfg(12), &json!({ "rule_id": rule, "due_month": "2019-01" }));

        assert!(out.is_err(), "2019-01 is far outside any realistic 12-month window");
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM bill_settlements", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0, "nothing may be written when due_month is refused");
    }

    #[test]
    fn settle_bill_with_no_window_reaches_any_due_month() {
        let mut conn = seeded();
        create_rule(&mut conn, &cfg(0), &json!({ "patterns": ["MERCADO"], "category_id": 1 }))
            .unwrap();
        let rule: i64 = conn.query_row("SELECT id FROM rules", [], |r| r.get(0)).unwrap();

        assert!(
            settle_bill(&conn, &cfg(0), &json!({ "rule_id": rule, "due_month": "2019-01" })).is_ok()
        );
    }

    /// The single-character pattern that would recategorize essentially every
    /// uncategorized transaction in the window at once — no preview, no undo.
    #[test]
    fn create_rule_refuses_a_pattern_shorter_than_the_minimum() {
        let mut conn = seeded();

        let out = create_rule(&mut conn, &cfg(0), &json!({ "patterns": ["a"], "category_id": 1 }));

        assert!(out.is_err());
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM rules", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 0, "nothing may be written when a pattern is refused");
    }

    #[test]
    fn create_rule_accepts_a_pattern_at_the_minimum_length() {
        let mut conn = seeded();

        assert!(
            create_rule(&mut conn, &cfg(0), &json!({ "patterns": ["ABC"], "category_id": 1 }))
                .is_ok()
        );
    }

    /// The UI's own pattern check is untouched: it has a preview screen the
    /// MCP path does not, so it stays free to accept whatever it accepts
    /// today. This only pins that `rules::create_with_scope` itself has no
    /// minimum — the MCP-only floor lives in `create_rule` above.
    #[test]
    fn the_shared_rule_creation_path_has_no_minimum_pattern_length() {
        let mut conn = seeded();

        let input = NewRule {
            patterns: vec!["a".to_string()],
            category_id: 1,
            priority: 0,
            due_day: None,
            pay_lead_months: 0,
            display_name: None,
        };

        assert!(rules::create_with_scope(&mut conn, input, None).is_ok());
    }

    #[test]
    fn categorize_refuses_a_batch_over_the_cap() {
        let conn = seeded();
        let ids: Vec<i64> = (0..(MAX_CATEGORIZE_BATCH + 1) as i64).collect();

        let out = categorize(&conn, &cfg(0), &json!({ "transaction_ids": ids, "category_id": 1 }));

        assert!(out.is_err());
    }

    #[test]
    fn categorize_accepts_a_batch_at_the_cap() {
        let conn = seeded();
        let ids: Vec<i64> = (0..MAX_CATEGORIZE_BATCH as i64).map(|_| tx(&conn, "2026-09-01")).collect();

        let out = categorize(&conn, &cfg(0), &json!({ "transaction_ids": ids, "category_id": 1 }));

        assert!(out.is_ok());
    }
}
