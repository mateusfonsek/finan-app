//! What the agent asked for, so "I trust it" can become "I saw it".
//!
//! In memory only, and gone when the app quits: a durable record of what an
//! agent read from someone's statement would be a new privacy problem created
//! to solve a convenience one.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

pub const CAPACITY: usize = 50;

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct CallEntry {
    /// Assigned by `CallLog::push`, in call order. `at` has only one-second
    /// resolution and two calls can land in the same second, so the screen
    /// keys its list on this instead — a duplicate key throws in Svelte 5's
    /// release build, not just in dev.
    pub id: u64,
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
    next_id: AtomicU64,
}

impl CallLog {
    pub fn new() -> CallLog {
        CallLog { entries: Mutex::new(VecDeque::with_capacity(CAPACITY)), next_id: AtomicU64::new(1) }
    }

    /// `entry.id` is ignored on the way in — the log itself is the only thing
    /// that hands out ids, so two callers can never race to assign the same
    /// one.
    pub fn push(&self, mut entry: CallEntry) {
        entry.id = self.next_id.fetch_add(1, Ordering::Relaxed);
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
            id: 0,
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

    /// The whole reason `id` exists: two calls landing in the same second must
    /// still be distinguishable, since the screen keys its list on this field.
    #[test]
    fn two_calls_in_the_same_second_get_different_ids() {
        let log = CallLog::new();
        log.push(entry("list_transactions"));
        log.push(entry("list_transactions"));

        let found = log.entries();

        assert_ne!(found[0].id, found[1].id);
    }

    #[test]
    fn ids_are_assigned_by_the_log_not_the_caller() {
        let log = CallLog::new();
        log.push(CallEntry { id: 999, ..entry("create_rule") });

        assert_ne!(log.entries()[0].id, 999, "the log must own id assignment, not trust the caller");
    }

    #[test]
    fn a_new_log_is_empty() {
        assert!(CallLog::new().entries().is_empty());
    }
}
