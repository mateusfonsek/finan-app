use rusqlite::params;
use tauri::State;

use crate::db::Db;
use crate::domain::transaction::{InsertResult, NewTransaction, Transaction};
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct TransactionFilters {
    pub account_id: Option<i64>,
    pub month: Option<String>,
    pub category_id: Option<i64>,
    pub limit: Option<u32>,
}

#[tauri::command]
#[specta::specta]
pub fn list_transactions(
    db: State<'_, Db>,
    filters: Option<TransactionFilters>,
) -> AppResult<Vec<Transaction>> {
    let f = filters.unwrap_or_default();
    let conn = db.conn.lock().expect("db mutex poisoned");

    let mut where_clauses: Vec<String> = Vec::new();
    let mut bound: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(id) = f.account_id {
        where_clauses.push(format!("account_id = ?{}", bound.len() + 1));
        bound.push(Box::new(id));
    }
    if let Some(month) = f.month.as_ref() {
        where_clauses.push(format!("date LIKE ?{}", bound.len() + 1));
        bound.push(Box::new(format!("{month}-%")));
    }
    if let Some(cid) = f.category_id {
        where_clauses.push(format!("category_id = ?{}", bound.len() + 1));
        bound.push(Box::new(cid));
    }

    let where_sql = if where_clauses.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", where_clauses.join(" AND "))
    };
    let limit_sql = match f.limit {
        Some(n) => format!(" LIMIT {n}"),
        None => String::new(),
    };
    let sql = format!(
        "SELECT id, account_id, date, amount, description, category_id, notes, ofx_fitid, imported_at
         FROM transactions{where_sql} ORDER BY date DESC, id DESC{limit_sql}",
    );

    let mut stmt = conn.prepare(&sql)?;
    let params_refs: Vec<&dyn rusqlite::ToSql> = bound.iter().map(|b| b.as_ref()).collect();
    let rows = stmt.query_map(params_refs.as_slice(), |row| {
        Ok(Transaction {
            id: row.get(0)?,
            account_id: row.get(1)?,
            date: row.get(2)?,
            amount: row.get(3)?,
            description: row.get(4)?,
            category_id: row.get(5)?,
            notes: row.get(6)?,
            ofx_fitid: row.get(7)?,
            imported_at: row.get(8)?,
        })
    })?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(AppError::from)
}

/// Insert a batch of new transactions. Returns counts of inserted vs skipped (duplicates).
/// Dedup happens via UNIQUE(account_id, ofx_fitid) + INSERT OR IGNORE.
#[tauri::command]
#[specta::specta]
pub fn insert_transactions(
    db: State<'_, Db>,
    account_id: i64,
    txs: Vec<NewTransaction>,
) -> AppResult<InsertResult> {
    let mut conn = db.conn.lock().expect("db mutex poisoned");

    for (idx, tx) in txs.iter().enumerate() {
        tx.parse_amount().map_err(|e| {
            AppError::Invalid(format!("tx[{idx}]: invalid amount '{}': {e}", tx.amount))
        })?;
    }

    let tx_conn = conn.transaction()?;
    let mut inserted = 0u32;
    let mut skipped = 0u32;
    {
        let mut stmt = tx_conn.prepare(
            "INSERT OR IGNORE INTO transactions
                (account_id, date, amount, description, ofx_fitid)
             VALUES (?1, ?2, ?3, ?4, ?5)",
        )?;
        for tx in &txs {
            let changes = stmt.execute(params![
                account_id,
                tx.date,
                tx.amount,
                tx.description,
                tx.ofx_fitid,
            ])?;
            if changes == 1 {
                inserted += 1;
            } else {
                skipped += 1;
            }
        }
    }
    tx_conn.commit()?;

    let auto_categorized =
        crate::commands::rules::apply_rules_internal(&mut conn, Some(account_id))?;

    Ok(InsertResult {
        inserted,
        skipped_duplicates: skipped,
        auto_categorized,
    })
}

/// Given a list of FITIDs, return the subset that already exists for this account.
#[tauri::command]
#[specta::specta]
pub fn check_existing_fitids(
    db: State<'_, Db>,
    account_id: i64,
    fitids: Vec<String>,
) -> AppResult<Vec<String>> {
    if fitids.is_empty() {
        return Ok(Vec::new());
    }
    let conn = db.conn.lock().expect("db mutex poisoned");

    let placeholders = (1..=fitids.len())
        .map(|i| format!("?{}", i + 1))
        .collect::<Vec<_>>()
        .join(",");
    let sql = format!(
        "SELECT ofx_fitid FROM transactions
         WHERE account_id = ?1 AND ofx_fitid IN ({placeholders})",
    );

    let mut stmt = conn.prepare(&sql)?;

    let mut params_vec: Vec<&dyn rusqlite::ToSql> = Vec::with_capacity(fitids.len() + 1);
    params_vec.push(&account_id);
    for f in &fitids {
        params_vec.push(f);
    }

    let rows = stmt.query_map(params_vec.as_slice(), |row| row.get::<_, Option<String>>(0))?;
    let existing: Vec<String> = rows.filter_map(|r| r.ok().flatten()).collect();

    Ok(existing)
}

#[tauri::command]
#[specta::specta]
pub fn update_transaction_category(
    db: State<'_, Db>,
    transaction_id: i64,
    category_id: Option<i64>,
) -> AppResult<()> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    let changed = conn.execute(
        "UPDATE transactions SET category_id = ?1 WHERE id = ?2",
        params![category_id, transaction_id],
    )?;
    if changed == 0 {
        return Err(AppError::Invalid(format!(
            "transaction {transaction_id} not found"
        )));
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn update_transaction_notes(
    db: State<'_, Db>,
    transaction_id: i64,
    notes: Option<String>,
) -> AppResult<()> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    let changed = conn.execute(
        "UPDATE transactions SET notes = ?1 WHERE id = ?2",
        params![notes, transaction_id],
    )?;
    if changed == 0 {
        return Err(AppError::Invalid(format!(
            "transaction {transaction_id} not found"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::db::migrations;
    use crate::domain::transaction::NewTransaction;
    use rusqlite::{params, Connection};

    fn fresh_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        migrations::apply(&conn).unwrap();
        conn
    }

    fn insert_account(conn: &Connection, name: &str, acctid: Option<&str>) -> i64 {
        conn.execute(
            "INSERT INTO accounts (name, bank, ofx_acctid) VALUES (?1, NULL, ?2)",
            params![name, acctid],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    fn raw_insert_batch(
        conn: &mut Connection,
        account_id: i64,
        txs: &[NewTransaction],
    ) -> (u32, u32) {
        let tx_conn = conn.transaction().unwrap();
        let mut inserted = 0u32;
        let mut skipped = 0u32;
        {
            let mut stmt = tx_conn
                .prepare(
                    "INSERT OR IGNORE INTO transactions
                        (account_id, date, amount, description, ofx_fitid)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                )
                .unwrap();
            for tx in txs {
                let changes = stmt
                    .execute(params![
                        account_id,
                        tx.date,
                        tx.amount,
                        tx.description,
                        tx.ofx_fitid
                    ])
                    .unwrap();
                if changes == 1 {
                    inserted += 1;
                } else {
                    skipped += 1;
                }
            }
        }
        tx_conn.commit().unwrap();
        (inserted, skipped)
    }

    fn mk(fitid: &str, amount: &str) -> NewTransaction {
        NewTransaction {
            date: "2026-04-12".into(),
            amount: amount.into(),
            description: format!("desc-{fitid}"),
            ofx_fitid: Some(fitid.into()),
        }
    }

    #[test]
    fn insert_batch_inserts_all_when_no_dups() {
        let mut conn = fresh_conn();
        let acc = insert_account(&conn, "test", Some("ACC1"));
        let txs = vec![mk("F1", "10.00"), mk("F2", "-5.50"), mk("F3", "100.00")];
        let (ins, skip) = raw_insert_batch(&mut conn, acc, &txs);
        assert_eq!(ins, 3);
        assert_eq!(skip, 0);
    }

    #[test]
    fn insert_batch_skips_duplicates() {
        let mut conn = fresh_conn();
        let acc = insert_account(&conn, "test", Some("ACC1"));
        let txs1 = vec![mk("F1", "10.00"), mk("F2", "-5.50")];
        raw_insert_batch(&mut conn, acc, &txs1);

        let txs2 = vec![mk("F1", "10.00"), mk("F2", "-5.50"), mk("F3", "100.00")];
        let (ins, skip) = raw_insert_batch(&mut conn, acc, &txs2);
        assert_eq!(ins, 1);
        assert_eq!(skip, 2);
    }

    #[test]
    fn same_fitid_different_account_is_not_duplicate() {
        let mut conn = fresh_conn();
        let a1 = insert_account(&conn, "acc1", Some("A1"));
        let a2 = insert_account(&conn, "acc2", Some("A2"));
        let txs = vec![mk("SHARED_FITID", "10.00")];
        raw_insert_batch(&mut conn, a1, &txs);
        let (ins, skip) = raw_insert_batch(&mut conn, a2, &txs);
        assert_eq!(ins, 1, "different account = no collision");
        assert_eq!(skip, 0);
    }

    #[test]
    fn null_fitid_does_not_dedup() {
        let mut conn = fresh_conn();
        let acc = insert_account(&conn, "test", Some("ACC1"));
        let mut a = mk("X", "10.00");
        a.ofx_fitid = None;
        let mut b = mk("Y", "10.00");
        b.ofx_fitid = None;
        let (ins, skip) = raw_insert_batch(&mut conn, acc, &[a, b]);
        assert_eq!(ins, 2, "NULL FITID = SQLite considers each row unique");
        assert_eq!(skip, 0);
    }

    #[test]
    fn check_existing_fitids_returns_subset() {
        let mut conn = fresh_conn();
        let acc = insert_account(&conn, "test", Some("ACC1"));
        let txs = vec![mk("F1", "10"), mk("F2", "20"), mk("F3", "30")];
        raw_insert_batch(&mut conn, acc, &txs);

        let mut stmt = conn
            .prepare("SELECT ofx_fitid FROM transactions WHERE account_id = ?1 AND ofx_fitid IN (?2, ?3)")
            .unwrap();
        let rows = stmt
            .query_map(params![acc, "F1", "F4"], |r| r.get::<_, Option<String>>(0))
            .unwrap();
        let existing: Vec<String> = rows.filter_map(|r| r.ok().flatten()).collect();

        assert_eq!(existing, vec!["F1".to_string()]);
    }

    #[test]
    fn list_transactions_filter_by_month() {
        let mut conn = fresh_conn();
        let acc = insert_account(&conn, "test", Some("ACC1"));
        let mut a = mk("F1", "10.00");
        a.date = "2026-03-15".into();
        let mut b = mk("F2", "-5.00");
        b.date = "2026-04-02".into();
        raw_insert_batch(&mut conn, acc, &[a, b]);

        let mut stmt = conn
            .prepare(
                "SELECT id FROM transactions WHERE account_id = ?1 AND date LIKE ?2 ORDER BY date DESC",
            )
            .unwrap();
        let ids: Vec<i64> = stmt
            .query_map(params![acc, "2026-03-%"], |r| r.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        assert_eq!(ids.len(), 1, "only March transaction should match");
    }

    #[test]
    fn limit_clause_caps_result_count() {
        let mut conn = fresh_conn();
        let acc = insert_account(&conn, "test", Some("ACC1"));
        let txs = vec![mk("F1", "10.00"), mk("F2", "20.00"), mk("F3", "30.00")];
        raw_insert_batch(&mut conn, acc, &txs);

        let mut stmt = conn
            .prepare("SELECT id FROM transactions ORDER BY date DESC, id DESC LIMIT 2")
            .unwrap();
        let rows: Vec<i64> = stmt
            .query_map([], |r| r.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn update_transaction_category_changes_value() {
        let mut conn = fresh_conn();
        let acc = insert_account(&conn, "test", Some("ACC1"));
        let txs = vec![mk("F1", "10")];
        raw_insert_batch(&mut conn, acc, &txs);
        let tx_id: i64 = conn
            .query_row(
                "SELECT id FROM transactions WHERE ofx_fitid = 'F1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let cat_id: i64 = conn
            .query_row(
                "SELECT id FROM categories WHERE name = 'Mercado'",
                [],
                |r| r.get(0),
            )
            .unwrap();

        conn.execute(
            "UPDATE transactions SET category_id = ?1 WHERE id = ?2",
            params![cat_id, tx_id],
        )
        .unwrap();

        let stored: i64 = conn
            .query_row(
                "SELECT category_id FROM transactions WHERE id = ?1",
                params![tx_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(stored, cat_id);
    }
}
