//! Orquestração do enriquecimento por tax id.
//!
//! Separado dos comandos de propósito: aqui não há `State`, `AppHandle` nem
//! `Channel`. Só a lógica — que é o que torna possível testar ordem de eventos,
//! resiliência a falha e cancelamento sem tocar a rede nem subir um app Tauri.

use std::collections::HashSet;

use rusqlite::params;
use serde::Serialize;
use specta::Type;

use crate::commands::cnpj::CnpjResolution;
use crate::commands::suggestions::AutoClassifyReport;
use crate::domain::rule::Rule;
use crate::enrich::extract_tax_id;
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
