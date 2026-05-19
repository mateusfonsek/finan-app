use regex::Regex;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::sync::OnceLock;
use tauri::State;

use crate::db::Db;
use crate::error::{AppError, AppResult};

fn cnpj_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"\b\d{2}\.\d{3}\.\d{3}/\d{4}-\d{2}\b").unwrap())
}

/// Extract the first canonical CNPJ (NN.NNN.NNN/NNNN-NN) found in a description.
pub fn extract_cnpj(description: &str) -> Option<String> {
    cnpj_re().find(description).map(|m| m.as_str().to_string())
}

/// CNAE prefix → category name. Longest prefix wins (e.g. "4711" beats "47").
/// Categories must exist in the `categories` table (created by migrations or
/// by the user). Unknown CNAE returns None — these go to manual triage.
const CNAE_MAP: &[(&str, &str)] = &[
    // Alimentação fora
    ("561", "Restaurante"),
    ("562", "Restaurante"),
    // Supermercado / mercado
    ("4711", "Mercado"),
    ("4712", "Mercado"),
    ("4721", "Mercado"),
    // Combustíveis / mobilidade
    ("4731", "Transporte"),
    ("4921", "Transporte"),
    ("4922", "Transporte"),
    ("4923", "Transporte"),
    ("4929", "Transporte"),
    ("5310", "Transporte"),
    // Farmácia / saúde
    ("4771", "Saúde"),
    ("4772", "Saúde"),
    ("8610", "Saúde"),
    ("8630", "Saúde"),
    ("8650", "Saúde"),
    // Lazer
    ("5914", "Lazer"),
    ("9319", "Lazer"),
    ("9329", "Lazer"),
    ("9001", "Lazer"),
    // Casa / utilidades públicas
    ("3511", "Casa"),
    ("3513", "Casa"),
    ("3514", "Casa"),
    ("3600", "Casa"),
    // Assinatura / serviços recorrentes (telecom, streaming, SaaS)
    ("6110", "Assinatura"),
    ("6120", "Assinatura"),
    ("6130", "Assinatura"),
    ("6190", "Assinatura"),
    ("6201", "Assinatura"),
    ("6202", "Assinatura"),
    ("6203", "Assinatura"),
    ("6204", "Assinatura"),
    ("6209", "Assinatura"),
    // Comércio varejista / e-commerce → Compras
    ("4789", "Compras"),
    ("4791", "Compras"),
    ("4781", "Compras"),
    ("4751", "Compras"),
    ("4754", "Compras"),
    ("4759", "Compras"),
    ("4761", "Compras"),
    ("4763", "Compras"),
    // Fallback comércio varejista genérico (2 dígitos = última opção)
    ("47", "Compras"),
];

/// Lookup category id by longest CNAE prefix that matches.
pub fn cnae_to_category_id(
    conn: &rusqlite::Connection,
    cnae: &str,
) -> AppResult<Option<i64>> {
    let mut best: Option<&'static (&str, &str)> = None;
    for entry in CNAE_MAP {
        if cnae.starts_with(entry.0) && best.map_or(true, |b| entry.0.len() > b.0.len()) {
            best = Some(entry);
        }
    }
    let Some((_, name)) = best else {
        return Ok(None);
    };
    let id: Option<i64> = conn
        .query_row(
            "SELECT id FROM categories WHERE name = ?1",
            params![name],
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
pub fn resolve_cnpj_with_conn(
    conn: &rusqlite::Connection,
    cnpj: &str,
) -> AppResult<CnpjResolution> {
    let digits = cnpj_digits(cnpj);
    if digits.len() != 14 {
        return Err(AppError::Invalid(format!("CNPJ inválido: {cnpj}")));
    }
    let resp = fetch_brasilapi(&digits)?;
    let cnae = resp.cnae_fiscal.as_ref().and_then(cnae_value_to_string);
    let suggested = match cnae.as_deref() {
        Some(c) => cnae_to_category_id(conn, c)?,
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
pub fn resolve_cnpj(db: State<'_, Db>, cnpj: String) -> AppResult<CnpjResolution> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    resolve_cnpj_with_conn(&conn, &cnpj)
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

    #[test]
    fn extracts_canonical_cnpj() {
        let desc = "Transferência enviada pelo Pix - DEMERGE - 33.967.103/0001-84 - banco";
        assert_eq!(extract_cnpj(desc), Some("33.967.103/0001-84".into()));
    }

    #[test]
    fn no_cnpj_returns_none() {
        assert_eq!(extract_cnpj("Compra no débito - MISTER SUSHI"), None);
    }

    #[test]
    fn cnpj_digits_strips_punctuation() {
        assert_eq!(cnpj_digits("33.967.103/0001-84"), "33967103000184");
    }

    #[test]
    fn cnae_lookup_finds_mercado_for_4711() {
        let conn = fresh_conn();
        let id = cnae_to_category_id(&conn, "4711301").unwrap();
        assert!(id.is_some(), "4711 prefix should map to Mercado");
    }

    #[test]
    fn cnae_lookup_finds_restaurante_for_561() {
        let conn = fresh_conn();
        let id = cnae_to_category_id(&conn, "5611201").unwrap();
        assert!(id.is_some());
    }

    #[test]
    fn cnae_lookup_unknown_returns_none() {
        let conn = fresh_conn();
        // 0111 is "Cultivo de cereais" — not in our map
        assert_eq!(cnae_to_category_id(&conn, "0111301").unwrap(), None);
    }

    #[test]
    fn longer_prefix_beats_shorter_for_4711_over_47() {
        // Both "4711" and "47" match; we want "4711" (Mercado) over "47" (Compras).
        let conn = fresh_conn();
        let id = cnae_to_category_id(&conn, "4711301").unwrap().unwrap();
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
        let id = cnae_to_category_id(&conn, "4789099").unwrap().unwrap();
        let compras: i64 = conn
            .query_row(
                "SELECT id FROM categories WHERE name = 'Compras'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(id, compras, "Amazon's CNAE 4789 maps to Compras");
    }
}
