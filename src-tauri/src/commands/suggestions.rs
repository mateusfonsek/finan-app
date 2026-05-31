use regex::Regex;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;
use std::thread;
use std::time::Duration;
use tauri::State;

use crate::commands::cnpj::{extract_cnpj, resolve_cnpj_with_conn, CnpjResolution};
use crate::commands::rules::apply_rules_internal;
use crate::db::Db;
use crate::domain::rule::Rule;
use crate::error::AppResult;

/// CPF parcial como aparece em descrições OFX do Nubank: `•••.NNN.NNN-••`.
fn cpf_mask_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"•••\.\d{3}\.\d{3}-••").unwrap())
}

/// Reduz uma descrição livre a (chave de agrupamento, label legível, padrão sugerido).
///
/// A chave precisa ser estável entre repetições da mesma contraparte.
/// O padrão é o que vai virar `rules.pattern` (LIKE substring).
///
/// Pública pra ser reusada por `income_sources` (mesmo modelo de agrupamento
/// por contraparte, mas pra entradas em vez de regras).
pub fn normalize(description: &str) -> (String, String, String) {
    if let Some(cnpj) = extract_cnpj(description) {
        // Label = tudo depois do primeiro " - " (mesma regra do RecentList no
        // dashboard). Mostra o nome da contraparte em vez de "CNPJ XX.XXX...".
        // Truncate na UI corta visualmente; o tooltip preserva o texto completo.
        let label = description
            .split_once(" - ")
            .map(|(_, after)| after.trim())
            .filter(|s| !s.is_empty())
            .map(String::from)
            .unwrap_or_else(|| description.to_string());
        return (format!("cnpj:{cnpj}"), label, cnpj);
    }
    if let Some(merchant) = description.strip_prefix("Compra no débito - ") {
        let m = merchant.trim();
        return (
            format!("debito:{m}"),
            format!("Débito · {m}"),
            m.to_string(),
        );
    }
    if description.starts_with("Transferência enviada pelo Pix") {
        if let Some(mask) = cpf_mask_re().find(description) {
            let name = pix_name(description).unwrap_or_else(|| mask.as_str().to_string());
            return (
                format!("pix_out:{}", mask.as_str()),
                format!("Pix → {name}"),
                mask.as_str().to_string(),
            );
        }
    }
    if description.starts_with("Transferência recebida pelo Pix") {
        if let Some(mask) = cpf_mask_re().find(description) {
            let name = pix_name(description).unwrap_or_else(|| mask.as_str().to_string());
            return (
                format!("pix_in:{}", mask.as_str()),
                format!("Pix ← {name}"),
                mask.as_str().to_string(),
            );
        }
    }
    if description.starts_with("Transferência Recebida - ") {
        if let Some(mask) = cpf_mask_re().find(description) {
            let name = pix_name(description).unwrap_or_else(|| mask.as_str().to_string());
            return (
                format!("ted_in:{}", mask.as_str()),
                format!("Transf. ← {name}"),
                mask.as_str().to_string(),
            );
        }
    }
    if let Some(rest) = description.strip_prefix("Pagamento de boleto efetuado - ") {
        let r = rest.trim();
        return (
            format!("boleto:{r}"),
            format!("Boleto · {r}"),
            r.to_string(),
        );
    }
    // Padrões de transferência interna — chave única, padrão direto.
    for (prefix, key, label) in &[
        ("Aplicação RDB", "system:rdb_apl", "Aplicação RDB"),
        ("Resgate RDB", "system:rdb_res", "Resgate RDB"),
        ("Pagamento de fatura", "system:fatura", "Pagamento de fatura"),
        ("Estorno -", "system:estorno", "Estorno"),
        ("Recarga de celular", "system:recarga", "Recarga de celular"),
    ] {
        if description.starts_with(prefix) {
            return ((*key).into(), (*label).into(), (*prefix).into());
        }
    }
    // Fallback: descrição inteira (cada tx vira seu próprio grupo).
    (
        format!("raw:{description}"),
        description.to_string(),
        description.to_string(),
    )
}

/// Extrai o nome da contraparte de descrições Pix do Nubank.
/// Formato: `Transferência (enviada|recebida) pelo Pix - NOME - CPF_MASK - BANCO ...`
fn pix_name(description: &str) -> Option<String> {
    let after_prefix = description
        .splitn(2, " - ")
        .nth(1)?
        .splitn(2, " - ")
        .next()?;
    Some(after_prefix.trim().to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RuleSuggestion {
    pub key: String,
    pub label: String,
    pub suggested_pattern: String,
    pub count: u32,
    pub total: String,
    pub sample_description: String,
    pub transaction_ids: Vec<i64>,
}

/// Returns groups of uncategorized OUTFLOW transactions whose normalized key
/// repeats at least `min_count` times. Sorted by absolute total descending.
///
/// **Importante**: só considera tx com `amount < 0` (saídas). Entradas não
/// precisam de categoria — renda é rastreada por contraparte no painel
/// "Fontes de Renda" do Dashboard.
#[tauri::command]
#[specta::specta]
pub fn suggest_rules(db: State<'_, Db>, min_count: u32) -> AppResult<Vec<RuleSuggestion>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    let mut stmt = conn.prepare(
        "SELECT id, description, amount FROM transactions
         WHERE category_id IS NULL
           AND CAST(amount AS REAL) < 0
         ORDER BY id ASC",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    struct Bucket {
        label: String,
        pattern: String,
        ids: Vec<i64>,
        total: rust_decimal::Decimal,
        sample: String,
    }
    let mut buckets: HashMap<String, Bucket> = HashMap::new();
    for (id, desc, amt) in rows {
        let (key, label, pattern) = normalize(&desc);
        let d: rust_decimal::Decimal = amt.parse().unwrap_or(rust_decimal::Decimal::ZERO);
        let entry = buckets.entry(key).or_insert_with(|| Bucket {
            label: label.clone(),
            pattern: pattern.clone(),
            ids: vec![],
            total: rust_decimal::Decimal::ZERO,
            sample: desc.clone(),
        });
        entry.ids.push(id);
        entry.total += d;
    }
    let min = min_count.max(1) as usize;
    let mut out: Vec<RuleSuggestion> = buckets
        .into_iter()
        .filter(|(_, b)| b.ids.len() >= min)
        .map(|(key, b)| RuleSuggestion {
            key,
            label: b.label,
            suggested_pattern: b.pattern,
            count: b.ids.len() as u32,
            total: b.total.to_string(),
            sample_description: b.sample,
            transaction_ids: b.ids,
        })
        .collect();
    out.sort_by(|a, b| {
        let ad: rust_decimal::Decimal = a.total.parse().unwrap_or(rust_decimal::Decimal::ZERO);
        let bd: rust_decimal::Decimal = b.total.parse().unwrap_or(rust_decimal::Decimal::ZERO);
        bd.abs().cmp(&ad.abs())
    });
    Ok(out)
}

/// Pattern sugerido (LIKE substring) pra uma única descrição — mesma lógica
/// da aba de Sugestões, exposta pra criar uma regra direto do painel de detalhe
/// de uma transação.
#[tauri::command]
#[specta::specta]
pub fn suggest_pattern_for(description: String) -> String {
    normalize(&description).2
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AutoClassifyReport {
    pub created_rules: Vec<Rule>,
    pub txs_classified: u32,
    pub unresolved: Vec<CnpjResolution>,
}

/// For every uncategorized OUTFLOW transaction with a CNPJ in its description:
/// 1. Skip if a rule already exists with that CNPJ as pattern.
/// 2. Else call BrasilAPI once.
/// 3. If CNAE maps to a category → create rule (priority 10) and apply.
/// 4. Else collect into `unresolved` for the UI to handle manually.
///
/// **Importante**: só considera tx com `amount < 0` (saídas). Categorizar entradas
/// (salário/freela vindos de um CNPJ) não faz sentido — entradas são rastreadas
/// por contraparte no painel "Fontes de Renda", não por categoria.
#[tauri::command]
#[specta::specta]
pub fn auto_classify_with_cnpj(
    db: State<'_, Db>,
    account_id: Option<i64>,
) -> AppResult<AutoClassifyReport> {
    let unique_cnpjs: Vec<String> = {
        let conn = db.conn.lock().expect("db mutex poisoned");
        let (sql, has_account) = match account_id {
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
        let descs: Vec<String> = if has_account {
            stmt.query_map(params![account_id.unwrap()], |r| r.get::<_, String>(0))?
                .collect::<rusqlite::Result<Vec<_>>>()?
        } else {
            stmt.query_map([], |r| r.get::<_, String>(0))?
                .collect::<rusqlite::Result<Vec<_>>>()?
        };
        let mut seen: HashSet<String> = HashSet::new();
        for d in descs {
            if let Some(c) = extract_cnpj(&d) {
                seen.insert(c);
            }
        }
        seen.into_iter().collect()
    };

    let mut created_rules: Vec<Rule> = Vec::new();
    let mut unresolved: Vec<CnpjResolution> = Vec::new();

    for (idx, cnpj) in unique_cnpjs.iter().enumerate() {
        // Skip CNPJs that already have a rule (the rule is our cache).
        let has_rule: bool = {
            let conn = db.conn.lock().expect("db mutex poisoned");
            conn.query_row(
                "SELECT 1 FROM rules WHERE pattern = ?1 LIMIT 1",
                params![cnpj],
                |_| Ok(()),
            )
            .is_ok()
        };
        if has_rule {
            continue;
        }

        // Be polite to BrasilAPI: ~250ms between calls.
        if idx > 0 {
            thread::sleep(Duration::from_millis(250));
        }

        let resolution = {
            let conn = db.conn.lock().expect("db mutex poisoned");
            match resolve_cnpj_with_conn(&conn, cnpj) {
                Ok(r) => r,
                Err(_) => continue, // network/parse error — try later
            }
        };

        match resolution.suggested_category_id {
            Some(cat_id) => {
                let display = resolution
                    .razao_social
                    .clone()
                    .or_else(|| resolution.nome_fantasia.clone());
                let conn = db.conn.lock().expect("db mutex poisoned");
                conn.execute(
                    "INSERT INTO rules (pattern, category_id, priority, due_day, display_name)
                     VALUES (?1, ?2, 10, NULL, ?3)",
                    params![cnpj, cat_id, display],
                )?;
                let id = conn.last_insert_rowid();
                let rule = conn.query_row(
                    "SELECT id, pattern, category_id, priority, due_day, display_name, created_at
                     FROM rules WHERE id = ?1",
                    params![id],
                    |row| {
                        Ok(Rule {
                            id: row.get(0)?,
                            pattern: row.get(1)?,
                            category_id: row.get(2)?,
                            priority: row.get(3)?,
                            due_day: row.get(4)?,
                            display_name: row.get(5)?,
                            created_at: row.get(6)?,
                        })
                    },
                )?;
                created_rules.push(rule);
            }
            None => unresolved.push(resolution),
        }
    }

    let txs_classified = {
        let mut conn = db.conn.lock().expect("db mutex poisoned");
        apply_rules_internal(&mut conn, account_id)?
    };

    Ok(AutoClassifyReport {
        created_rules,
        txs_classified,
        unresolved,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_debito_groups_by_merchant() {
        let (k1, _, p1) = normalize("Compra no débito - MISTER SUSHI");
        let (k2, _, p2) = normalize("Compra no débito - MISTER SUSHI");
        assert_eq!(k1, k2);
        assert_eq!(p1, "MISTER SUSHI");
    }

    #[test]
    fn normalize_pix_out_groups_by_cpf_mask() {
        let a = "Transferência enviada pelo Pix - Mateus Fonseca (CAIXA) - •••.982.424-•• - CAIXA";
        let b = "Transferência enviada pelo Pix - Mateus Fonseca (Caixa) - •••.982.424-•• - CAIXA outro";
        let (k1, _, p1) = normalize(a);
        let (k2, _, _) = normalize(b);
        assert_eq!(k1, k2, "same CPF mask = same key, even with name variation");
        assert_eq!(p1, "•••.982.424-••");
    }

    #[test]
    fn normalize_cnpj_wins_over_pix_prefix() {
        let d = "Transferência enviada pelo Pix - ENERGISA - 09.095.183/0001-40 - ITAU";
        let (k, _, p) = normalize(d);
        assert!(k.starts_with("cnpj:"), "CNPJ should take precedence");
        assert_eq!(p, "09.095.183/0001-40");
    }

    #[test]
    fn normalize_aplicacao_rdb_is_system_key() {
        let (k, _, p) = normalize("Aplicação RDB");
        assert_eq!(k, "system:rdb_apl");
        assert_eq!(p, "Aplicação RDB");
    }

    #[test]
    fn normalize_fallback_uses_full_description() {
        let (k, _, _) = normalize("Algo bem estranho aqui");
        assert!(k.starts_with("raw:"));
    }
}
