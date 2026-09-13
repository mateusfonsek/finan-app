//! What the agent asked for, so "I trust it" can become "I saw it".
//!
//! In memory only, and gone when the app quits: a durable record of what an
//! agent read from someone's statement would be a new privacy problem created
//! to solve a convenience one.

use std::collections::VecDeque;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

pub const CAPACITY: usize = 50;

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct CallEntry {
    /// Local time, `YYYY-MM-DDTHH:MM:SS`. The reader is in the same timezone as
    /// the server — it is the same machine.
    pub at: String,
    pub tool: String,
    /// Arguments as received, already truncated for display.
    pub args: String,
    pub ok: bool,
    pub error: Option<String>,
}

pub struct CallLog {
    entries: Mutex<VecDeque<CallEntry>>,
}

impl CallLog {
    pub fn new() -> CallLog {
        CallLog { entries: Mutex::new(VecDeque::with_capacity(CAPACITY)) }
    }

    pub fn push(&self, entry: CallEntry) {
        let mut entries = self.entries.lock().expect("call log mutex poisoned");
        entries.push_front(entry);
        if entries.len() > CAPACITY {
            entries.pop_back();
        }
    }

    pub fn entries(&self) -> Vec<CallEntry> {
        self.entries
            .lock()
            .expect("call log mutex poisoned")
            .iter()
            .cloned()
            .collect()
    }
}

impl Default for CallLog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(tool: &str) -> CallEntry {
        CallEntry {
            at: "2026-09-13T10:00:00".into(),
            tool: tool.into(),
            args: "{}".into(),
            ok: true,
            error: None,
        }
    }

    /// Newest first: the screen reads top-down and the last call is the one
    /// being looked for.
    #[test]
    fn the_most_recent_call_comes_first() {
        let log = CallLog::new();
        log.push(entry("list_transactions"));
        log.push(entry("get_trend"));

        let found = log.entries();

        assert_eq!(found[0].tool, "get_trend");
        assert_eq!(found[1].tool, "list_transactions");
    }

    #[test]
    fn it_never_grows_past_the_cap() {
        let log = CallLog::new();
        for i in 0..(CAPACITY + 10) {
            log.push(entry(&format!("tool_{i}")));
        }

        let found = log.entries();

        assert_eq!(found.len(), CAPACITY);
        assert_eq!(found[0].tool, format!("tool_{}", CAPACITY + 9), "newest kept");
        assert_eq!(found[CAPACITY - 1].tool, format!("tool_10"), "oldest dropped");
    }

    #[test]
    fn a_failed_call_is_logged_too() {
        let log = CallLog::new();
        log.push(CallEntry {
            ok: false,
            error: Some("tool disabled".into()),
            ..entry("create_rule")
        });

        let found = log.entries();

        assert!(!found[0].ok);
        assert_eq!(found[0].error.as_deref(), Some("tool disabled"));
    }

    #[test]
    fn a_new_log_is_empty() {
        assert!(CallLog::new().entries().is_empty());
    }
}
