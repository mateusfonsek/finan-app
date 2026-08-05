//! Tauri surface for tax-id enrichment. All logic lives in [`crate::enrich`];
//! this only adapts to the shape the frontend already consumes.

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

use crate::db::Db;
use crate::enrich;
use crate::error::AppResult;
use crate::locale::{LocalePack, LocaleState};

pub use enrich::extract_tax_id as extract_cnpj;

/// Brazilian field names on purpose: this is the UI contract, not the core.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CnpjResolution {
    pub cnpj: String,
    pub razao_social: Option<String>,
    pub nome_fantasia: Option<String>,
    pub cnae_fiscal: Option<String>,
    pub cnae_fiscal_descricao: Option<String>,
    pub suggested_category_id: Option<i64>,
}

impl CnpjResolution {
    /// Tax id seen, nothing found — what a locale without a provider returns.
    fn bare(cnpj: &str) -> Self {
        Self {
            cnpj: cnpj.to_string(),
            razao_social: None,
            nome_fantasia: None,
            cnae_fiscal: None,
            cnae_fiscal_descricao: None,
            suggested_category_id: None,
        }
    }
}

/// Does not check the enabled flag — callers decide. `resolve_cnpj` is a
/// direct user action; the import path gates before reaching here.
pub fn resolve_cnpj_with_conn(
    conn: &rusqlite::Connection,
    cnpj: &str,
    pack: &LocalePack,
) -> AppResult<CnpjResolution> {
    let Some(e) = enrich::lookup(conn, cnpj, pack)? else {
        return Ok(CnpjResolution::bare(cnpj));
    };
    Ok(CnpjResolution {
        cnpj: cnpj.to_string(),
        razao_social: e.company.legal_name,
        nome_fantasia: e.company.trade_name,
        cnae_fiscal: e.company.activity_code,
        cnae_fiscal_descricao: e.company.activity_label,
        suggested_category_id: e.suggested_category_id,
    })
}

#[tauri::command]
#[specta::specta]
pub fn resolve_cnpj(
    db: State<'_, Db>,
    locale: State<'_, LocaleState>,
    cnpj: String,
) -> AppResult<CnpjResolution> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    let pack = locale.pack.lock().expect("locale mutex poisoned");
    resolve_cnpj_with_conn(&conn, &cnpj, &pack)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;
    use crate::enrich::category_for_activity;
    use rusqlite::Connection;

    fn fresh_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        migrations::apply(&conn).unwrap();
        conn
    }

    fn pack() -> LocalePack {
        LocalePack::embedded_pt_br()
    }

    fn category_named(conn: &Connection, name: &str) -> i64 {
        conn.query_row(
            "SELECT id FROM categories WHERE name = ?1",
            rusqlite::params![name],
            |r| r.get(0),
        )
        .unwrap()
    }

    fn category_keyed(conn: &Connection, key: &str) -> i64 {
        conn.query_row(
            "SELECT id FROM categories WHERE key = ?1",
            rusqlite::params![key],
            |r| r.get(0),
        )
        .unwrap()
    }

    /// Each CNAE mapping categorizes without asking, so pin the main ones.
    #[test]
    fn cnae_map_routes_known_activities() {
        let conn = fresh_conn();
        let p = pack();
        let cases = [
            ("4711301", category_named(&conn, "Mercado")),
            ("5611201", category_named(&conn, "Restaurante")),
            ("4789099", category_named(&conn, "Compras")),
            ("8531700", category_keyed(&conn, "education")),
            ("7500100", category_keyed(&conn, "pets")),
        ];
        for (cnae, expected) in cases {
            assert_eq!(
                category_for_activity(&conn, cnae, &p).unwrap(),
                Some(expected),
                "CNAE {cnae}"
            );
        }
    }

    #[test]
    fn cnae_map_ignores_unmapped_activity() {
        let conn = fresh_conn();
        // 0111 = grain farming, not in the map.
        assert_eq!(
            category_for_activity(&conn, "0111301", &pack()).unwrap(),
            None
        );
    }

    /// No provider yields an empty resolution, not an error.
    #[test]
    fn no_provider_yields_bare_resolution() {
        let conn = fresh_conn();
        let mut p = pack();
        p.manifest.tax_id.provider = String::new();
        let r = resolve_cnpj_with_conn(&conn, "33.967.103/0001-84", &p).unwrap();
        assert_eq!(r.cnpj, "33.967.103/0001-84");
        assert!(r.razao_social.is_none());
        assert!(r.suggested_category_id.is_none());
    }
}
