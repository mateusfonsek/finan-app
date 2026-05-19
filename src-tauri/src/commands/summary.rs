use rusqlite::params;
use rust_decimal::Decimal;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use tauri::State;

use crate::commands::suggestions::normalize;
use crate::db::Db;
use crate::domain::summary::{
    CategorySpend, IncomeSource, InvestmentSummary, KpiSummary, MonthSummary, TransferSummary,
};
use crate::error::{AppError, AppResult};

#[tauri::command]
#[specta::specta]
pub fn summary_kpis(db: State<'_, Db>, month: Option<String>) -> AppResult<KpiSummary> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    let pattern: Option<String> = month.as_ref().map(|m| format!("{m}-%"));

    // Excluímos tx cujo category.kind = 'transfer' (movimentações internas como
    // pagamento de fatura ou aplicação em poupança). Categoria NULL conta normal.
    let mut stmt = conn.prepare(
        "SELECT t.amount
         FROM transactions t
         LEFT JOIN categories c ON c.id = t.category_id
         WHERE (?1 IS NULL OR t.date LIKE ?1)
           AND COALESCE(c.kind, '') != 'transfer'",
    )?;
    let pat_ref: Option<&str> = pattern.as_deref();
    let rows: Vec<String> = stmt
        .query_map(params![pat_ref], |row| row.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let mut income = Decimal::ZERO;
    let mut expense = Decimal::ZERO;
    let count = rows.len() as u32;
    for s in &rows {
        let d = Decimal::from_str(s)
            .map_err(|e| AppError::Invalid(format!("bad amount '{s}': {e}")))?;
        if d.is_sign_negative() {
            expense += -d;
        } else {
            income += d;
        }
    }
    let net = income - expense;

    Ok(KpiSummary {
        income: income.to_string(),
        expense: expense.to_string(),
        net: net.to_string(),
        transaction_count: count,
    })
}

#[tauri::command]
#[specta::specta]
pub fn summary_by_category(
    db: State<'_, Db>,
    month: Option<String>,
) -> AppResult<Vec<CategorySpend>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    let pattern = month.as_ref().map(|m| format!("{m}-%"));

    let mut stmt = conn.prepare(
        "SELECT t.amount, t.category_id, c.name, c.color_token
         FROM transactions t
         LEFT JOIN categories c ON c.id = t.category_id
         WHERE (?1 IS NULL OR t.date LIKE ?1)
           AND COALESCE(c.kind, '') != 'transfer'",
    )?;
    type CategoryRow = (String, Option<i64>, Option<String>, Option<String>);
    let pat_for_query: Option<&str> = pattern.as_deref();
    let rows: Vec<CategoryRow> = stmt
        .query_map(params![pat_for_query], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let mut by: HashMap<Option<i64>, (String, Option<String>, Decimal)> = HashMap::new();
    let mut total_expense = Decimal::ZERO;

    for (amt, cat_id, name, color) in rows {
        let d =
            Decimal::from_str(&amt).map_err(|e| AppError::Invalid(format!("bad amount: {e}")))?;
        if !d.is_sign_negative() {
            continue;
        }
        let abs = -d;
        total_expense += abs;
        let display_name = name.unwrap_or_else(|| "Sem categoria".to_string());
        let entry = by
            .entry(cat_id)
            .or_insert((display_name, color, Decimal::ZERO));
        entry.2 += abs;
    }

    let mut out: Vec<CategorySpend> = by
        .into_iter()
        .map(|(cat_id, (name, color, total))| {
            let percent = if total_expense.is_zero() {
                0.0
            } else {
                let p: f64 = (total / total_expense).to_string().parse().unwrap_or(0.0);
                p * 100.0
            };
            CategorySpend {
                category_id: cat_id,
                name,
                color_token: color,
                total: total.to_string(),
                percent,
            }
        })
        .collect();
    out.sort_by(|a, b| {
        let ad: Decimal = Decimal::from_str(&a.total).unwrap_or(Decimal::ZERO);
        let bd: Decimal = Decimal::from_str(&b.total).unwrap_or(Decimal::ZERO);
        bd.cmp(&ad)
    });

    Ok(out)
}

#[tauri::command]
#[specta::specta]
pub fn summary_by_month(db: State<'_, Db>, months_back: u32) -> AppResult<Vec<MonthSummary>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    let cutoff = compute_cutoff(months_back);

    let mut stmt = conn.prepare(
        "SELECT substr(t.date, 1, 7) AS month, t.amount
         FROM transactions t
         LEFT JOIN categories c ON c.id = t.category_id
         WHERE t.date >= ?1
           AND COALESCE(c.kind, '') != 'transfer'
         ORDER BY t.date ASC",
    )?;
    let rows: Vec<(String, String)> = stmt
        .query_map(params![cutoff], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let mut by_month: HashMap<String, (Decimal, Decimal)> = HashMap::new();
    for (m, amt) in rows {
        let d =
            Decimal::from_str(&amt).map_err(|e| AppError::Invalid(format!("bad amount: {e}")))?;
        let entry = by_month.entry(m).or_insert((Decimal::ZERO, Decimal::ZERO));
        if d.is_sign_negative() {
            entry.1 += -d;
        } else {
            entry.0 += d;
        }
    }

    let mut out: Vec<MonthSummary> = by_month
        .into_iter()
        .map(|(month, (income, expense))| MonthSummary {
            month,
            income: income.to_string(),
            expense: expense.to_string(),
        })
        .collect();
    out.sort_by(|a, b| a.month.cmp(&b.month));
    Ok(out)
}

#[tauri::command]
#[specta::specta]
pub fn investment_summary(
    db: State<'_, Db>,
    month: Option<String>,
) -> AppResult<InvestmentSummary> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    let pattern: Option<String> = month.as_ref().map(|m| format!("{m}-%"));

    // Aggregação do mês.
    let mut stmt = conn.prepare(
        "SELECT t.amount
         FROM transactions t
         JOIN categories c ON c.id = t.category_id
         WHERE c.is_investment = 1
           AND (?1 IS NULL OR t.date LIKE ?1)",
    )?;
    let pat_ref: Option<&str> = pattern.as_deref();
    let month_rows: Vec<String> = stmt
        .query_map(params![pat_ref], |row| row.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let mut aplicado = Decimal::ZERO;
    let mut resgatado = Decimal::ZERO;
    let mut aplicacoes = 0u32;
    let mut resgates = 0u32;
    for s in &month_rows {
        let d = Decimal::from_str(s)
            .map_err(|e| AppError::Invalid(format!("bad amount '{s}': {e}")))?;
        if d.is_sign_negative() {
            aplicado += -d;
            aplicacoes += 1;
        } else if !d.is_zero() {
            resgatado += d;
            resgates += 1;
        }
    }

    // Saldo acumulado all-time. Agregamos em Decimal em Rust pra preservar
    // a precisão monetária (rust_decimal não casa bem com SQL SUM em TEXT).
    let saldo: Decimal = {
        let mut all_stmt = conn.prepare(
            "SELECT t.amount FROM transactions t
             JOIN categories c ON c.id = t.category_id
             WHERE c.is_investment = 1",
        )?;
        let all_rows: Vec<String> = all_stmt
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        let mut s = Decimal::ZERO;
        for amt in &all_rows {
            let d = Decimal::from_str(amt)
                .map_err(|e| AppError::Invalid(format!("bad amount '{amt}': {e}")))?;
            // amount negativo (aplicação) → -d positivo → aumenta saldo investido.
            // amount positivo (resgate)   → -d negativo → diminui saldo investido.
            s += -d;
        }
        s
    };

    Ok(InvestmentSummary {
        aplicado_no_mes: aplicado.to_string(),
        resgatado_no_mes: resgatado.to_string(),
        aplicacoes_count: aplicacoes,
        resgates_count: resgates,
        saldo_acumulado: saldo.to_string(),
    })
}

#[tauri::command]
#[specta::specta]
pub fn transfer_summary(
    db: State<'_, Db>,
    month: Option<String>,
) -> AppResult<TransferSummary> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    let pattern: Option<String> = month.as_ref().map(|m| format!("{m}-%"));

    let mut stmt = conn.prepare(
        "SELECT t.amount
         FROM transactions t
         JOIN categories c ON c.id = t.category_id
         WHERE c.kind = 'transfer'
           AND c.is_investment = 0
           AND (?1 IS NULL OR t.date LIKE ?1)",
    )?;
    let pat_ref: Option<&str> = pattern.as_deref();
    let rows: Vec<String> = stmt
        .query_map(params![pat_ref], |row| row.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let mut total_out = Decimal::ZERO;
    let mut total_in = Decimal::ZERO;
    let count = rows.len() as u32;
    for s in &rows {
        let d = Decimal::from_str(s)
            .map_err(|e| AppError::Invalid(format!("bad amount '{s}': {e}")))?;
        if d.is_sign_negative() {
            total_out += -d;
        } else {
            total_in += d;
        }
    }

    Ok(TransferSummary {
        total_out: total_out.to_string(),
        total_in: total_in.to_string(),
        count,
    })
}

/// Agrega entradas (`amount > 0`, kind != 'transfer') do mês por contraparte.
/// Marca como recorrente quando a mesma fonte aparece em ≥2 meses distintos
/// (considerando todo o histórico da DB, não só o mês filtrado).
#[tauri::command]
#[specta::specta]
pub fn income_sources(
    db: State<'_, Db>,
    month: Option<String>,
) -> AppResult<Vec<IncomeSource>> {
    let conn = db.conn.lock().expect("db mutex poisoned");

    // Carrega TODAS as entradas reais (positivas, não-transfer) de toda a DB.
    // Precisamos do histórico completo pra detectar recorrência.
    let mut stmt = conn.prepare(
        "SELECT t.date, t.amount, t.description
         FROM transactions t
         LEFT JOIN categories c ON c.id = t.category_id
         WHERE COALESCE(c.kind, '') != 'transfer'
           AND CAST(t.amount AS REAL) > 0",
    )?;
    type Row = (String, String, String);
    let rows: Vec<Row> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let target_prefix = month.as_deref().map(|m| format!("{m}-"));

    // Pra cada chave normalizada, rastreia:
    // - meses únicos onde apareceu (pra recorrência)
    // - agregado do mês filtrado (total, count, label)
    struct Agg {
        label: String,
        total: Decimal,
        count: u32,
    }
    let mut months_seen: HashMap<String, HashSet<String>> = HashMap::new();
    let mut current: HashMap<String, Agg> = HashMap::new();

    for (date, amount_str, desc) in rows {
        let (key, label, _pattern) = normalize(&desc);
        let month_key: &str = if date.len() >= 7 { &date[..7] } else { &date };

        months_seen
            .entry(key.clone())
            .or_default()
            .insert(month_key.to_string());

        let in_month = match target_prefix.as_deref() {
            Some(p) => date.starts_with(p),
            None => true,
        };
        if !in_month {
            continue;
        }
        let amt = Decimal::from_str(&amount_str)
            .map_err(|e| AppError::Invalid(format!("bad amount '{amount_str}': {e}")))?;
        let entry = current.entry(key).or_insert_with(|| Agg {
            label,
            total: Decimal::ZERO,
            count: 0,
        });
        entry.total += amt;
        entry.count += 1;
    }

    let total_income: Decimal = current.values().map(|a| a.total).sum();

    let mut sources: Vec<IncomeSource> = current
        .into_iter()
        .map(|(key, agg)| {
            let recurring_months = months_seen.get(&key).map(|s| s.len()).unwrap_or(0) as u32;
            let percent = if total_income.is_zero() {
                0.0
            } else {
                let ratio: f64 = (agg.total / total_income).to_string().parse().unwrap_or(0.0);
                ratio * 100.0
            };
            IncomeSource {
                key,
                label: agg.label,
                total: agg.total.to_string(),
                count: agg.count,
                percent,
                is_recurring: recurring_months >= 2,
                recurring_months,
            }
        })
        .collect();
    sources.sort_by(|a, b| {
        let ad: Decimal = Decimal::from_str(&a.total).unwrap_or(Decimal::ZERO);
        let bd: Decimal = Decimal::from_str(&b.total).unwrap_or(Decimal::ZERO);
        bd.cmp(&ad)
    });
    Ok(sources)
}

fn compute_cutoff(months_back: u32) -> String {
    let now = chrono::Utc::now().naive_utc().date();
    let total_months =
        (chrono::Datelike::year(&now)) * 12 + (chrono::Datelike::month(&now) as i32 - 1);
    let target = total_months - (months_back as i32);
    let y = target.div_euclid(12);
    let m = target.rem_euclid(12) + 1;
    format!("{:04}-{:02}-01", y, m)
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

    fn insert_account(conn: &Connection) -> i64 {
        conn.execute(
            "INSERT INTO accounts (name, bank, ofx_acctid) VALUES ('test', NULL, 'A1')",
            [],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    fn insert_tx(
        conn: &Connection,
        account_id: i64,
        date: &str,
        amount: &str,
        category_id: Option<i64>,
    ) {
        conn.execute(
            "INSERT INTO transactions (account_id, date, amount, description, category_id, ofx_fitid)
             VALUES (?1, ?2, ?3, 'desc', ?4, NULL)",
            params![account_id, date, amount, category_id],
        )
        .unwrap();
    }

    fn cat_id(conn: &Connection, name: &str) -> i64 {
        conn.query_row(
            "SELECT id FROM categories WHERE name = ?1",
            params![name],
            |r| r.get(0),
        )
        .unwrap()
    }

    #[test]
    fn kpis_sum_correctly_for_month() {
        let conn = fresh_conn();
        let acc = insert_account(&conn);
        insert_tx(&conn, acc, "2026-04-05", "100.00", None);
        insert_tx(&conn, acc, "2026-04-10", "-30.00", None);
        insert_tx(&conn, acc, "2026-04-15", "-20.50", None);
        insert_tx(&conn, acc, "2026-03-30", "999.00", None);

        let mut stmt = conn
            .prepare("SELECT amount FROM transactions WHERE date LIKE ?1")
            .unwrap();
        let rows: Vec<String> = stmt
            .query_map(params!["2026-04-%"], |r| r.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        let mut income = Decimal::ZERO;
        let mut expense = Decimal::ZERO;
        for s in &rows {
            let d: Decimal = s.parse().unwrap();
            if d.is_sign_negative() {
                expense += -d;
            } else {
                income += d;
            }
        }
        assert_eq!(income, Decimal::from_str("100.00").unwrap());
        assert_eq!(expense, Decimal::from_str("50.50").unwrap());
        assert_eq!(rows.len(), 3);
    }

    #[test]
    fn by_category_groups_only_expenses() {
        let conn = fresh_conn();
        let acc = insert_account(&conn);
        let mercado = cat_id(&conn, "Mercado");

        insert_tx(&conn, acc, "2026-04-05", "-50.00", Some(mercado));
        insert_tx(&conn, acc, "2026-04-10", "-30.00", Some(mercado));
        // Tx positiva sem categoria — renda não é categorizada após migration 0008.
        insert_tx(&conn, acc, "2026-04-15", "5000.00", None);
        insert_tx(&conn, acc, "2026-04-20", "-10.00", None);

        let mut stmt = conn
            .prepare(
                "SELECT t.amount, t.category_id
                 FROM transactions t WHERE t.date LIKE ?1",
            )
            .unwrap();
        let rows: Vec<(String, Option<i64>)> = stmt
            .query_map(params!["2026-04-%"], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();

        let mut mercado_total = Decimal::ZERO;
        let mut sem_cat_total = Decimal::ZERO;
        let mut renda_count = 0u32;
        for (amt, cid) in &rows {
            let d: Decimal = amt.parse().unwrap();
            if !d.is_sign_negative() {
                renda_count += 1;
                continue;
            }
            let abs = -d;
            if *cid == Some(mercado) {
                mercado_total += abs;
            } else if cid.is_none() {
                sem_cat_total += abs;
            }
        }

        assert_eq!(mercado_total, Decimal::from_str("80.00").unwrap());
        assert_eq!(sem_cat_total, Decimal::from_str("10.00").unwrap());
        assert_eq!(renda_count, 1);
    }

    #[test]
    fn kpis_exclude_transfer_kind_categories() {
        let conn = fresh_conn();
        let acc = insert_account(&conn);
        let transferencias = cat_id(&conn, "Transferências");
        let mercado = cat_id(&conn, "Mercado");

        // Real spending and income, plus a transfer that should be ignored.
        // Renda fica sem categoria (categorias só pra gastos).
        insert_tx(&conn, acc, "2026-04-05", "-100.00", Some(mercado));
        insert_tx(&conn, acc, "2026-04-10", "5000.00", None);
        insert_tx(&conn, acc, "2026-04-15", "-732.52", Some(transferencias)); // pagamento de fatura
        insert_tx(&conn, acc, "2026-04-20", "-1000.00", Some(transferencias)); // aplicação RDB

        let mut stmt = conn
            .prepare(
                "SELECT t.amount FROM transactions t
                 LEFT JOIN categories c ON c.id = t.category_id
                 WHERE t.date LIKE ?1 AND COALESCE(c.kind, '') != 'transfer'",
            )
            .unwrap();
        let rows: Vec<String> = stmt
            .query_map(params!["2026-04-%"], |r| r.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();

        let mut income = Decimal::ZERO;
        let mut expense = Decimal::ZERO;
        for s in &rows {
            let d: Decimal = s.parse().unwrap();
            if d.is_sign_negative() {
                expense += -d;
            } else {
                income += d;
            }
        }
        assert_eq!(income, Decimal::from_str("5000.00").unwrap());
        assert_eq!(expense, Decimal::from_str("100.00").unwrap(),
            "Transferências (-732.52 -1000.00) devem ser ignoradas");
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn cutoff_computation_returns_iso_date() {
        let s = compute_cutoff(12);
        assert_eq!(s.len(), 10);
        assert_eq!(&s[4..5], "-");
        assert_eq!(&s[7..10], "-01");
    }
}
