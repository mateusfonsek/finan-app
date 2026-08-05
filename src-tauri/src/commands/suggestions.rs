use rusqlite::params;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::{HashMap, HashSet};
use std::thread;
use std::time::Duration;
use tauri::State;

use crate::commands::cnpj::{extract_cnpj, resolve_cnpj_with_conn, CnpjResolution};
use crate::commands::rules::apply_rules_internal;
use crate::db::Db;
use crate::domain::rule::Rule;
use crate::error::AppResult;
use crate::locale::{LocalePack, LocaleState};

/// Reduz uma descrição livre a (chave de agrupamento, label legível, padrão sugerido),
/// usando as regras de normalização do locale ativo (`rules.normalization`).
///
/// A chave precisa ser estável entre repetições da mesma contraparte.
/// O padrão é o que vai virar `rules.pattern` (LIKE substring).
///
/// Pública pra ser reusada por `income_sources` (mesmo modelo de agrupamento
/// por contraparte, mas pra entradas em vez de regras).
pub fn normalize(description: &str, pack: &LocalePack) -> (String, String, String) {
    let norm = &pack.rules.normalization;
    let sep = if norm.field_separator.is_empty() {
        " - "
    } else {
        norm.field_separator.as_str()
    };

    // 1. Tax id (CNPJ) tem precedência: label = texto depois do 1º separador.
    if let Some(cnpj) = extract_cnpj(pack, description) {
        let label = description
            .split_once(sep)
            .map(|(_, after)| after.trim())
            .filter(|s| !s.is_empty())
            .map(String::from)
            .unwrap_or_else(|| description.to_string());
        let kp = if norm.cnpj_key_prefix.is_empty() {
            "cnpj"
        } else {
            norm.cnpj_key_prefix.as_str()
        };
        return (format!("{kp}:{cnpj}"), label, cnpj);
    }

    // 2. Regras ordenadas do pack (strip / masked / system).
    for rule in &norm.rules {
        match rule.kind.as_str() {
            "strip" => {
                if let Some(rest) = description.strip_prefix(&rule.prefix) {
                    let v = rest.trim();
                    return (
                        format!("{}:{v}", rule.key_prefix),
                        rule.label.replace("{v}", v),
                        v.to_string(),
                    );
                }
            }
            "masked" => {
                if description.starts_with(&rule.prefix) {
                    if let Some(re) = pack.cpf_mask_re.as_ref() {
                        if let Some(mask) = re.find(description) {
                            let mask_s = mask.as_str();
                            let name =
                                pix_name(description, sep).unwrap_or_else(|| mask_s.to_string());
                            return (
                                format!("{}:{mask_s}", rule.key_prefix),
                                rule.label.replace("{name}", &name),
                                mask_s.to_string(),
                            );
                        }
                    }
                }
            }
            "system" => {
                if description.starts_with(&rule.prefix) {
                    return (rule.key.clone(), rule.label.clone(), rule.prefix.clone());
                }
            }
            _ => {}
        }
    }

    // 3. Fallback: descrição inteira (cada tx vira seu próprio grupo).
    (
        format!("raw:{description}"),
        description.to_string(),
        description.to_string(),
    )
}

/// Extrai o nome da contraparte de descrições Pix do Nubank.
/// Formato: `Transferência (enviada|recebida) pelo Pix - NOME - CPF_MASK - BANCO ...`
fn pix_name(description: &str, sep: &str) -> Option<String> {
    let after_prefix = description.splitn(2, sep).nth(1)?.splitn(2, sep).next()?;
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
pub fn suggest_rules(
    db: State<'_, Db>,
    locale: State<'_, LocaleState>,
    min_count: u32,
) -> AppResult<Vec<RuleSuggestion>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    let pack = locale.pack.lock().expect("locale mutex poisoned");
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
        let (key, label, pattern) = normalize(&desc, &pack);
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
pub fn suggest_pattern_for(locale: State<'_, LocaleState>, description: String) -> String {
    let pack = locale.pack.lock().expect("locale mutex poisoned");
    normalize(&description, &pack).2
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
    locale: State<'_, LocaleState>,
    account_id: Option<i64>,
) -> AppResult<AutoClassifyReport> {
    let pack = locale.pack.lock().expect("locale mutex poisoned");

    // Gate lives here, not in callers: any future caller inherits it. Off (or
    // a locale without a provider) returns an empty report and touches no network.
    {
        let conn = db.conn.lock().expect("db mutex poisoned");
        if !crate::enrich::is_active(&conn, &pack) {
            return Ok(AutoClassifyReport {
                created_rules: Vec::new(),
                txs_classified: 0,
                unresolved: Vec::new(),
            });
        }
    }

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
            if let Some(c) = extract_cnpj(&pack, &d) {
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
                "SELECT 1 FROM rule_patterns WHERE pattern = ?1 LIMIT 1",
                params![cnpj],
                |_| Ok(()),
            )
            .is_ok()
        };
        if has_rule {
            continue;
        }

        // Delay comes from the provider, not from this file.
        if idx > 0 {
            thread::sleep(Duration::from_millis(crate::enrich::courtesy_delay_ms(&pack)));
        }

        let resolution = {
            let conn = db.conn.lock().expect("db mutex poisoned");
            match resolve_cnpj_with_conn(&conn, cnpj, &pack) {
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
                    "INSERT INTO rules (category_id, priority, due_day, display_name)
                     VALUES (?1, 10, NULL, ?2)",
                    params![cat_id, display],
                )?;
                let id = conn.last_insert_rowid();
                // A regra nasce com o CNPJ como único trecho.
                conn.execute(
                    "INSERT INTO rule_patterns (rule_id, pattern) VALUES (?1, ?2)",
                    params![id, cnpj],
                )?;
                let rule = conn.query_row(
                    "SELECT id, category_id, priority, due_day, display_name, created_at
                     FROM rules WHERE id = ?1",
                    params![id],
                    |row| {
                        Ok(Rule {
                            id: row.get(0)?,
                            patterns: vec![cnpj.clone()],
                            category_id: row.get(1)?,
                            priority: row.get(2)?,
                            due_day: row.get(3)?,
                            display_name: row.get(4)?,
                            created_at: row.get(5)?,
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
    use crate::locale::LocalePack;

    fn pack() -> LocalePack {
        LocalePack::embedded_pt_br()
    }

    #[test]
    fn normalize_debito_groups_by_merchant() {
        let p = pack();
        let (k1, _, p1) = normalize("Compra no débito - MISTER SUSHI", &p);
        let (k2, _, _p2) = normalize("Compra no débito - MISTER SUSHI", &p);
        assert_eq!(k1, k2);
        assert_eq!(p1, "MISTER SUSHI");
    }

    #[test]
    fn normalize_pix_out_groups_by_cpf_mask() {
        let p = pack();
        let a = "Transferência enviada pelo Pix - Mateus Fonseca (CAIXA) - •••.982.424-•• - CAIXA";
        let b = "Transferência enviada pelo Pix - Mateus Fonseca (Caixa) - •••.982.424-•• - CAIXA outro";
        let (k1, _, p1) = normalize(a, &p);
        let (k2, _, _) = normalize(b, &p);
        assert_eq!(k1, k2, "same CPF mask = same key, even with name variation");
        assert_eq!(p1, "•••.982.424-••");
    }

    #[test]
    fn normalize_cnpj_wins_over_pix_prefix() {
        let d = "Transferência enviada pelo Pix - ENERGISA - 09.095.183/0001-40 - ITAU";
        let (k, _, p) = normalize(d, &pack());
        assert!(k.starts_with("cnpj:"), "CNPJ should take precedence");
        assert_eq!(p, "09.095.183/0001-40");
    }

    #[test]
    fn normalize_aplicacao_rdb_is_system_key() {
        let (k, _, p) = normalize("Aplicação RDB", &pack());
        assert_eq!(k, "system:rdb_apl");
        assert_eq!(p, "Aplicação RDB");
    }

    #[test]
    fn normalize_fallback_uses_full_description() {
        let (k, _, _) = normalize("Algo bem estranho aqui", &pack());
        assert!(k.starts_with("raw:"));
    }
}
