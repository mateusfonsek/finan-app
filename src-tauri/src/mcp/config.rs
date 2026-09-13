use std::collections::BTreeMap;

use chrono::{Datelike, Local, NaiveDate};
use rusqlite::Connection;

use crate::commands::app_settings::{get_setting, set_setting};
use crate::error::AppResult;

const ENABLED_KEY: &str = "mcp_enabled";
const WINDOW_KEY: &str = "mcp_window_months";
const DEFAULT_WINDOW_MONTHS: u32 = 12;

/// Every tool the server knows, and whether it is on when nobody has said.
/// Reading is useful immediately; writing is a decision, so it starts off.
pub const TOOL_DEFAULTS: &[(&str, bool)] = &[
    ("list_transactions", true),
    ("get_month_summary", true),
    ("get_trend", true),
    ("list_bills", true),
    ("list_categories", true),
    ("list_rules", true),
    ("categorize_transactions", false),
    ("create_rule", false),
    ("settle_bill", false),
];

fn tool_key(name: &str) -> String {
    format!("mcp_tool_{name}")
}

#[derive(Debug, Clone)]
pub struct McpConfig {
    pub enabled: bool,
    pub tools: BTreeMap<String, bool>,
    /// `0` means no limit.
    pub window_months: u32,
}

impl McpConfig {
    pub fn load(conn: &Connection) -> AppResult<McpConfig> {
        let enabled = get_setting(conn, ENABLED_KEY)?.as_deref() == Some("1");

        let mut tools = BTreeMap::new();
        for (name, default) in TOOL_DEFAULTS {
            let stored = get_setting(conn, &tool_key(name))?;
            let on = match stored.as_deref() {
                Some("1") => true,
                Some("0") => false,
                // A value we did not write is not a reason to change the
                // posture — fall back to the default, which is safe by design.
                _ => *default,
            };
            tools.insert((*name).to_string(), on);
        }

        let window_months = get_setting(conn, WINDOW_KEY)?
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(DEFAULT_WINDOW_MONTHS);

        Ok(McpConfig { enabled, tools, window_months })
    }

    /// A name that is not in `TOOL_DEFAULTS` is not a tool, so it is off — the
    /// registry, not the caller, decides what exists.
    pub fn tool_enabled(&self, name: &str) -> bool {
        self.tools.get(name).copied().unwrap_or(false)
    }

    /// Oldest date the agent may see or touch. Month boundary rather than "N
    /// days ago" so the answer does not drift with the day it is asked.
    pub fn cutoff(&self) -> Option<String> {
        if self.window_months == 0 {
            return None;
        }
        let today = Local::now().date_naive();
        let total = today.year() * 12 + today.month0() as i32 - (self.window_months as i32 - 1);
        let (year, month0) = (total.div_euclid(12), total.rem_euclid(12));
        NaiveDate::from_ymd_opt(year, month0 as u32 + 1, 1).map(|d| d.to_string())
    }
}

pub fn set_enabled(conn: &Connection, on: bool) -> AppResult<()> {
    set_setting(conn, ENABLED_KEY, if on { "1" } else { "0" })
}

pub fn set_tool(conn: &Connection, name: &str, on: bool) -> AppResult<()> {
    set_setting(conn, &tool_key(name), if on { "1" } else { "0" })
}

pub fn set_window(conn: &Connection, months: u32) -> AppResult<()> {
    set_setting(conn, WINDOW_KEY, &months.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;
    use rusqlite::Connection;

    fn fresh_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        migrations::apply(&conn).unwrap();
        conn
    }

    /// Defaults are the security posture: reading works out of the box,
    /// writing never does until someone says so.
    #[test]
    fn defaults_enable_reads_and_disable_writes() {
        let conn = fresh_conn();
        let cfg = McpConfig::load(&conn).unwrap();

        assert!(!cfg.enabled, "the server itself starts off");
        assert!(cfg.tool_enabled("list_transactions"));
        assert!(!cfg.tool_enabled("categorize_transactions"));
        assert_eq!(cfg.window_months, 12);
    }

    #[test]
    fn an_unknown_tool_is_never_enabled() {
        let conn = fresh_conn();
        assert!(!McpConfig::load(&conn).unwrap().tool_enabled("drop_everything"));
    }

    #[test]
    fn a_stored_choice_wins_over_the_default() {
        let conn = fresh_conn();
        set_tool(&conn, "categorize_transactions", true).unwrap();
        set_tool(&conn, "list_transactions", false).unwrap();

        let cfg = McpConfig::load(&conn).unwrap();

        assert!(cfg.tool_enabled("categorize_transactions"));
        assert!(!cfg.tool_enabled("list_transactions"));
    }

    /// The cutoff is what makes the window real: every read and every write
    /// compares against it.
    #[test]
    fn the_cutoff_is_the_first_day_of_the_window() {
        let conn = fresh_conn();
        set_window(&conn, 12).unwrap();
        let cfg = McpConfig::load(&conn).unwrap();

        let cutoff = cfg.cutoff().expect("12 months is a limit");
        assert_eq!(cutoff.len(), 10, "cutoff is a full date: {cutoff}");
        assert!(cutoff.ends_with("-01"), "windows start at a month boundary");
    }

    #[test]
    fn no_window_means_no_cutoff() {
        let conn = fresh_conn();
        set_window(&conn, 0).unwrap();
        assert!(McpConfig::load(&conn).unwrap().cutoff().is_none());
    }

    #[test]
    fn an_unusable_window_falls_back_to_the_default() {
        let conn = fresh_conn();
        crate::commands::app_settings::set_setting(&conn, "mcp_window_months", "banana").unwrap();
        assert_eq!(McpConfig::load(&conn).unwrap().window_months, 12);
    }
}
