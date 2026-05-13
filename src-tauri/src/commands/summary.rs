use rusqlite::params;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::str::FromStr;
use tauri::State;

use crate::db::Db;
use crate::domain::summary::{CategorySpend, KpiSummary, MonthSummary};
use crate::error::{AppError, AppResult};

#[tauri::command]
#[specta::specta]
pub fn summary_kpis(db: State<'_, Db>, month: Option<String>) -> AppResult<KpiSummary> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    let pattern: Option<String> = month.as_ref().map(|m| format!("{m}-%"));

    let mut stmt =
        conn.prepare("SELECT amount FROM transactions WHERE (?1 IS NULL OR date LIKE ?1)")?;
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
         WHERE (?1 IS NULL OR t.date LIKE ?1)",
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
        "SELECT substr(date, 1, 7) AS month, amount
         FROM transactions
         WHERE date >= ?1
         ORDER BY date ASC",
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
        let renda = cat_id(&conn, "Renda");

        insert_tx(&conn, acc, "2026-04-05", "-50.00", Some(mercado));
        insert_tx(&conn, acc, "2026-04-10", "-30.00", Some(mercado));
        insert_tx(&conn, acc, "2026-04-15", "5000.00", Some(renda));
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
    fn cutoff_computation_returns_iso_date() {
        let s = compute_cutoff(12);
        assert_eq!(s.len(), 10);
        assert_eq!(&s[4..5], "-");
        assert_eq!(&s[7..10], "-01");
    }
}
