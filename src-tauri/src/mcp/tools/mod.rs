//! The tool registry: the single list of what exists, what it is called and
//! what it accepts.
//!
//! Names and descriptions are in English regardless of the active locale pack.
//! They are protocol, read by a model — not interface, read by a person. The
//! data they return is whatever the database holds, in the user's own language.

pub mod read;
pub mod write;

use rusqlite::Connection;
use serde_json::{json, Value};

use crate::error::{AppError, AppResult};
use crate::locale::LocalePack;
use crate::mcp::config::McpConfig;

pub struct ToolSpec {
    pub name: &'static str,
    pub description: &'static str,
    pub schema: fn() -> Value,
}

fn obj(properties: Value, required: Value) -> Value {
    json!({ "type": "object", "properties": properties, "required": required })
}

pub fn specs() -> Vec<ToolSpec> {
    vec![
        ToolSpec {
            name: "list_transactions",
            description: "List bank transactions, newest first. Filter by month (YYYY-MM), \
                          category, a text fragment of the description, or only the ones with \
                          no category yet.",
            schema: || {
                obj(
                    json!({
                        "month": { "type": "string", "description": "YYYY-MM" },
                        "category_id": { "type": "integer" },
                        "q": { "type": "string", "description": "text contained in the description" },
                        "uncategorized_only": { "type": "boolean" },
                        "limit": { "type": "integer" }
                    }),
                    json!([]),
                )
            },
        },
        ToolSpec {
            name: "get_month_summary",
            description: "Everything about one month in a single answer: income, expense and \
                          balance, spending per category, who paid the money in, and how much \
                          went to investments. `month` is required unless the user has allowed \
                          unlimited history — only then does omitting it mean all time.",
            schema: || obj(json!({ "month": { "type": "string", "description": "YYYY-MM" } }), json!([])),
        },
        ToolSpec {
            name: "get_trend",
            description: "Income and expense month by month, to see whether spending is rising \
                          or falling.",
            schema: || obj(json!({ "months_back": { "type": "integer", "default": 12 } }), json!([])),
        },
        ToolSpec {
            name: "list_bills",
            description: "Recurring bills for a month, each with its due day and whether it is \
                          already paid, still due, or overdue.",
            schema: || obj(json!({ "month": { "type": "string", "description": "YYYY-MM" } }), json!(["month"])),
        },
        ToolSpec {
            name: "list_categories",
            description: "Every category, with the id needed to categorize a transaction.",
            schema: || obj(json!({}), json!([])),
        },
        ToolSpec {
            name: "list_rules",
            description: "Every auto-categorization rule. Check this before creating one, so an \
                          existing rule is not duplicated.",
            schema: || obj(json!({}), json!([])),
        },
        ToolSpec {
            name: "categorize_transactions",
            description: "Assign a category to one or more transactions. Pass null as the \
                          category to clear it.",
            schema: || {
                obj(
                    json!({
                        "transaction_ids": { "type": "array", "items": { "type": "integer" } },
                        "category_id": { "type": ["integer", "null"] }
                    }),
                    json!(["transaction_ids"]),
                )
            },
        },
        ToolSpec {
            name: "create_rule",
            description: "Create a rule that categorizes any transaction whose description \
                          contains one of the given fragments, and applies it to what is already \
                          imported.",
            schema: || {
                obj(
                    json!({
                        "patterns": { "type": "array", "items": { "type": "string" } },
                        "category_id": { "type": "integer" },
                        "display_name": { "type": "string" },
                        "due_day": { "type": "integer", "minimum": 1, "maximum": 31 },
                        "priority": { "type": "integer", "default": 0 }
                    }),
                    json!(["patterns", "category_id"]),
                )
            },
        },
        ToolSpec {
            name: "settle_bill",
            description: "Mark a bill occurrence as settled for a month, optionally pointing at \
                          the transaction that paid it.",
            schema: || {
                obj(
                    json!({
                        "rule_id": { "type": "integer" },
                        "due_month": { "type": "string", "description": "YYYY-MM" },
                        "transaction_id": { "type": ["integer", "null"] }
                    }),
                    json!(["rule_id", "due_month"]),
                )
            },
        },
    ]
}

/// Returns `true` when this tool changed the database, so the caller knows to
/// tell the webview its screens are stale.
pub struct Outcome {
    pub value: Value,
    pub wrote: bool,
}

pub fn dispatch(
    conn: &mut Connection,
    pack: &LocalePack,
    cfg: &McpConfig,
    name: &str,
    args: &Value,
) -> AppResult<Outcome> {
    if !cfg.tool_enabled(name) {
        // Same answer for "off" and "does not exist": which tools are enabled
        // is not something an unauthenticated caller gets to enumerate.
        return Err(AppError::Invalid(format!("unknown or disabled tool: {name}")));
    }
    let value = match name {
        "list_transactions" => read::list_transactions(conn, cfg, args)?,
        "get_month_summary" => read::get_month_summary(conn, pack, cfg, args)?,
        "get_trend" => read::get_trend(conn, cfg, args)?,
        "list_bills" => read::list_bills(conn, cfg, args)?,
        "list_categories" => read::list_categories(conn, cfg, args)?,
        "list_rules" => read::list_rules(conn, cfg, args)?,
        "categorize_transactions" => {
            return Ok(Outcome { value: write::categorize(conn, cfg, args)?, wrote: true })
        }
        "create_rule" => {
            return Ok(Outcome { value: write::create_rule(conn, cfg, args)?, wrote: true })
        }
        "settle_bill" => {
            return Ok(Outcome { value: write::settle_bill(conn, cfg, args)?, wrote: true })
        }
        other => return Err(AppError::Invalid(format!("unknown or disabled tool: {other}"))),
    };
    Ok(Outcome { value, wrote: false })
}
