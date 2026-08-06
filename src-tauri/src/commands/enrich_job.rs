//! Tauri surface for background enrichment.
//!
//! This file exists for one specific reason: the previous command was a
//! synchronous `pub fn`, and Tauri runs synchronous commands ON THE MAIN
//! THREAD. Since the body did blocking HTTP per tax id, with a courtesy pause
//! between them, the app went tens of seconds without servicing the event loop
//! and macOS drew the spinning wait cursor. Here the command only spawns a
//! thread and returns at once — what must never come back to this file is I/O.

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::ipc::Channel;
use tauri::{AppHandle, Manager, State};

use crate::commands::suggestions::AutoClassifyReport;
use crate::db::Db;
use crate::enrich;
use crate::enrich::job::{run_enrichment, EnrichEvent};
use crate::error::{AppError, AppResult};
use crate::locale::LocaleState;

/// State of the running job. Follows the precedent set by `PendingOpen`
/// (`commands/openfile.rs:8`): managed, mutable, simple state.
#[derive(Default)]
pub struct EnrichJob {
    cancel: AtomicBool,
    running: AtomicBool,
}

impl EnrichJob {
    /// Claims the job. `false` when one is already running — two concurrent
    /// jobs would contend for the same provider and double the external traffic.
    fn claim(&self) -> bool {
        let taken = self
            .running
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok();
        if taken {
            // Cleared here rather than in release: a cancellation arriving after
            // the job already finished would leave the flag set, and the NEXT job
            // would be born cancelled — stopping on its own, with nothing on
            // screen to explain it.
            self.cancel.store(false, Ordering::SeqCst);
        }
        taken
    }

    fn release(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    fn request_cancel(&self) {
        self.cancel.store(true, Ordering::SeqCst);
    }

    /// Only tests ask this: the thread reads the flag directly, through
    /// [`Self::cancel_flag`].
    #[cfg(test)]
    fn is_cancelled(&self) -> bool {
        self.cancel.load(Ordering::SeqCst)
    }

    fn cancel_flag(&self) -> &AtomicBool {
        &self.cancel
    }
}

/// Spawns the enrichment on a thread and returns immediately.
///
/// It is still a synchronous `fn`, and that is correct: the body does not
/// block, it only spawns.
#[tauri::command]
#[specta::specta]
pub fn start_cnpj_enrichment(
    app: AppHandle,
    account_id: Option<i64>,
    on_event: Channel<EnrichEvent>,
) -> AppResult<()> {
    // Locale snapshot: the thread takes its own copy instead of holding the
    // mutex for the job's ~30s.
    let (pack, active) = {
        let locale = app.state::<LocaleState>();
        let db = app.state::<Db>();
        let pack = locale.pack.lock().expect("locale mutex poisoned").clone();
        let conn = db.conn.lock().expect("db mutex poisoned");
        let active = enrich::is_active(&conn, &pack);
        (pack, active)
    };

    // Off, or a locale with no provider: empty report, no thread, no network.
    // Emitting anyway is what lets the screen handle a single path instead of
    // asking beforehand whether the call is worth making.
    if !active {
        let _ = on_event.send(EnrichEvent::Started { total: 0 });
        let _ = on_event.send(EnrichEvent::Finished {
            report: AutoClassifyReport {
                created_rules: Vec::new(),
                txs_classified: 0,
                unresolved: Vec::new(),
            },
        });
        return Ok(());
    }

    let job = app.state::<EnrichJob>();
    if !job.claim() {
        return Err(AppError::Invalid("enrichment already running".into()));
    }

    let Some(provider) = enrich::provider::for_name(&pack.manifest.tax_id.provider) else {
        job.release();
        return Err(AppError::Invalid("locale has no provider".into()));
    };

    let handle = app.clone();
    std::thread::spawn(move || {
        let db = handle.state::<Db>();
        let job = handle.state::<EnrichJob>();

        let result = run_enrichment(
            &db.conn,
            &pack,
            provider.as_ref(),
            account_id,
            job.cancel_flag(),
            &mut |event| {
                // A closed channel (the window is gone) is no reason to bring
                // anything down: the work already done stays valid in the database.
                let _ = on_event.send(event);
            },
        );

        if let Err(e) = result {
            let _ = on_event.send(EnrichEvent::Aborted {
                message: e.to_string(),
            });
        }

        job.release();
    });

    Ok(())
}

/// Requests a stop. What was already created stays — cancelling is stopping
/// work, not undoing the work done.
#[tauri::command]
#[specta::specta]
pub fn cancel_cnpj_enrichment(job: State<'_, EnrichJob>) {
    job.request_cancel();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claim_succeeds_once_then_refuses_until_released() {
        let job = EnrichJob::default();

        assert!(job.claim(), "the first start claims the job");
        assert!(!job.claim(), "a second start is refused while it runs");

        job.release();
        assert!(job.claim(), "after releasing, a new job can start");
    }

    #[test]
    fn claim_clears_a_stale_cancel_flag() {
        let job = EnrichJob::default();
        job.claim();
        job.request_cancel();
        assert!(job.is_cancelled());
        job.release();

        // Without the cleanup, the next job would be born cancelled and stop at once.
        job.claim();
        assert!(!job.is_cancelled(), "a new job starts uncancelled");
    }
}
