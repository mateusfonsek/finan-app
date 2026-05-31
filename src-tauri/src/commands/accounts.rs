use rusqlite::params;
use tauri::State;

use crate::db::Db;
use crate::domain::account::{Account, NewAccount};
use crate::error::{AppError, AppResult};

fn validate_kind(kind: &str) -> AppResult<()> {
    if !matches!(kind, "checking" | "credit_card") {
        return Err(AppError::Invalid(format!(
            "kind inválido '{kind}' (esperado 'checking' ou 'credit_card')"
        )));
    }
    Ok(())
}

fn row_to_account(row: &rusqlite::Row) -> rusqlite::Result<Account> {
    Ok(Account {
        id: row.get(0)?,
        name: row.get(1)?,
        bank: row.get(2)?,
        ofx_acctid: row.get(3)?,
        kind: row.get(4)?,
        created_at: row.get(5)?,
    })
}

const SELECT_COLS: &str = "id, name, bank, ofx_acctid, kind, created_at";

#[tauri::command]
#[specta::specta]
pub fn list_accounts(db: State<'_, Db>) -> AppResult<Vec<Account>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    let sql = format!("SELECT {SELECT_COLS} FROM accounts ORDER BY id");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], row_to_account)?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(AppError::from)
}

/// Returns existing account matching `ofx_acctid` if any, otherwise creates one.
/// `kind` é preservado quando a conta já existe (não sobrescreve).
#[tauri::command]
#[specta::specta]
pub fn create_or_get_account(db: State<'_, Db>, input: NewAccount) -> AppResult<Account> {
    validate_kind(&input.kind)?;
    let conn = db.conn.lock().expect("db mutex poisoned");

    if let Some(acctid) = &input.ofx_acctid {
        let sql = format!("SELECT {SELECT_COLS} FROM accounts WHERE ofx_acctid = ?1");
        let existing: Option<Account> = conn
            .query_row(&sql, params![acctid], row_to_account)
            .ok();
        if let Some(account) = existing {
            return Ok(account);
        }
    }

    conn.execute(
        "INSERT INTO accounts (name, bank, ofx_acctid, kind) VALUES (?1, ?2, ?3, ?4)",
        params![input.name, input.bank, input.ofx_acctid, input.kind],
    )?;
    let id = conn.last_insert_rowid();

    let sql = format!("SELECT {SELECT_COLS} FROM accounts WHERE id = ?1");
    conn.query_row(&sql, params![id], row_to_account)
        .map_err(AppError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;
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
        kind: &str,
    ) -> i64 {
        conn.execute(
            "INSERT INTO accounts (name, bank, ofx_acctid, kind) VALUES (?1, ?2, ?3, ?4)",
            params![name, bank, acctid, kind],
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
        let id1 = create_account_raw(
            &conn,
            "Itaú Conta Corrente",
            Some("itau"),
            Some("12345-6"),
            "checking",
        );

        let sql = format!("SELECT {SELECT_COLS} FROM accounts WHERE ofx_acctid = ?1");
        let found: Account = conn
            .query_row(&sql, params!["12345-6"], row_to_account)
            .unwrap();

        assert_eq!(found.id, id1);
        assert_eq!(found.bank.as_deref(), Some("itau"));
        assert_eq!(found.kind, "checking");
    }

    #[test]
    fn schema_allows_null_acctid_for_manual_accounts() {
        let conn = fresh_conn();
        create_account_raw(&conn, "Manual", None, None, "checking");
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM accounts", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn supports_credit_card_kind() {
        let conn = fresh_conn();
        let id = create_account_raw(
            &conn,
            "Nubank · cartão",
            Some("nubank"),
            Some("uuid-do-cartao"),
            "credit_card",
        );
        let sql = format!("SELECT {SELECT_COLS} FROM accounts WHERE id = ?1");
        let found: Account = conn.query_row(&sql, params![id], row_to_account).unwrap();
        assert_eq!(found.kind, "credit_card");
    }

    #[test]
    fn rejects_invalid_kind_in_check_constraint() {
        let conn = fresh_conn();
        let result = conn.execute(
            "INSERT INTO accounts (name, bank, ofx_acctid, kind) VALUES (?1, ?2, ?3, ?4)",
            params!["x", "y", "z", "garbage"],
        );
        assert!(result.is_err(), "CHECK deveria bloquear kind inválido");
    }
}
