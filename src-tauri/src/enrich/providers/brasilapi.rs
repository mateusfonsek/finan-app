//! BrasilAPI — public CNPJ lookup.
//!
//! The only file that knows this service's URL, response shape and quirks.

use serde::Deserialize;

use crate::enrich::provider::{Company, TaxIdProvider};
use crate::error::{AppError, AppResult};

const BASE_URL: &str = "https://brasilapi.com.br/api/cnpj/v1";
const TIMEOUT_SECS: u64 = 8;

const CNPJ_DIGITS: usize = 14;

pub struct BrasilApi;

#[derive(Debug, Deserialize)]
struct Response {
    razao_social: Option<String>,
    nome_fantasia: Option<String>,
    /// Comes back as a number, not a string — coerced below.
    cnae_fiscal: Option<serde_json::Value>,
    cnae_fiscal_descricao: Option<String>,
}

fn cnae_to_string(v: &serde_json::Value) -> Option<String> {
    match v {
        serde_json::Value::Number(n) => Some(n.to_string()),
        serde_json::Value::String(s) => Some(s.clone()),
        _ => None,
    }
}

impl TaxIdProvider for BrasilApi {
    fn lookup(&self, tax_id_digits: &str) -> AppResult<Company> {
        if tax_id_digits.len() != CNPJ_DIGITS {
            return Err(AppError::Invalid(format!(
                "invalid CNPJ: {tax_id_digits}"
            )));
        }
        let url = format!("{BASE_URL}/{tax_id_digits}");
        let resp = ureq::get(&url)
            .timeout(std::time::Duration::from_secs(TIMEOUT_SECS))
            .call()
            .map_err(|e| AppError::Invalid(format!("BrasilAPI {tax_id_digits}: {e}")))?
            .into_json::<Response>()
            .map_err(|e| AppError::Invalid(format!("BrasilAPI parse: {e}")))?;

        Ok(Company {
            legal_name: resp.razao_social,
            trade_name: resp.nome_fantasia,
            activity_code: resp.cnae_fiscal.as_ref().and_then(cnae_to_string),
            activity_label: resp.cnae_fiscal_descricao,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coerces_numeric_cnae_to_string() {
        let v = serde_json::json!(4711302);
        assert_eq!(cnae_to_string(&v), Some("4711302".to_string()));
    }

    #[test]
    fn keeps_string_cnae() {
        let v = serde_json::json!("4711302");
        assert_eq!(cnae_to_string(&v), Some("4711302".to_string()));
    }

    #[test]
    fn rejects_wrong_digit_count_without_network() {
        assert!(BrasilApi.lookup("123").is_err());
    }
}
