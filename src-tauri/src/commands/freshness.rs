//! How current the imported data is, per account.
//!
//! Only dates leave this module. "How many days behind is that?" is answered in
//! the frontend, where the clock and the timezone are the user's.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

use crate::db::Db;
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AccountFreshness {
    pub account_id: i64,
    /// Already human-readable: the importer builds it from the OFX ("Nubank ·
    /// cartão"), so it carries the account kind without a field of its own.
    pub name: String,
    /// Newest transaction date (`YYYY-MM-DD`) on this account. `None` for an
    /// account that has none — an import that created the account and then
    /// failed must not pass for data.
    pub latest_date: Option<String>,
}

pub fn query_freshness(conn: &Connection) -> AppResult<Vec<AccountFreshness>> {
    let mut stmt = conn.prepare(
        "SELECT a.id, a.name, MAX(t.date)
           FROM accounts a
           LEFT JOIN transactions t ON t.account_id = a.id
          GROUP BY a.id
          ORDER BY a.id",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(AccountFreshness {
            account_id: row.get(0)?,
            name: row.get(1)?,
            latest_date: row.get(2)?,
        })
    })?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(AppError::from)
}

#[tauri::command]
#[specta::specta]
pub fn data_freshness(db: State<'_, Db>) -> AppResult<Vec<AccountFreshness>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    query_freshness(&conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;
    use rusqlite::params;

    fn fresh_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        migrations::apply(&conn).unwrap();
        conn
    }

    fn account(conn: &Connection, name: &str, kind: &str) -> i64 {
        conn.execute(
            "INSERT INTO accounts (name, kind) VALUES (?1, ?2)",
            params![name, kind],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    fn transaction(conn: &Connection, account_id: i64, date: &str) {
        conn.execute(
            "INSERT INTO transactions (account_id, date, amount, description)
             VALUES (?1, ?2, '-10.00', 'x')",
            params![account_id, date],
        )
        .unwrap();
    }

    #[test]
    fn latest_date_is_the_newest_transaction_of_the_account() {
        let conn = fresh_conn();
        let checking = account(&conn, "Checking", "checking");
        transaction(&conn, checking, "2026-06-30");
        transaction(&conn, checking, "2026-07-12");
        transaction(&conn, checking, "2026-07-02");

        let rows = query_freshness(&conn).unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].latest_date.as_deref(), Some("2026-07-12"));
    }

    /// The whole point of grouping per account: a checking account imported
    /// yesterday must not hide a card that stopped in June.
    #[test]
    fn each_account_keeps_its_own_date() {
        let conn = fresh_conn();
        let checking = account(&conn, "Checking", "checking");
        let card = account(&conn, "Card", "credit_card");
        transaction(&conn, checking, "2026-07-12");
        transaction(&conn, card, "2026-06-30");

        let rows = query_freshness(&conn).unwrap();

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].name, "Checking");
        assert_eq!(rows[0].latest_date.as_deref(), Some("2026-07-12"));
        assert_eq!(rows[1].name, "Card");
        assert_eq!(rows[1].latest_date.as_deref(), Some("2026-06-30"));
    }

    #[test]
    fn an_account_with_no_transactions_is_listed_with_no_date() {
        let conn = fresh_conn();
        account(&conn, "Empty", "checking");

        let rows = query_freshness(&conn).unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].latest_date, None);
    }

    #[test]
    fn a_database_with_no_account_returns_nothing() {
        let conn = fresh_conn();

        assert!(query_freshness(&conn).unwrap().is_empty());
    }
}
