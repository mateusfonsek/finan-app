use rusqlite::params;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

use crate::db::Db;
use crate::error::{AppError, AppResult};
use crate::locale::{LocalePack, LocaleState};

/// Extract the first business tax id (e.g. a Brazilian CNPJ) found in a
/// description, using the active locale's `taxId.regex`. Returns None when the
/// locale defines no tax id.
pub fn extract_cnpj(pack: &LocalePack, description: &str) -> Option<String> {
    let re = pack.tax_id_re.as_ref()?;
    re.find(description).map(|m| m.as_str().to_string())
}

/// Lookup category id by the longest tax-classification-code prefix that
/// matches (e.g. Brazilian CNAE). The `cnae_map` maps prefixes to a stable
/// category **key**, resolved to an id via the `categories` table. Longest
/// prefix wins (e.g. "4711" beats "47"). Unknown code returns None.
pub fn cnae_to_category_id(
    conn: &rusqlite::Connection,
    cnae: &str,
    pack: &LocalePack,
) -> AppResult<Option<i64>> {
    let mut best: Option<&crate::locale::CnaeEntry> = None;
    for entry in &pack.rules.cnae_map {
        if cnae.starts_with(&entry.prefix)
            && best.map_or(true, |b| entry.prefix.len() > b.prefix.len())
        {
            best = Some(entry);
        }
    }
    let Some(entry) = best else {
        return Ok(None);
    };
    let id: Option<i64> = conn
        .query_row(
            "SELECT id FROM categories WHERE key = ?1",
            params![entry.category],
            |r| r.get(0),
        )
        .ok();
    Ok(id)
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CnpjResolution {
    pub cnpj: String,
    pub razao_social: Option<String>,
    pub nome_fantasia: Option<String>,
    pub cnae_fiscal: Option<String>,
    pub cnae_fiscal_descricao: Option<String>,
    pub suggested_category_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct BrasilApiResponse {
    razao_social: Option<String>,
    nome_fantasia: Option<String>,
    /// BrasilAPI returns this as a number; we coerce to String for stability.
    cnae_fiscal: Option<serde_json::Value>,
    cnae_fiscal_descricao: Option<String>,
}

fn cnpj_digits(cnpj: &str) -> String {
    cnpj.chars().filter(|c| c.is_ascii_digit()).collect()
}

fn cnae_value_to_string(v: &serde_json::Value) -> Option<String> {
    match v {
        serde_json::Value::Number(n) => Some(n.to_string()),
        serde_json::Value::String(s) => Some(s.clone()),
        _ => None,
    }
}

fn fetch_brasilapi(cnpj_digits_only: &str) -> AppResult<BrasilApiResponse> {
    let url = format!("https://brasilapi.com.br/api/cnpj/v1/{cnpj_digits_only}");
    let resp = ureq::get(&url)
        .timeout(std::time::Duration::from_secs(8))
        .call()
        .map_err(|e| AppError::Invalid(format!("BrasilAPI {cnpj_digits_only}: {e}")))?;
    resp.into_json::<BrasilApiResponse>()
        .map_err(|e| AppError::Invalid(format!("BrasilAPI parse: {e}")))
}

/// Shared engine used by `resolve_cnpj` and `auto_classify_with_cnpj`.
///
/// The online company lookup is gated on `taxId.provider`. Today only
/// `brasilapi` is implemented; any other provider returns a bare resolution
/// (no company name / no suggested category) so the app still works.
pub fn resolve_cnpj_with_conn(
    conn: &rusqlite::Connection,
    cnpj: &str,
    pack: &LocalePack,
) -> AppResult<CnpjResolution> {
    if pack.manifest.tax_id.provider != "brasilapi" {
        return Ok(CnpjResolution {
            cnpj: cnpj.to_string(),
            razao_social: None,
            nome_fantasia: None,
            cnae_fiscal: None,
            cnae_fiscal_descricao: None,
            suggested_category_id: None,
        });
    }

    let digits = cnpj_digits(cnpj);
    if digits.len() != 14 {
        return Err(AppError::Invalid(format!("CNPJ inválido: {cnpj}")));
    }
    let resp = fetch_brasilapi(&digits)?;
    let cnae = resp.cnae_fiscal.as_ref().and_then(cnae_value_to_string);
    let suggested = match cnae.as_deref() {
        Some(c) => cnae_to_category_id(conn, c, pack)?,
        None => None,
    };
    Ok(CnpjResolution {
        cnpj: cnpj.to_string(),
        razao_social: resp.razao_social,
        nome_fantasia: resp.nome_fantasia,
        cnae_fiscal: cnae,
        cnae_fiscal_descricao: resp.cnae_fiscal_descricao,
        suggested_category_id: suggested,
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
    use crate::locale::LocalePack;
    use rusqlite::Connection;

    fn fresh_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        migrations::apply(&conn).unwrap();
        conn
    }

    fn pack() -> LocalePack {
        LocalePack::embedded_pt_br()
    }

    #[test]
    fn extracts_canonical_cnpj() {
        let desc = "Transferência enviada pelo Pix - DEMERGE - 33.967.103/0001-84 - banco";
        assert_eq!(
            extract_cnpj(&pack(), desc),
            Some("33.967.103/0001-84".into())
        );
    }

    #[test]
    fn no_cnpj_returns_none() {
        assert_eq!(extract_cnpj(&pack(), "Compra no débito - MISTER SUSHI"), None);
    }

    #[test]
    fn cnpj_digits_strips_punctuation() {
        assert_eq!(cnpj_digits("33.967.103/0001-84"), "33967103000184");
    }

    #[test]
    fn cnae_lookup_finds_mercado_for_4711() {
        let conn = fresh_conn();
        let id = cnae_to_category_id(&conn, "4711301", &pack()).unwrap();
        assert!(id.is_some(), "4711 prefix should map to Mercado");
    }

    #[test]
    fn cnae_lookup_finds_restaurante_for_561() {
        let conn = fresh_conn();
        let id = cnae_to_category_id(&conn, "5611201", &pack()).unwrap();
        assert!(id.is_some());
    }

    #[test]
    fn cnae_lookup_unknown_returns_none() {
        let conn = fresh_conn();
        // 0111 is "Cultivo de cereais" — not in our map
        assert_eq!(cnae_to_category_id(&conn, "0111301", &pack()).unwrap(), None);
    }

    #[test]
    fn longer_prefix_beats_shorter_for_4711_over_47() {
        // Both "4711" and "47" match; we want "4711" (Mercado) over "47" (Compras).
        let conn = fresh_conn();
        let id = cnae_to_category_id(&conn, "4711301", &pack())
            .unwrap()
            .unwrap();
        let mercado: i64 = conn
            .query_row(
                "SELECT id FROM categories WHERE name = 'Mercado'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(id, mercado);
    }

    #[test]
    fn cnae_lookup_finds_compras_for_4789_amazon() {
        let conn = fresh_conn();
        let id = cnae_to_category_id(&conn, "4789099", &pack())
            .unwrap()
            .unwrap();
        let compras: i64 = conn
            .query_row(
                "SELECT id FROM categories WHERE name = 'Compras'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(id, compras, "Amazon's CNAE 4789 maps to Compras");
    }

    #[test]
    fn cnae_lookup_finds_education_for_8531() {
        let conn = fresh_conn();
        // 8531-7 = Educação superior - graduação
        let id = cnae_to_category_id(&conn, "8531700", &pack())
            .unwrap()
            .unwrap();
        let education: i64 = conn
            .query_row(
                "SELECT id FROM categories WHERE key = 'education'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(id, education);
    }

    #[test]
    fn cnae_lookup_finds_pets_for_veterinaria() {
        let conn = fresh_conn();
        // 7500-1 = Atividades veterinárias
        let id = cnae_to_category_id(&conn, "7500100", &pack())
            .unwrap()
            .unwrap();
        let pets: i64 = conn
            .query_row("SELECT id FROM categories WHERE key = 'pets'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(id, pets);
    }
}
