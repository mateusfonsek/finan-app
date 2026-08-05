//! Orquestração do enriquecimento por tax id.
//!
//! Separado dos comandos de propósito: aqui não há `State`, `AppHandle` nem
//! `Channel`. Só a lógica — que é o que torna possível testar ordem de eventos,
//! resiliência a falha e cancelamento sem tocar a rede nem subir um app Tauri.

use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use rusqlite::params;
use serde::Serialize;
use specta::Type;

use crate::commands::cnpj::CnpjResolution;
use crate::commands::rules::apply_rules_internal;
use crate::commands::suggestions::AutoClassifyReport;
use crate::domain::rule::Rule;
use crate::enrich::provider::TaxIdProvider;
use crate::enrich::{extract_tax_id, lookup_with};
use crate::error::AppResult;
use crate::locale::LocalePack;

/// O que a thread de fundo conta para a interface, em ordem.
///
/// `Started` carrega o denominador — é ele que compra a barra determinada que a
/// HIG prefere, sem estimativa inventada, porque a lista de tax ids únicos é
/// conhecida antes da primeira consulta.
///
/// `Failed` e `Aborted` são falhas de naturezas diferentes e a interface precisa
/// distingui-las: uma consulta que deu errado dentro de um loop que segue
/// adiante não é o trabalho ter morrido.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(tag = "kind")]
pub enum EnrichEvent {
    Started { total: u32 },
    Resolved { done: u32, label: String, rule: Rule },
    Unresolved { done: u32, resolution: CnpjResolution },
    Failed { done: u32, tax_id: String },
    Finished { report: AutoClassifyReport },
    Cancelled { report: AutoClassifyReport },
    Aborted { message: String },
}

/// Tax ids distintos nas descrições de saídas sem categoria.
///
/// Ordenado alfabeticamente porque `HashSet` não tem ordem: sem isso, o mesmo
/// extrato produziria uma sequência de progresso diferente a cada execução, e os
/// testes de ordem de eventos seriam intermitentes.
pub fn collect_unique_tax_ids(
    conn: &rusqlite::Connection,
    pack: &LocalePack,
    account_id: Option<i64>,
) -> AppResult<Vec<String>> {
    let (sql, scoped) = match account_id {
        Some(_) => (
            "SELECT description FROM transactions
             WHERE category_id IS NULL
               AND CAST(amount AS REAL) < 0
               AND account_id = ?1",
            true,
        ),
        None => (
            "SELECT description FROM transactions
             WHERE category_id IS NULL
               AND CAST(amount AS REAL) < 0",
            false,
        ),
    };

    let mut stmt = conn.prepare(sql)?;
    let descriptions: Vec<String> = if scoped {
        stmt.query_map(params![account_id.unwrap()], |r| r.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?
    } else {
        stmt.query_map([], |r| r.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?
    };

    let mut seen: HashSet<String> = HashSet::new();
    for d in descriptions {
        if let Some(id) = extract_tax_id(pack, &d) {
            seen.insert(id);
        }
    }
    let mut out: Vec<String> = seen.into_iter().collect();
    out.sort();
    Ok(out)
}

/// Roda o enriquecimento inteiro, contando o progresso pelo caminho.
///
/// Nunca segura o lock do banco através de uma espera de rede: cada trava é por
/// operação, como no resto do código. É isso que permite a interface continuar
/// consultando o banco enquanto este job roda.
pub fn run_enrichment(
    conn: &Mutex<rusqlite::Connection>,
    pack: &LocalePack,
    provider: &dyn TaxIdProvider,
    account_id: Option<i64>,
    cancel: &AtomicBool,
    on_event: &mut dyn FnMut(EnrichEvent),
) -> AppResult<()> {
    let tax_ids = {
        let c = conn.lock().expect("db mutex poisoned");
        collect_unique_tax_ids(&c, pack, account_id)?
    };

    on_event(EnrichEvent::Started {
        total: tax_ids.len() as u32,
    });

    let mut created_rules: Vec<Rule> = Vec::new();
    let mut unresolved: Vec<CnpjResolution> = Vec::new();
    let mut done: u32 = 0;
    let mut cancelled = false;
    let mut consulted = 0usize;

    for tax_id in &tax_ids {
        if cancel.load(Ordering::Relaxed) {
            cancelled = true;
            break;
        }

        // A regra existente é o cache: se já há uma, não há o que consultar.
        let has_rule = {
            let c = conn.lock().expect("db mutex poisoned");
            c.query_row(
                "SELECT 1 FROM rule_patterns WHERE pattern = ?1 LIMIT 1",
                params![tax_id],
                |_| Ok(()),
            )
            .is_ok()
        };
        if has_rule {
            done += 1;
            continue;
        }

        // Cortesia só entre consultas de verdade. Contar iterações puladas aqui
        // faria o job dormir por trabalho que não aconteceu.
        if consulted > 0 {
            std::thread::sleep(Duration::from_millis(provider.courtesy_delay_ms()));
        }
        consulted += 1;

        let enrichment = {
            let c = conn.lock().expect("db mutex poisoned");
            lookup_with(&c, tax_id, pack, provider)
        };

        done += 1;

        let enrichment = match enrichment {
            Ok(e) => e,
            Err(_) => {
                // Falha de uma consulta não derruba o job — mas também não é
                // engolida: vira evento contável.
                on_event(EnrichEvent::Failed {
                    done,
                    tax_id: tax_id.clone(),
                });
                continue;
            }
        };

        let resolution = CnpjResolution {
            cnpj: tax_id.clone(),
            razao_social: enrichment.company.legal_name.clone(),
            nome_fantasia: enrichment.company.trade_name.clone(),
            cnae_fiscal: enrichment.company.activity_code.clone(),
            cnae_fiscal_descricao: enrichment.company.activity_label.clone(),
            suggested_category_id: enrichment.suggested_category_id,
        };

        match resolution.suggested_category_id {
            Some(category_id) => {
                let rule = insert_rule(conn, tax_id, category_id, &resolution)?;
                let label = rule.display_name.clone().unwrap_or_else(|| tax_id.clone());
                created_rules.push(rule.clone());
                on_event(EnrichEvent::Resolved { done, label, rule });
            }
            None => {
                unresolved.push(resolution.clone());
                on_event(EnrichEvent::Unresolved { done, resolution });
            }
        }
    }

    // Aplicar as regras acontece mesmo num cancelamento: o que já foi criado
    // deve valer. Cancelar é parar de trabalhar, não desfazer o trabalho feito.
    let txs_classified = {
        let mut c = conn.lock().expect("db mutex poisoned");
        apply_rules_internal(&mut c, account_id)?
    };

    let report = AutoClassifyReport {
        created_rules,
        txs_classified,
        unresolved,
    };

    on_event(if cancelled {
        EnrichEvent::Cancelled { report }
    } else {
        EnrichEvent::Finished { report }
    });

    Ok(())
}

/// Cria a regra do tax id resolvido. Começa com um único trecho: o próprio id.
fn insert_rule(
    conn: &Mutex<rusqlite::Connection>,
    tax_id: &str,
    category_id: i64,
    resolution: &CnpjResolution,
) -> AppResult<Rule> {
    let c = conn.lock().expect("db mutex poisoned");
    let display = resolution
        .razao_social
        .clone()
        .or_else(|| resolution.nome_fantasia.clone());
    c.execute(
        "INSERT INTO rules (category_id, priority, due_day, display_name)
         VALUES (?1, 10, NULL, ?2)",
        params![category_id, display],
    )?;
    let id = c.last_insert_rowid();
    c.execute(
        "INSERT INTO rule_patterns (rule_id, pattern) VALUES (?1, ?2)",
        params![id, tax_id],
    )?;
    Ok(c.query_row(
        "SELECT id, category_id, priority, due_day, display_name, created_at
         FROM rules WHERE id = ?1",
        params![id],
        |row| {
            Ok(Rule {
                id: row.get(0)?,
                patterns: vec![tax_id.to_string()],
                category_id: row.get(1)?,
                priority: row.get(2)?,
                due_day: row.get(3)?,
                display_name: row.get(4)?,
                created_at: row.get(5)?,
            })
        },
    )?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;
    use crate::locale::LocalePack;
    use rusqlite::{params, Connection};

    fn fresh_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        migrations::apply(&conn).unwrap();
        conn
    }

    fn pack() -> LocalePack {
        LocalePack::embedded_pt_br()
    }

    /// Cria uma conta e devolve o id.
    fn seed_account(conn: &Connection) -> i64 {
        let n: i64 = conn
            .query_row("SELECT COUNT(*) FROM accounts", [], |r| r.get(0))
            .unwrap();
        conn.execute(
            "INSERT INTO accounts (name, bank, ofx_acctid, kind)
             VALUES ('Conta', NULL, ?1, 'checking')",
            params![n.to_string()],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    /// Transação de saída sem categoria — o único formato que o job olha.
    fn seed_tx(conn: &Connection, account_id: i64, description: &str) {
        conn.execute(
            "INSERT INTO transactions (account_id, date, amount, description, ofx_fitid, category_id)
             VALUES (?1, '2026-03-01', '-100.00', ?2, ?3, NULL)",
            params![account_id, description, format!("fit-{description}")],
        )
        .unwrap();
    }

    use crate::enrich::test_support::FakeProvider;
    use std::sync::atomic::AtomicBool;
    use std::sync::Mutex;

    /// Coletor de eventos: os testes afirmam sobre a sequência inteira.
    fn collect_events(
        conn: Connection,
        provider: &FakeProvider,
        account_id: Option<i64>,
        cancel: &AtomicBool,
    ) -> (Vec<EnrichEvent>, Mutex<Connection>) {
        let guarded = Mutex::new(conn);
        let mut events = Vec::new();
        run_enrichment(&guarded, &pack(), provider, account_id, cancel, &mut |e| {
            events.push(e)
        })
        .unwrap();
        (events, guarded)
    }

    /// Quantas regras existem para este tax id.
    ///
    /// Contar `rules` inteira não serve: as migrações 0006 e 0012 semeiam
    /// regras de transferência e cartão, então um banco recém-migrado já começa
    /// com 15. O que estes testes afirmam é sobre a regra DESTE tax id.
    fn rules_for(db: &Mutex<Connection>, tax_id: &str) -> i64 {
        db.lock()
            .unwrap()
            .query_row(
                "SELECT COUNT(*) FROM rule_patterns WHERE pattern = ?1",
                params![tax_id],
                |r| r.get(0),
            )
            .unwrap()
    }

    fn kinds(events: &[EnrichEvent]) -> Vec<&'static str> {
        events
            .iter()
            .map(|e| match e {
                EnrichEvent::Started { .. } => "started",
                EnrichEvent::Resolved { .. } => "resolved",
                EnrichEvent::Unresolved { .. } => "unresolved",
                EnrichEvent::Failed { .. } => "failed",
                EnrichEvent::Finished { .. } => "finished",
                EnrichEvent::Cancelled { .. } => "cancelled",
                EnrichEvent::Aborted { .. } => "aborted",
            })
            .collect()
    }

    #[test]
    fn emits_started_with_the_real_total_then_finishes() {
        let conn = fresh_conn();
        let acc = seed_account(&conn);
        seed_tx(&conn, acc, "Pix - ENERGISA - 09.095.183/0001-40 - ITAU");
        seed_tx(&conn, acc, "Pix - DEMERGE - 33.967.103/0001-84 - BB");
        let fake = FakeProvider::new(&[
            ("09095183000140", "4711301"), // mercado — mapeado
            ("33967103000184", "5611201"), // restaurante — mapeado
        ]);

        let (events, _db) = collect_events(conn, &fake, Some(acc), &AtomicBool::new(false));

        assert_eq!(
            kinds(&events),
            vec!["started", "resolved", "resolved", "finished"]
        );
        match &events[0] {
            EnrichEvent::Started { total } => assert_eq!(*total, 2),
            other => panic!("primeiro evento deveria ser Started, veio {other:?}"),
        }
    }

    #[test]
    fn progress_counter_advances_monotonically_to_the_total() {
        let conn = fresh_conn();
        let acc = seed_account(&conn);
        seed_tx(&conn, acc, "Pix - ENERGISA - 09.095.183/0001-40 - ITAU");
        seed_tx(&conn, acc, "Pix - DEMERGE - 33.967.103/0001-84 - BB");
        let fake = FakeProvider::new(&[
            ("09095183000140", "4711301"),
            ("33967103000184", "5611201"),
        ]);

        let (events, _db) = collect_events(conn, &fake, Some(acc), &AtomicBool::new(false));

        let dones: Vec<u32> = events
            .iter()
            .filter_map(|e| match e {
                EnrichEvent::Resolved { done, .. } => Some(*done),
                _ => None,
            })
            .collect();
        assert_eq!(dones, vec![1, 2], "o contador nunca anda para trás nem pula");
    }

    #[test]
    fn unmapped_activity_becomes_unresolved_and_creates_no_rule() {
        let conn = fresh_conn();
        let acc = seed_account(&conn);
        seed_tx(&conn, acc, "Pix - FAZENDA - 09.095.183/0001-40 - ITAU");
        let fake = FakeProvider::new(&[("09095183000140", "0111301")]); // fora do mapa

        let (events, db) = collect_events(conn, &fake, Some(acc), &AtomicBool::new(false));

        assert_eq!(kinds(&events), vec!["started", "unresolved", "finished"]);
        assert_eq!(
            rules_for(&db, "09.095.183/0001-40"),
            0,
            "sem categoria sugerida, nenhuma regra é criada"
        );
    }

    #[test]
    fn skips_tax_ids_that_already_have_a_rule() {
        let conn = fresh_conn();
        let acc = seed_account(&conn);
        seed_tx(&conn, acc, "Pix - ENERGISA - 09.095.183/0001-40 - ITAU");
        conn.execute(
            "INSERT INTO rules (category_id, priority, due_day, display_name)
             VALUES ((SELECT id FROM categories LIMIT 1), 10, NULL, 'já existe')",
            [],
        )
        .unwrap();
        let rule_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO rule_patterns (rule_id, pattern) VALUES (?1, '09.095.183/0001-40')",
            params![rule_id],
        )
        .unwrap();
        let fake = FakeProvider::new(&[("09095183000140", "4711301")]);

        let (events, _db) = collect_events(conn, &fake, Some(acc), &AtomicBool::new(false));

        assert_eq!(
            fake.call_count(),
            0,
            "a regra existente é o cache — não consulta"
        );
        assert_eq!(kinds(&events), vec!["started", "finished"]);
    }

    #[test]
    fn a_failed_lookup_is_counted_and_the_loop_carries_on() {
        let conn = fresh_conn();
        let acc = seed_account(&conn);
        seed_tx(&conn, acc, "Pix - ENERGISA - 09.095.183/0001-40 - ITAU");
        seed_tx(&conn, acc, "Pix - DEMERGE - 33.967.103/0001-84 - BB");
        // Só o segundo tem resposta; o primeiro (ordem alfabética) falha.
        let fake = FakeProvider::new(&[("33967103000184", "5611201")]);

        let (events, _db) = collect_events(conn, &fake, Some(acc), &AtomicBool::new(false));

        assert_eq!(
            kinds(&events),
            vec!["started", "failed", "resolved", "finished"],
            "a falha não interrompe o que vem depois"
        );
        match &events[1] {
            EnrichEvent::Failed { done, tax_id } => {
                assert_eq!(*done, 1);
                assert_eq!(tax_id, "09.095.183/0001-40");
            }
            other => panic!("esperava Failed, veio {other:?}"),
        }
    }

    #[test]
    fn cancelling_stops_early_and_creates_nothing() {
        let conn = fresh_conn();
        let acc = seed_account(&conn);
        seed_tx(&conn, acc, "Pix - ENERGISA - 09.095.183/0001-40 - ITAU");
        seed_tx(&conn, acc, "Pix - DEMERGE - 33.967.103/0001-84 - BB");
        let fake = FakeProvider::new(&[
            ("09095183000140", "4711301"),
            ("33967103000184", "5611201"),
        ]);
        // Já cancelado antes da primeira volta: nenhuma consulta deve sair.
        let cancel = AtomicBool::new(true);

        let (events, db) = collect_events(conn, &fake, Some(acc), &cancel);

        assert_eq!(kinds(&events), vec!["started", "cancelled"]);
        assert_eq!(fake.call_count(), 0, "cancelado antes de consultar");
        assert_eq!(rules_for(&db, "09.095.183/0001-40"), 0);
        assert_eq!(rules_for(&db, "33.967.103/0001-84"), 0);
    }

    #[test]
    fn cancelled_report_carries_the_partial_result() {
        let conn = fresh_conn();
        let acc = seed_account(&conn);
        seed_tx(&conn, acc, "Pix - ENERGISA - 09.095.183/0001-40 - ITAU");
        seed_tx(&conn, acc, "Pix - DEMERGE - 33.967.103/0001-84 - BB");
        let fake = FakeProvider::new(&[
            ("09095183000140", "4711301"),
            ("33967103000184", "5611201"),
        ]);
        let cancel = AtomicBool::new(false);

        // Cancela assim que o primeiro resolver: a segunda volta não acontece.
        let guarded = Mutex::new(conn);
        let mut events = Vec::new();
        run_enrichment(&guarded, &pack(), &fake, Some(acc), &cancel, &mut |e| {
            if matches!(e, EnrichEvent::Resolved { .. }) {
                cancel.store(true, Ordering::Relaxed);
            }
            events.push(e);
        })
        .unwrap();

        assert_eq!(kinds(&events), vec!["started", "resolved", "cancelled"]);
        match events.last().unwrap() {
            EnrichEvent::Cancelled { report } => {
                assert_eq!(
                    report.created_rules.len(),
                    1,
                    "a regra criada antes da parada permanece no relatório"
                );
            }
            other => panic!("esperava Cancelled, veio {other:?}"),
        }
        assert_eq!(
            rules_for(&guarded, "09.095.183/0001-40"),
            1,
            "a regra continua no banco depois do cancelamento"
        );
        assert_eq!(
            rules_for(&guarded, "33.967.103/0001-84"),
            0,
            "a que nunca foi consultada não existe"
        );
    }

    #[test]
    fn collects_unique_tax_ids_sorted_and_deduped() {
        let conn = fresh_conn();
        let acc = seed_account(&conn);
        seed_tx(&conn, acc, "Pix - ENERGISA - 09.095.183/0001-40 - ITAU");
        seed_tx(&conn, acc, "Pix - ENERGISA 2 - 09.095.183/0001-40 - ITAU");
        seed_tx(&conn, acc, "Pix - DEMERGE - 33.967.103/0001-84 - BB");
        seed_tx(&conn, acc, "Compra no débito - MISTER SUSHI");

        let ids = collect_unique_tax_ids(&conn, &pack(), Some(acc)).unwrap();

        assert_eq!(ids, vec!["09.095.183/0001-40", "33.967.103/0001-84"]);
    }

    #[test]
    fn collects_nothing_when_no_description_has_a_tax_id() {
        let conn = fresh_conn();
        let acc = seed_account(&conn);
        seed_tx(&conn, acc, "Compra no débito - MISTER SUSHI");

        assert!(collect_unique_tax_ids(&conn, &pack(), Some(acc))
            .unwrap()
            .is_empty());
    }

    #[test]
    fn ignores_transactions_from_another_account() {
        let conn = fresh_conn();
        let a = seed_account(&conn);
        let b = seed_account(&conn);
        seed_tx(&conn, b, "Pix - ENERGISA - 09.095.183/0001-40 - ITAU");

        assert!(collect_unique_tax_ids(&conn, &pack(), Some(a))
            .unwrap()
            .is_empty());
        assert_eq!(
            collect_unique_tax_ids(&conn, &pack(), None).unwrap().len(),
            1
        );
    }
}
