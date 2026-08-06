//! Superfície Tauri do enriquecimento em segundo plano.
//!
//! Este arquivo existe por um motivo específico: o comando anterior era `pub fn`
//! síncrono, e o Tauri executa comandos síncronos NO MAIN THREAD. Como o corpo
//! fazia HTTP bloqueante por tax id, com pausa de cortesia entre eles, o app
//! ficava dezenas de segundos sem atender o event loop e o macOS desenhava o
//! cursor de espera. Aqui o comando só dispara uma thread e volta na hora — o
//! que não pode voltar a existir neste arquivo é I/O.

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::ipc::Channel;
use tauri::{AppHandle, Manager, State};

use crate::commands::suggestions::AutoClassifyReport;
use crate::db::Db;
use crate::enrich;
use crate::enrich::job::{run_enrichment, EnrichEvent};
use crate::error::{AppError, AppResult};
use crate::locale::LocaleState;

/// Estado do job em andamento. Segue o precedente de `PendingOpen`
/// (`commands/openfile.rs:8`): estado gerenciado, mutável, simples.
#[derive(Default)]
pub struct EnrichJob {
    cancel: AtomicBool,
    running: AtomicBool,
}

impl EnrichJob {
    /// Toma o job para si. `false` quando já há um rodando — dois jobs
    /// concorrentes disputariam o mesmo provedor e dobrariam o tráfego externo.
    fn claim(&self) -> bool {
        let taken = self
            .running
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok();
        if taken {
            // Limpar aqui, e não no release: um cancelamento que chega depois do
            // job já ter acabado deixaria a flag ligada, e o PRÓXIMO job nasceria
            // cancelado — parando sozinho, sem explicação nenhuma na tela.
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

    /// Só os testes perguntam isso: a thread lê a flag direto, via
    /// [`Self::cancel_flag`].
    #[cfg(test)]
    fn is_cancelled(&self) -> bool {
        self.cancel.load(Ordering::SeqCst)
    }

    fn cancel_flag(&self) -> &AtomicBool {
        &self.cancel
    }
}

/// Dispara o enriquecimento numa thread e retorna imediatamente.
///
/// Continua sendo `fn` síncrono, e isso está correto: o corpo não bloqueia, só
/// faz `spawn`.
#[tauri::command]
#[specta::specta]
pub fn start_cnpj_enrichment(
    app: AppHandle,
    account_id: Option<i64>,
    on_event: Channel<EnrichEvent>,
) -> AppResult<()> {
    // Snapshot do locale: a thread leva a própria cópia em vez de segurar o
    // mutex pelos ~30s do job.
    let (pack, active) = {
        let locale = app.state::<LocaleState>();
        let db = app.state::<Db>();
        let pack = locale.pack.lock().expect("locale mutex poisoned").clone();
        let conn = db.conn.lock().expect("db mutex poisoned");
        let active = enrich::is_active(&conn, &pack);
        (pack, active)
    };

    // Desligado, ou locale sem provedor: relatório vazio, nenhuma thread,
    // nenhuma rede. Emitir mesmo assim é o que permite a tela tratar um caminho
    // só, em vez de perguntar antes se vale a pena chamar.
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
        return Err(AppError::Invalid("enriquecimento já em andamento".into()));
    }

    let Some(provider) = enrich::provider::for_name(&pack.manifest.tax_id.provider) else {
        job.release();
        return Err(AppError::Invalid("locale sem provedor".into()));
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
                // Canal fechado (janela sumiu) não é motivo para derrubar nada:
                // o trabalho já feito continua válido no banco.
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

/// Pede parada. O que já foi criado permanece — cancelar é parar de trabalhar,
/// não desfazer o trabalho feito.
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

        assert!(job.claim(), "primeiro início toma o job");
        assert!(!job.claim(), "segundo início é recusado enquanto roda");

        job.release();
        assert!(job.claim(), "depois de liberar, um novo job pode começar");
    }

    #[test]
    fn claim_clears_a_stale_cancel_flag() {
        let job = EnrichJob::default();
        job.claim();
        job.request_cancel();
        assert!(job.is_cancelled());
        job.release();

        // Sem a limpeza, o próximo job nasceria cancelado e pararia na hora.
        job.claim();
        assert!(!job.is_cancelled(), "um job novo começa não-cancelado");
    }
}
