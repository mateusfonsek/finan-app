use rusqlite::params;
use tauri::State;

use crate::db::Db;
use crate::domain::account::{Account, NewAccount};
use crate::error::{AppError, AppResult};

#[tauri::command]
#[specta::specta]
pub fn list_accounts(db: State<'_, Db>) -> AppResult<Vec<Account>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    let mut stmt =
        conn.prepare("SELECT id, name, bank, ofx_acctid, created_at FROM accounts ORDER BY id")?;
    let rows = stmt.query_map([], |row| {
        Ok(Account {
            id: row.get(0)?,
            name: row.get(1)?,
            bank: row.get(2)?,
            ofx_acctid: row.get(3)?,
            created_at: row.get(4)?,
        })
    })?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(AppError::from)
}

/// Returns existing account matching `ofx_acctid` if any, otherwise creates one.
#[tauri::command]
#[specta::specta]
pub fn create_or_get_account(db: State<'_, Db>, input: NewAccount) -> AppResult<Account> {
    let conn = db.conn.lock().expect("db mutex poisoned");

    if let Some(acctid) = &input.ofx_acctid {
        let existing: Option<Account> = conn
            .query_row(
                "SELECT id, name, bank, ofx_acctid, created_at FROM accounts WHERE ofx_acctid = ?1",
                params![acctid],
                |row| {
                    Ok(Account {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        bank: row.get(2)?,
                        ofx_acctid: row.get(3)?,
                        created_at: row.get(4)?,
                    })
                },
            )
            .ok();
        if let Some(account) = existing {
            return Ok(account);
        }
    }

    conn.execute(
        "INSERT INTO accounts (name, bank, ofx_acctid) VALUES (?1, ?2, ?3)",
        params![input.name, input.bank, input.ofx_acctid],
    )?;
    let id = conn.last_insert_rowid();

    conn.query_row(
        "SELECT id, name, bank, ofx_acctid, created_at FROM accounts WHERE id = ?1",
        params![id],
        |row| {
            Ok(Account {
                id: row.get(0)?,
                name: row.get(1)?,
                bank: row.get(2)?,
                ofx_acctid: row.get(3)?,
                created_at: row.get(4)?,
            })
        },
    )
    .map_err(AppError::from)
}

#[cfg(test)]
mod tests {
    use crate::db::migrations;
    use crate::domain::account::Account;
    use rusqlite::{params, Connection};

    fn fresh_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        migrations::apply(&conn).unwrap();
        conn
    }

    fn create_account_raw(
        conn: &Connection,
        name: &str,
        bank: Option<&str>,
        acctid: Option<&str>,
    ) -> i64 {
        conn.execute(
            "INSERT INTO accounts (name, bank, ofx_acctid) VALUES (?1, ?2, ?3)",
            params![name, bank, acctid],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    #[test]
    fn list_accounts_returns_empty_initially() {
        let conn = fresh_conn();
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM accounts").unwrap();
        let count: i64 = stmt.query_row([], |r| r.get(0)).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn create_or_get_returns_existing_when_acctid_matches() {
        let conn = fresh_conn();
        let id1 = create_account_raw(&conn, "Itaú Conta Corrente", Some("itau"), Some("12345-6"));

        let found: Account = conn
            .query_row(
                "SELECT id, name, bank, ofx_acctid, created_at FROM accounts WHERE ofx_acctid = ?1",
                params!["12345-6"],
                |row| {
                    Ok(Account {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        bank: row.get(2)?,
                        ofx_acctid: row.get(3)?,
                        created_at: row.get(4)?,
                    })
                },
            )
            .unwrap();

        assert_eq!(found.id, id1);
        assert_eq!(found.bank.as_deref(), Some("itau"));
    }

    #[test]
    fn schema_allows_null_acctid_for_manual_accounts() {
        let conn = fresh_conn();
        create_account_raw(&conn, "Manual", None, None);
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM accounts", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }
}
