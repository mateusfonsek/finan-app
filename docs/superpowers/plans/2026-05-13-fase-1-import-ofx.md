# Fase 1 — Import OFX Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implementar o fluxo completo de importar extrato OFX brasileiro: usuário arrasta um arquivo `.ofx` em `/onboarding` ou `/import`, o frontend parseia em TS (com normalização de encoding + bank quirks), envia transações pro Rust que persiste no SQLite com dedup por `UNIQUE(account_id, ofx_fitid)`, e o usuário é redirecionado pra `/transactions` vendo a lista crua das transações importadas.

**Architecture:** Parsing OFX acontece 100% no frontend (TypeScript via `ofx-data-extractor`) — não existe lib madura em Rust. Backend Rust expõe commands tipados (`list_accounts`, `create_account`, `list_transactions`, `insert_transactions`, `check_existing_fitids`) gerados via `tauri-specta`. Dedupe é feito client-side (pre-check via `check_existing_fitids`) E server-side (`INSERT OR IGNORE ON CONFLICT`). Conta é auto-criada na primeira importação a partir do `<ACCTID>` do OFX.

**Tech Stack:**
- Rust: `rusqlite` + `rust_decimal` + `chrono` + `tauri-specta` (já instalado)
- TS: `ofx-data-extractor` (novo) + Svelte 5 runes + svelte-spa-router
- Tailwind 4 + shadcn-svelte (Button já existe)

**Acceptance criteria (Fase 1):**
1. Importar um extrato OFX real (Itaú, Nubank ou Bradesco) via drag-and-drop OU clique → "Escolher arquivo".
2. Preview mostra: banco detectado, número de transações, período, total entradas/saídas, marca duplicadas (`existing FITID`).
3. Confirmar import grava no SQLite. Reimport do MESMO arquivo NÃO duplica (`INSERT OR IGNORE` + UNIQUE).
4. Redirect automático pra `/transactions` filtrado pelas recém-importadas.
5. `/transactions` lista todas as transações ordenadas por data desc, mostrando date / description / amount (sem categoria por enquanto — fase 2).
6. Todos os testes passam: `cargo test --lib` ≥ 10 testes (5 antigos + 5+ novos), `pnpm test` ≥ 8 testes (4 antigos + 4+ novos), `pnpm check` 0 erros, clippy `-D warnings` limpo.

**Out of scope (próximas fases):**
- Categorização (fase 2)
- Regras automáticas (fase 3)
- Dashboard (fase 4)

---

## Estrutura de arquivos

```
finan-app/
├── src-tauri/
│   ├── Cargo.toml                       (nenhuma dep nova — já temos tudo)
│   └── src/
│       ├── domain/                      ← novo módulo
│       │   ├── mod.rs                   T1
│       │   ├── account.rs               T1
│       │   └── transaction.rs           T1
│       ├── commands/
│       │   ├── mod.rs                   T4 (atualizado)
│       │   ├── accounts.rs              T2
│       │   ├── transactions.rs          T3
│       │   └── health.rs                (sem mudança)
│       └── lib.rs                       T4 (registra novos commands)
└── src/
    └── lib/
        ├── ofx/                          ← novo módulo
        │   ├── types.ts                 T5
        │   ├── parse.ts                 T5
        │   ├── parse.test.ts            T5
        │   ├── normalize.ts             T6
        │   ├── normalize.test.ts        T6
        │   └── __fixtures__/
        │       └── itau-minimal.ofx     T5
        ├── api/
        │   ├── accounts.ts              T7
        │   └── transactions.ts          T7
        └── components/
            ├── import/
            │   ├── DropZone.svelte      T8
            │   └── ImportPreview.svelte T10
            └── transactions/
                └── TxTable.svelte       T11
    └── routes/
        ├── Onboarding.svelte            T9 (atualizado)
        ├── Import.svelte                T10 (rewrite)
        └── Transactions.svelte          T11 (rewrite)
```

---

## Task 1: Rust domain types — Account & Transaction

**Files:**
- Create: `src-tauri/src/domain/mod.rs`
- Create: `src-tauri/src/domain/account.rs`
- Create: `src-tauri/src/domain/transaction.rs`
- Modify: `src-tauri/src/lib.rs` (declare `mod domain;`)

- [ ] **Step 1: Create `src-tauri/src/domain/mod.rs`**

```rust
pub mod account;
pub mod transaction;
```

- [ ] **Step 2: Create `src-tauri/src/domain/account.rs`**

```rust
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Account {
    pub id: i64,
    pub name: String,
    pub bank: Option<String>,
    pub ofx_acctid: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct NewAccount {
    pub name: String,
    pub bank: Option<String>,
    pub ofx_acctid: Option<String>,
}
```

- [ ] **Step 3: Create `src-tauri/src/domain/transaction.rs`**

```rust
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Transaction {
    pub id: i64,
    pub account_id: i64,
    pub date: String,
    /// Decimal serialized as string (e.g. "-123.45"). Never f64.
    pub amount: String,
    pub description: String,
    pub category_id: Option<i64>,
    pub notes: Option<String>,
    pub ofx_fitid: Option<String>,
    pub imported_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct NewTransaction {
    pub date: String,
    /// Decimal as string. Backend converts via rust_decimal.
    pub amount: String,
    pub description: String,
    pub ofx_fitid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct InsertResult {
    pub inserted: u32,
    pub skipped_duplicates: u32,
}

impl NewTransaction {
    pub fn parse_amount(&self) -> Result<Decimal, rust_decimal::Error> {
        self.amount.parse::<Decimal>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[test]
    fn new_transaction_parses_positive_amount() {
        let t = NewTransaction {
            date: "2026-04-12".into(),
            amount: "123.45".into(),
            description: "salary".into(),
            ofx_fitid: Some("ABC123".into()),
        };
        assert_eq!(t.parse_amount().unwrap(), Decimal::from_str("123.45").unwrap());
    }

    #[test]
    fn new_transaction_parses_negative_amount() {
        let t = NewTransaction {
            date: "2026-04-12".into(),
            amount: "-50.99".into(),
            description: "grocery".into(),
            ofx_fitid: Some("ABC124".into()),
        };
        assert_eq!(t.parse_amount().unwrap(), Decimal::from_str("-50.99").unwrap());
    }

    #[test]
    fn new_transaction_rejects_garbage_amount() {
        let t = NewTransaction {
            date: "2026-04-12".into(),
            amount: "abc".into(),
            description: "x".into(),
            ofx_fitid: None,
        };
        assert!(t.parse_amount().is_err());
    }
}
```

- [ ] **Step 4: Declare module in `src-tauri/src/lib.rs`**

Add `mod domain;` at the top (after `mod commands;` and `mod db;`):

```rust
mod commands;
mod db;
mod domain;
mod error;
```

- [ ] **Step 5: Run tests**

```bash
cd src-tauri && cargo test --lib 2>&1 | tail -15 && cd ..
```
Expected: 8 tests pass total (5 prior + 3 new from `domain::transaction::tests`).

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/domain/ src-tauri/src/lib.rs
git commit -m "$(cat <<'EOF'
feat(domain): Account, Transaction, NewTransaction, InsertResult types

- Decimal serialized as string (IPC contract: TS recebe string, soma no Rust)
- NewTransaction::parse_amount valida a string antes de persistir
- 3 testes (positivo, negativo, garbage)

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 2: Account commands (list, create_or_get)

**Files:**
- Create: `src-tauri/src/commands/accounts.rs`
- Modify: `src-tauri/src/commands/mod.rs`

- [ ] **Step 1: Update `src-tauri/src/commands/mod.rs`**

```rust
pub mod accounts;
pub mod health;
```

- [ ] **Step 2: Create `src-tauri/src/commands/accounts.rs`**

```rust
use rusqlite::params;
use tauri::State;

use crate::db::Db;
use crate::domain::account::{Account, NewAccount};
use crate::error::{AppError, AppResult};

#[tauri::command]
#[specta::specta]
pub fn list_accounts(db: State<'_, Db>) -> AppResult<Vec<Account>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    let mut stmt = conn.prepare(
        "SELECT id, name, bank, ofx_acctid, created_at FROM accounts ORDER BY id",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Account {
            id: row.get(0)?,
            name: row.get(1)?,
            bank: row.get(2)?,
            ofx_acctid: row.get(3)?,
            created_at: row.get(4)?,
        })
    })?;
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(AppError::from)
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
    use super::*;
    use crate::db::migrations;
    use rusqlite::Connection;

    fn fresh_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        migrations::apply(&conn).unwrap();
        conn
    }

    fn create_account_raw(conn: &Connection, name: &str, bank: Option<&str>, acctid: Option<&str>) -> i64 {
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
        let mut stmt = conn
            .prepare("SELECT COUNT(*) FROM accounts")
            .unwrap();
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
```

> **Nota técnica sobre os testes:** as funções `list_accounts` e `create_or_get_account` recebem `State<'_, Db>` (Tauri runtime state) e por isso não podem ser invocadas diretamente em `cargo test` sem montar uma instância de Tauri. Os testes acima validam a LÓGICA SQL subjacente (queries idênticas às do comando). Quando T3 adicionar `insert_transactions` com dedup, o mesmo padrão se aplica.

- [ ] **Step 3: Run tests**

```bash
cd src-tauri && cargo test --lib 2>&1 | tail -15 && cd ..
```
Expected: 11 tests pass (8 prior + 3 new from `accounts::tests`).

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/
git commit -m "$(cat <<'EOF'
feat(commands): list_accounts + create_or_get_account

- create_or_get_account procura por ofx_acctid antes de inserir (idempotente
  durante reimport do mesmo OFX)
- list_accounts retorna Vec<Account> ordenado por id
- 3 testes (count vazio, lookup por acctid, NULL acctid)

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: Transaction commands (list, insert_batch, check_existing_fitids)

**Files:**
- Create: `src-tauri/src/commands/transactions.rs`
- Modify: `src-tauri/src/commands/mod.rs`

- [ ] **Step 1: Update `src-tauri/src/commands/mod.rs`**

```rust
pub mod accounts;
pub mod health;
pub mod transactions;
```

- [ ] **Step 2: Create `src-tauri/src/commands/transactions.rs`**

```rust
use rusqlite::params;
use tauri::State;

use crate::db::Db;
use crate::domain::transaction::{InsertResult, NewTransaction, Transaction};
use crate::error::{AppError, AppResult};

/// List transactions for an account (or all if account_id is None), ordered by date desc.
#[tauri::command]
#[specta::specta]
pub fn list_transactions(
    db: State<'_, Db>,
    account_id: Option<i64>,
) -> AppResult<Vec<Transaction>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    let (sql, params_vec): (&str, Vec<&dyn rusqlite::ToSql>) = match account_id {
        Some(id) => (
            "SELECT id, account_id, date, amount, description, category_id, notes, ofx_fitid, imported_at
             FROM transactions WHERE account_id = ?1 ORDER BY date DESC, id DESC",
            vec![&id as &dyn rusqlite::ToSql],
        ),
        None => (
            "SELECT id, account_id, date, amount, description, category_id, notes, ofx_fitid, imported_at
             FROM transactions ORDER BY date DESC, id DESC",
            Vec::new(),
        ),
    };

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(params_vec.as_slice(), |row| {
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
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(AppError::from)
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

    Ok(InsertResult {
        inserted,
        skipped_duplicates: skipped,
    })
}

/// Given a list of FITIDs, return the subset that already exists for this account.
/// Used by the frontend to mark duplicates in the preview before commit.
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

    let placeholders = (1..=fitids.len()).map(|i| format!("?{}", i + 1)).collect::<Vec<_>>().join(",");
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
    let existing: Vec<String> = rows
        .filter_map(|r| r.ok().flatten())
        .collect();

    Ok(existing)
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

    fn raw_insert_batch(conn: &mut Connection, account_id: i64, txs: &[NewTransaction]) -> (u32, u32) {
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
                    .execute(params![account_id, tx.date, tx.amount, tx.description, tx.ofx_fitid])
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

        // re-import: F1 + F2 should be skipped, F3 inserted
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

        // query F1, F4 — should return [F1]
        let mut stmt = conn
            .prepare("SELECT ofx_fitid FROM transactions WHERE account_id = ?1 AND ofx_fitid IN (?2, ?3)")
            .unwrap();
        let rows = stmt
            .query_map(params![acc, "F1", "F4"], |r| r.get::<_, Option<String>>(0))
            .unwrap();
        let existing: Vec<String> = rows.filter_map(|r| r.ok().flatten()).collect();

        assert_eq!(existing, vec!["F1".to_string()]);
    }
}
```

- [ ] **Step 3: Run tests**

```bash
cd src-tauri && cargo test --lib 2>&1 | tail -20 && cd ..
```
Expected: 16 tests pass (11 prior + 5 new from `transactions::tests`).

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/
git commit -m "$(cat <<'EOF'
feat(commands): list_transactions + insert_transactions + check_existing_fitids

- insert_transactions: batch insert dentro de transação, INSERT OR IGNORE pra dedup
  por UNIQUE(account_id, ofx_fitid); valida amount via rust_decimal antes;
  retorna InsertResult { inserted, skipped_duplicates }
- list_transactions: filtro opcional por account_id, ordem date desc
- check_existing_fitids: subset query pra preview marcar duplicadas antes do commit
- 5 testes (insert all, skip dups, cross-account, NULL fitid, check subset)

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 4: Register commands + regenerate bindings.ts

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Update `src-tauri/src/lib.rs`**

Replace the `collect_commands!` macro call (currently has only `health_check`) with the full list:

```rust
mod commands;
mod db;
mod domain;
mod error;

use tauri::Manager;
use tauri_specta::{collect_commands, Builder};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let specta_builder = Builder::<tauri::Wry>::new().commands(collect_commands![
        commands::health::health_check,
        commands::accounts::list_accounts,
        commands::accounts::create_or_get_account,
        commands::transactions::list_transactions,
        commands::transactions::insert_transactions,
        commands::transactions::check_existing_fitids,
    ]);

    #[cfg(debug_assertions)]
    {
        let bindings_path = "../src/lib/bindings.ts";
        specta_builder
            .export(specta_typescript::Typescript::default(), bindings_path)
            .expect("failed to export TS bindings");

        let contents =
            std::fs::read_to_string(bindings_path).expect("failed to read generated bindings");
        if !contents.starts_with("// @ts-nocheck") {
            std::fs::write(bindings_path, format!("// @ts-nocheck\n{contents}"))
                .expect("failed to prepend ts-nocheck to bindings");
        }
    }

    tauri::Builder::default()
        .invoke_handler(specta_builder.invoke_handler())
        .setup(|app| {
            let database = db::init(app.handle()).expect("failed to initialize database");
            eprintln!("[finan] db at {}", database.path.display());
            app.manage(database);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 2: Boot dev server briefly to regenerate bindings**

```bash
pkill -f "tauri dev" 2>/dev/null; pkill -f "vite" 2>/dev/null; pkill -f "target/debug/finan" 2>/dev/null
pnpm tauri dev > /tmp/finan-dev.log 2>&1 &
# wait until bindings have all the new types
until grep -q "Transaction" src/lib/bindings.ts 2>/dev/null && grep -q "Account" src/lib/bindings.ts 2>/dev/null && grep -q "InsertResult" src/lib/bindings.ts 2>/dev/null && grep -q "checkExistingFitids" src/lib/bindings.ts 2>/dev/null; do sleep 2; done
echo "bindings ready"
```

Then verify content:

```bash
grep "export type" src/lib/bindings.ts
grep "async " src/lib/bindings.ts
```

Expected to find: `Account`, `NewAccount`, `Transaction`, `NewTransaction`, `InsertResult`, `HealthInfo` types AND `healthCheck`, `listAccounts`, `createOrGetAccount`, `listTransactions`, `insertTransactions`, `checkExistingFitids` async functions.

Kill server:

```bash
pkill -f "tauri dev"; pkill -f "vite"; pkill -f "target/debug/finan"
```

- [ ] **Step 3: Type-check**

```bash
pnpm check 2>&1 | tail -5
```
Expected: `0 ERRORS 0 WARNINGS`.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "$(cat <<'EOF'
feat(ipc): registra commands de accounts e transactions no specta builder

- 6 commands totais: health_check + 2 accounts + 3 transactions
- bindings.ts regenerado com Account, Transaction, NewTransaction, InsertResult
- pnpm check passa 0/0

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 5: Install `ofx-data-extractor` + parser wrapper + canonical fixture

**Files:**
- Modify: `package.json` (add dep)
- Create: `src/lib/ofx/types.ts`
- Create: `src/lib/ofx/parse.ts`
- Create: `src/lib/ofx/parse.test.ts`
- Create: `src/lib/ofx/__fixtures__/itau-minimal.ofx`

- [ ] **Step 1: Install the lib**

```bash
pnpm add ofx-data-extractor
```

> **Nota:** Esta lib é TS-first, modo strict/lenient. A API exata da v2+ pode mudar; o wrapper abaixo isola o resto do app dela. Se a importação não funcionar com `import { OfxData } from "ofx-data-extractor"`, leia `node_modules/ofx-data-extractor/dist/index.d.ts` ou `package.json` exports pra confirmar o ponto de entrada.

- [ ] **Step 2: Create `src/lib/ofx/types.ts`**

```ts
export interface ParsedOfx {
  account: ParsedAccount;
  transactions: ParsedTransaction[];
  summary: ParsedSummary;
}

export interface ParsedAccount {
  /** Banco normalizado: 'itau' | 'nubank' | 'bradesco' | 'unknown' */
  bank: string;
  /** ACCTID do OFX (id do banco pra essa conta) */
  ofxAcctid: string | null;
  /** BANKID quando presente */
  ofxBankid: string | null;
  /** FID/ORG do FI quando presente */
  ofxFid: string | null;
  /** Nome legível pra mostrar e armazenar (ex: "Itaú · ag 1234 cc 56789-0") */
  displayName: string;
}

export interface ParsedTransaction {
  /** ID único da transação no OFX (FITID) */
  fitid: string | null;
  /** ISO 8601 YYYY-MM-DD */
  date: string;
  /** Decimal como string (ex: "-123.45") */
  amount: string;
  /** Descrição (MEMO ou NAME do OFX) */
  description: string;
}

export interface ParsedSummary {
  /** Total de entradas (somatório dos amounts positivos) */
  totalIn: string;
  /** Total de saídas em valor absoluto */
  totalOut: string;
  /** Saldo líquido (in - out) */
  net: string;
  /** Data mais antiga, ISO 8601 */
  earliest: string | null;
  /** Data mais recente, ISO 8601 */
  latest: string | null;
}
```

- [ ] **Step 3: Create canonical fixture `src/lib/ofx/__fixtures__/itau-minimal.ofx`**

OFX format real (cabeçalho + body simplificado, válido pra parser typical):

```
OFXHEADER:100
DATA:OFXSGML
VERSION:102
SECURITY:NONE
ENCODING:USASCII
CHARSET:1252
COMPRESSION:NONE
OLDFILEUID:NONE
NEWFILEUID:NONE

<OFX>
<SIGNONMSGSRSV1>
<SONRS>
<STATUS>
<CODE>0
<SEVERITY>INFO
</STATUS>
<DTSERVER>20260412120000[-3:BRT]
<LANGUAGE>POR
<FI>
<ORG>Banco Itau
<FID>0341
</FI>
</SONRS>
</SIGNONMSGSRSV1>
<BANKMSGSRSV1>
<STMTTRNRS>
<TRNUID>0
<STATUS>
<CODE>0
<SEVERITY>INFO
</STATUS>
<STMTRS>
<CURDEF>BRL
<BANKACCTFROM>
<BANKID>0341
<BRANCHID>1234
<ACCTID>56789-0
<ACCTTYPE>CHECKING
</BANKACCTFROM>
<BANKTRANLIST>
<DTSTART>20260301000000[-3:BRT]
<DTEND>20260331000000[-3:BRT]
<STMTTRN>
<TRNTYPE>DEBIT
<DTPOSTED>20260305000000[-3:BRT]
<TRNAMT>-50.00
<FITID>ITAU0001
<MEMO>SUPERMERCADO ABC
</STMTTRN>
<STMTTRN>
<TRNTYPE>CREDIT
<DTPOSTED>20260310000000[-3:BRT]
<TRNAMT>3500.00
<FITID>ITAU0002
<MEMO>SALARIO
</STMTTRN>
<STMTTRN>
<TRNTYPE>DEBIT
<DTPOSTED>20260315000000[-3:BRT]
<TRNAMT>-29.90
<FITID>ITAU0003
<MEMO>NETFLIX
</STMTTRN>
</BANKTRANLIST>
<LEDGERBAL>
<BALAMT>3420.10
<DTASOF>20260331000000[-3:BRT]
</LEDGERBAL>
</STMTRS>
</STMTTRNRS>
</BANKMSGSRSV1>
</OFX>
```

- [ ] **Step 4: Create `src/lib/ofx/parse.ts`**

```ts
import { OfxData } from "ofx-data-extractor";
import type {
  ParsedAccount,
  ParsedOfx,
  ParsedSummary,
  ParsedTransaction,
} from "./types";

/**
 * Parse raw OFX text (already decoded) into our normalized shape.
 * Throws on malformed input. Bank detection is just heuristic on FID/ORG;
 * full normalization (encoding, bank-specific quirks) happens in normalize.ts.
 */
export function parseOfx(content: string): ParsedOfx {
  const ofx = OfxData.fromString(content);
  const rawAccount = ofx.getAccount?.() ?? ofx.getBankAccount?.();
  const rawTxs = ofx.getTransactions?.() ?? [];
  const rawSignon = ofx.getSignOn?.() ?? null;

  const account = extractAccount(rawAccount, rawSignon);
  const transactions: ParsedTransaction[] = rawTxs.map((t: unknown) => extractTx(t));
  const summary = computeSummary(transactions);

  return { account, transactions, summary };
}

function extractAccount(rawAccount: unknown, rawSignon: unknown): ParsedAccount {
  const a = rawAccount as Record<string, unknown> | null | undefined;
  const s = rawSignon as Record<string, unknown> | null | undefined;
  const fi = (s?.FI as Record<string, unknown> | undefined) ?? undefined;

  const bankid = (a?.BANKID as string | undefined) ?? null;
  const acctid = (a?.ACCTID as string | undefined) ?? null;
  const branchid = (a?.BRANCHID as string | undefined) ?? null;
  const fid = (fi?.FID as string | undefined) ?? null;
  const org = (fi?.ORG as string | undefined) ?? null;

  const bank = detectBank({ fid, org, bankid });
  const displayName = formatDisplayName(bank, branchid, acctid);

  return {
    bank,
    ofxAcctid: acctid,
    ofxBankid: bankid,
    ofxFid: fid,
    displayName,
  };
}

function extractTx(raw: unknown): ParsedTransaction {
  const t = raw as Record<string, unknown>;
  return {
    fitid: (t.FITID as string | undefined) ?? null,
    date: parseOfxDate((t.DTPOSTED as string | undefined) ?? ""),
    amount: String(t.TRNAMT ?? "0"),
    description: String(t.MEMO ?? t.NAME ?? ""),
  };
}

/** OFX dates: YYYYMMDDhhmmss[tz]. We only keep YYYY-MM-DD. */
function parseOfxDate(raw: string): string {
  const m = raw.match(/^(\d{4})(\d{2})(\d{2})/);
  if (!m) return "";
  return `${m[1]}-${m[2]}-${m[3]}`;
}

function detectBank(meta: {
  fid: string | null;
  org: string | null;
  bankid: string | null;
}): string {
  const fingerprint = [meta.fid, meta.org, meta.bankid]
    .filter(Boolean)
    .join(" ")
    .toLowerCase();
  if (/itau|341/.test(fingerprint)) return "itau";
  if (/bradesco|237/.test(fingerprint)) return "bradesco";
  if (/nubank|260|nu pagamentos|nu_pagamentos/.test(fingerprint)) return "nubank";
  if (/santander|033/.test(fingerprint)) return "santander";
  if (/inter|077/.test(fingerprint)) return "inter";
  if (/c6\b|c6 bank|336/.test(fingerprint)) return "c6";
  return "unknown";
}

function formatDisplayName(
  bank: string,
  branchid: string | null,
  acctid: string | null,
): string {
  const bankLabel = bank === "unknown" ? "Conta" : capitalize(bank);
  const parts = [bankLabel];
  if (branchid) parts.push(`ag ${branchid}`);
  if (acctid) parts.push(`cc ${acctid}`);
  return parts.join(" · ");
}

function capitalize(s: string): string {
  return s ? s.charAt(0).toUpperCase() + s.slice(1) : s;
}

function computeSummary(txs: ParsedTransaction[]): ParsedSummary {
  let totalIn = 0;
  let totalOut = 0;
  let earliest: string | null = null;
  let latest: string | null = null;
  for (const t of txs) {
    const n = Number(t.amount);
    if (Number.isFinite(n)) {
      if (n >= 0) totalIn += n;
      else totalOut += -n;
    }
    if (t.date) {
      if (earliest === null || t.date < earliest) earliest = t.date;
      if (latest === null || t.date > latest) latest = t.date;
    }
  }
  return {
    totalIn: totalIn.toFixed(2),
    totalOut: totalOut.toFixed(2),
    net: (totalIn - totalOut).toFixed(2),
    earliest,
    latest,
  };
}
```

> **Nota técnica:** o wrapper acima usa optional chaining nos métodos de `OfxData` porque a v2 da lib pode ter mudado nomes (`getAccount` vs `getBankAccount`). Se nenhum dos dois existir após `pnpm add`, leia `node_modules/ofx-data-extractor/dist/index.d.ts` e ajuste a chamada. O resto do app NÃO se importa com a forma exata da lib — só com `ParsedOfx`.

- [ ] **Step 5: Create `src/lib/ofx/parse.test.ts`**

```ts
import { describe, it, expect } from "vitest";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
import { parseOfx } from "./parse";

const __dirname = dirname(fileURLToPath(import.meta.url));
const itauFixture = readFileSync(
  join(__dirname, "__fixtures__/itau-minimal.ofx"),
  "utf-8",
);

describe("parseOfx — Itaú minimal fixture", () => {
  const result = parseOfx(itauFixture);

  it("detects Itaú as the bank", () => {
    expect(result.account.bank).toBe("itau");
  });

  it("extracts ACCTID and BANKID", () => {
    expect(result.account.ofxAcctid).toBe("56789-0");
    expect(result.account.ofxBankid).toBe("0341");
  });

  it("formats a sensible display name", () => {
    expect(result.account.displayName).toContain("Itau");
    expect(result.account.displayName).toContain("1234");
    expect(result.account.displayName).toContain("56789-0");
  });

  it("extracts 3 transactions", () => {
    expect(result.transactions).toHaveLength(3);
  });

  it("each transaction has fitid, date, amount, description", () => {
    for (const t of result.transactions) {
      expect(t.fitid).toMatch(/^ITAU\d+$/);
      expect(t.date).toMatch(/^\d{4}-\d{2}-\d{2}$/);
      expect(typeof t.amount).toBe("string");
      expect(t.description.length).toBeGreaterThan(0);
    }
  });

  it("summary aggregates in/out/net correctly", () => {
    // out = 50.00 + 29.90 = 79.90; in = 3500.00; net = 3420.10
    expect(result.summary.totalIn).toBe("3500.00");
    expect(result.summary.totalOut).toBe("79.90");
    expect(result.summary.net).toBe("3420.10");
  });

  it("summary captures date range", () => {
    expect(result.summary.earliest).toBe("2026-03-05");
    expect(result.summary.latest).toBe("2026-03-15");
  });
});
```

- [ ] **Step 6: Run tests**

```bash
pnpm test 2>&1 | tail -15
```
Expected: 11 tests pass (4 prior + 7 new from parse.test).

If a test fails because `ofx-data-extractor` exposes a different API than assumed:
- Inspect `node_modules/ofx-data-extractor/dist/index.d.ts` to find the real method names.
- Adjust `parse.ts` (only `parseOfx`'s top calls — the wrapper boundary). Tests should keep passing because they assert on our `ParsedOfx` shape, not on the lib's shape.

- [ ] **Step 7: Type-check**

```bash
pnpm check 2>&1 | tail -3
```
Expected: 0 errors.

- [ ] **Step 8: Commit**

```bash
git add package.json pnpm-lock.yaml src/lib/ofx/
git commit -m "$(cat <<'EOF'
feat(ofx): parser wrapper + canonical Itaú fixture

- ofx-data-extractor instalado
- lib/ofx/types.ts: ParsedOfx, ParsedAccount, ParsedTransaction, ParsedSummary
- lib/ofx/parse.ts: wrapper que isola o app da API da lib externa
- detectBank por fingerprint FID/ORG/BANKID (Itaú, Bradesco, Nubank, Santander, Inter, C6)
- summary: totalIn, totalOut, net, earliest, latest
- fixture sintética itau-minimal.ofx + 7 testes Vitest

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 6: Encoding + bank-specific normalization

**Files:**
- Create: `src/lib/ofx/normalize.ts`
- Create: `src/lib/ofx/normalize.test.ts`

OFX brasileiro normalmente vem em ISO-8859-1 (Latin-1) — `FileReader.readAsText()` default usa UTF-8 e BORRA acentos. Esta camada decodifica corretamente.

- [ ] **Step 1: Create `src/lib/ofx/normalize.ts`**

```ts
/**
 * Decode an OFX file's bytes into a string using the encoding declared in the
 * OFX header (or sensible fallback for Brazilian banks). Returns the decoded
 * content ready for parseOfx().
 */
export async function decodeOfxFile(file: File): Promise<string> {
  const buf = await file.arrayBuffer();
  const headerAscii = new TextDecoder("ascii", { fatal: false }).decode(
    buf.slice(0, Math.min(buf.byteLength, 512)),
  );
  const encoding = detectEncoding(headerAscii);
  return new TextDecoder(encoding, { fatal: false }).decode(buf);
}

/**
 * Inspect the OFX header lines to find the declared CHARSET.
 * Returns a TextDecoder-compatible encoding name.
 */
export function detectEncoding(header: string): string {
  // OFXHEADER format: CHARSET:1252 or ENCODING:USASCII
  const charsetMatch = header.match(/^CHARSET:\s*(\S+)/im);
  const encodingMatch = header.match(/^ENCODING:\s*(\S+)/im);

  const charset = charsetMatch?.[1]?.toLowerCase();
  const encoding = encodingMatch?.[1]?.toLowerCase();

  if (charset === "utf-8" || encoding === "utf-8") return "utf-8";
  if (charset === "1252" || charset === "windows-1252") return "windows-1252";
  if (charset === "8859-1" || charset === "iso-8859-1") return "iso-8859-1";
  if (encoding === "usascii" || encoding === "us-ascii") return "windows-1252";

  // Brazilian banks frequently lie or omit; windows-1252 is the safest
  // superset of ASCII for Latin Portuguese accents.
  return "windows-1252";
}

/**
 * Apply bank-specific text fixups after parsing.
 * Currently a passthrough — extension point for fase 2+ when we encounter
 * concrete real-world quirks (e.g. Nubank embedding extra <STMTTRN> attrs).
 */
export function normalizeTransactionText(description: string): string {
  return description
    .replace(/\s+/g, " ")
    .trim();
}
```

- [ ] **Step 2: Create `src/lib/ofx/normalize.test.ts`**

```ts
import { describe, it, expect } from "vitest";
import { detectEncoding, normalizeTransactionText } from "./normalize";

describe("detectEncoding", () => {
  it("detects CHARSET:1252 as windows-1252", () => {
    const header = `OFXHEADER:100
DATA:OFXSGML
VERSION:102
CHARSET:1252`;
    expect(detectEncoding(header)).toBe("windows-1252");
  });

  it("detects CHARSET:8859-1 as iso-8859-1", () => {
    expect(detectEncoding("CHARSET:8859-1")).toBe("iso-8859-1");
  });

  it("detects ENCODING:UTF-8", () => {
    expect(detectEncoding("ENCODING:UTF-8")).toBe("utf-8");
  });

  it("falls back to windows-1252 when ENCODING:USASCII is declared", () => {
    expect(detectEncoding("ENCODING:USASCII")).toBe("windows-1252");
  });

  it("defaults to windows-1252 when no header info is present", () => {
    expect(detectEncoding("")).toBe("windows-1252");
  });
});

describe("normalizeTransactionText", () => {
  it("collapses whitespace and trims", () => {
    expect(normalizeTransactionText("  SUPER\tMERCADO   ABC  ")).toBe(
      "SUPER MERCADO ABC",
    );
  });

  it("handles empty input", () => {
    expect(normalizeTransactionText("")).toBe("");
  });

  it("preserves Portuguese accents", () => {
    expect(normalizeTransactionText("Saída de R$ 50")).toBe("Saída de R$ 50");
  });
});
```

- [ ] **Step 3: Run tests**

```bash
pnpm test 2>&1 | tail -15
```
Expected: 19 tests pass (11 prior + 8 new).

- [ ] **Step 4: Commit**

```bash
git add src/lib/ofx/normalize.ts src/lib/ofx/normalize.test.ts
git commit -m "$(cat <<'EOF'
feat(ofx): encoding + text normalization

- decodeOfxFile lê o File e usa TextDecoder com encoding detectado no header
- detectEncoding cobre CHARSET:1252, 8859-1, UTF-8, USASCII; default windows-1252
  (superset seguro de ASCII pros acentos do português)
- normalizeTransactionText colapsa whitespace e trim
- 8 testes Vitest

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 7: TS API wrappers (accounts + transactions)

**Files:**
- Create: `src/lib/api/accounts.ts`
- Create: `src/lib/api/transactions.ts`

- [ ] **Step 1: Create `src/lib/api/accounts.ts`**

```ts
import { commands } from "../bindings";
import type { Account, NewAccount } from "../bindings";

function unwrap<T>(result: { status: "ok"; data: T } | { status: "error"; error: string }): T {
  if (result.status === "error") throw new Error(result.error);
  return result.data;
}

export async function listAccounts(): Promise<Account[]> {
  return unwrap(await commands.listAccounts());
}

export async function createOrGetAccount(input: NewAccount): Promise<Account> {
  return unwrap(await commands.createOrGetAccount(input));
}
```

- [ ] **Step 2: Create `src/lib/api/transactions.ts`**

```ts
import { commands } from "../bindings";
import type { InsertResult, NewTransaction, Transaction } from "../bindings";

function unwrap<T>(result: { status: "ok"; data: T } | { status: "error"; error: string }): T {
  if (result.status === "error") throw new Error(result.error);
  return result.data;
}

export async function listTransactions(accountId: number | null = null): Promise<Transaction[]> {
  return unwrap(await commands.listTransactions(accountId));
}

export async function insertTransactions(
  accountId: number,
  txs: NewTransaction[],
): Promise<InsertResult> {
  return unwrap(await commands.insertTransactions(accountId, txs));
}

export async function checkExistingFitids(
  accountId: number,
  fitids: string[],
): Promise<string[]> {
  return unwrap(await commands.checkExistingFitids(accountId, fitids));
}
```

- [ ] **Step 3: Type-check**

```bash
pnpm check 2>&1 | tail -5
```
Expected: 0 errors.

- [ ] **Step 4: Commit**

```bash
git add src/lib/api/
git commit -m "$(cat <<'EOF'
feat(api): wrappers TS pra accounts e transactions commands

- listAccounts, createOrGetAccount
- listTransactions, insertTransactions, checkExistingFitids
- unwrap() centraliza desempacotamento do Result<T, string> do tauri-specta

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 8: DropZone component

**Files:**
- Create: `src/lib/components/import/DropZone.svelte`

- [ ] **Step 1: Create `src/lib/components/import/DropZone.svelte`**

```svelte
<script lang="ts">
  import { decodeOfxFile } from "$lib/ofx/normalize";
  import { parseOfx } from "$lib/ofx/parse";
  import type { ParsedOfx } from "$lib/ofx/types";

  type Props = {
    onparsed?: (result: { file: File; parsed: ParsedOfx }) => void;
    onerror?: (message: string) => void;
  };

  let { onparsed, onerror }: Props = $props();

  let active = $state(false);
  let busy = $state(false);
  let fileInput: HTMLInputElement | undefined = $state();

  async function handleFile(file: File) {
    busy = true;
    try {
      const content = await decodeOfxFile(file);
      const parsed = parseOfx(content);
      onparsed?.({ file, parsed });
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      onerror?.(msg);
    } finally {
      busy = false;
    }
  }

  function onDragOver(e: DragEvent) {
    e.preventDefault();
    active = true;
  }

  function onDragLeave(e: DragEvent) {
    e.preventDefault();
    active = false;
  }

  async function onDrop(e: DragEvent) {
    e.preventDefault();
    active = false;
    const file = e.dataTransfer?.files?.[0];
    if (file) await handleFile(file);
  }

  async function onFilePicked(e: Event) {
    const input = e.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    if (file) await handleFile(file);
  }
</script>

<div
  role="button"
  tabindex="0"
  ondragover={onDragOver}
  ondragleave={onDragLeave}
  ondrop={onDrop}
  onclick={() => fileInput?.click()}
  onkeydown={(e) => (e.key === "Enter" || e.key === " ") && fileInput?.click()}
  class="rounded-xl border border-dashed p-9 flex flex-col items-center gap-3 cursor-pointer transition-colors text-center
         {active ? 'border-accent bg-accent-soft' : 'border-border bg-surface hover:bg-surface-2'}"
>
  <div class="w-14 h-14 rounded-2xl grid place-items-center"
       style="background: var(--color-surface-2); border: 1px solid var(--color-border); color: var(--color-accent-hi)">
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7"
         stroke-linecap="round" stroke-linejoin="round" class="w-7 h-7">
      <path d="M12 4v11m0 0 4-4m-4 4-4-4" />
      <path d="M4 17v2.5A1.5 1.5 0 0 0 5.5 21h13a1.5 1.5 0 0 0 1.5-1.5V17" />
    </svg>
  </div>

  <h3 class="text-base font-semibold tracking-tight" style="font-family: var(--font-display)">
    Arraste seu extrato OFX
  </h3>
  <p class="text-fg-muted text-xs max-w-sm">
    Exporte o extrato mensal do seu banco (Itaú, Nubank, Bradesco, etc.) no formato
    <strong class="text-fg">.ofx</strong> e solte aqui — ou clique pra escolher um arquivo.
  </p>

  {#if busy}
    <p class="text-fg-faint text-xs mt-2">Lendo arquivo…</p>
  {/if}

  <input
    bind:this={fileInput}
    type="file"
    accept=".ofx,.OFX,application/x-ofx,text/plain"
    onchange={onFilePicked}
    class="hidden"
  />
</div>
```

- [ ] **Step 2: Type-check**

```bash
pnpm check 2>&1 | tail -5
```
Expected: 0 errors.

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/import/
git commit -m "$(cat <<'EOF'
feat(ui): DropZone component (drag-and-drop + click-to-pick OFX)

- decodeOfxFile + parseOfx integrados
- callbacks tipados onparsed/onerror
- active state + busy state visual
- accessibility: role=button, keyboard activation (Enter/Space)

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 9: Onboarding com DropZone real

**Files:**
- Modify: `src/routes/Onboarding.svelte`

- [ ] **Step 1: Replace `src/routes/Onboarding.svelte`**

```svelte
<script lang="ts">
  import DropZone from "$lib/components/import/DropZone.svelte";
  import type { ParsedOfx } from "$lib/ofx/types";
  import { push } from "svelte-spa-router";

  let error = $state<string | null>(null);

  function onparsed(detail: { file: File; parsed: ParsedOfx }) {
    // Persist temporarily on window so /import can pick it up.
    // (Svelte stores would be cleaner; this is fine for MVP.)
    (window as unknown as { __finanPending?: typeof detail }).__finanPending = detail;
    push("/import");
  }

  function onerror(msg: string) {
    error = msg;
  }
</script>

<section class="p-10 max-w-xl mx-auto flex flex-col gap-6">
  <header class="text-center flex flex-col gap-2">
    <h1 class="text-3xl font-semibold tracking-tight" style="font-family: var(--font-display)">
      Suas finanças, no seu Mac.
    </h1>
    <p class="text-fg-muted text-sm max-w-md mx-auto leading-relaxed">
      Sem nuvem, sem login, sem assinatura. Você arrasta o extrato OFX
      do seu banco e o finan organiza tudo num arquivo SQLite local.
    </p>
  </header>

  <DropZone {onparsed} {onerror} />

  {#if error}
    <div class="rounded-lg border border-border bg-surface p-3 text-sm text-neg">
      Erro ao ler o arquivo: {error}
    </div>
  {/if}

  <p class="text-fg-faint text-xs text-center">
    Seus dados ficam em <span class="font-mono text-fg-muted">~/Library/Application Support/app.finan/finan.db</span>
  </p>
</section>
```

> **Nota arquitetural:** uso `window.__finanPending` como "hand-off" entre rotas temporariamente. Pra um app pequeno, é mais simples que montar uma store global agora. Em fase 5 (polish) podemos refatorar pra Svelte store em `$lib/stores/import.svelte.ts`.

- [ ] **Step 2: Type-check**

```bash
pnpm check 2>&1 | tail -5
```
Expected: 0 errors.

- [ ] **Step 3: Smoke test — visual**

Não conseguimos verificar drag-and-drop sem interação real. Boot dev, deixe a janela abrir, valide que `/onboarding` renderiza com DropZone (sem erros no console do Vite):

```bash
pkill -f "tauri dev" 2>/dev/null; pkill -f "vite" 2>/dev/null
pnpm tauri dev > /tmp/finan-dev.log 2>&1 &
until lsof -iTCP:1420 -sTCP:LISTEN >/dev/null 2>&1; do sleep 2; done
grep -i "error" /tmp/finan-dev.log | head -5
pkill -f "tauri dev"; pkill -f "vite"; pkill -f "target/debug/finan"
```

Expected: nenhum erro de compilação. (Tests cobrem a lógica do parser; UI funcional precisa de teste manual depois.)

- [ ] **Step 4: Commit**

```bash
git add src/routes/Onboarding.svelte
git commit -m "$(cat <<'EOF'
feat(onboarding): DropZone real + redirect pra /import

- Onboarding renderiza DropZone que parseia OFX no drop/click
- Em sucesso, salva resultado em window.__finanPending (handoff simples
  pré-store global) e push pra /import
- Em erro, mostra mensagem inline

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 10: Import route com preview, dedup check, commit

**Files:**
- Create: `src/lib/components/import/ImportPreview.svelte`
- Modify: `src/routes/Import.svelte`

- [ ] **Step 1: Create `src/lib/components/import/ImportPreview.svelte`**

```svelte
<script lang="ts">
  import { formatMoney } from "$lib/format/money";
  import type { ParsedTransaction } from "$lib/ofx/types";

  type Props = {
    transactions: ParsedTransaction[];
    duplicateFitids: Set<string>;
    selected: Set<string>;
    ontoggle: (fitid: string) => void;
    ontoggleAll: (checked: boolean) => void;
  };

  let { transactions, duplicateFitids, selected, ontoggle, ontoggleAll }: Props = $props();

  let allChecked = $derived(
    transactions.length > 0 && transactions.every((t) => !t.fitid || selected.has(t.fitid)),
  );
</script>

<div class="rounded-lg border border-border-subtle bg-surface overflow-hidden">
  <table class="w-full text-[12px]">
    <thead class="bg-surface-2">
      <tr>
        <th class="text-left px-3 py-2 w-8">
          <input
            type="checkbox"
            checked={allChecked}
            onchange={(e) => ontoggleAll((e.currentTarget as HTMLInputElement).checked)}
          />
        </th>
        <th class="text-left px-3 py-2 font-medium text-fg-faint uppercase tracking-wider text-[10.5px]">Data</th>
        <th class="text-left px-3 py-2 font-medium text-fg-faint uppercase tracking-wider text-[10.5px]">Descrição</th>
        <th class="text-right px-3 py-2 font-medium text-fg-faint uppercase tracking-wider text-[10.5px]">Valor</th>
      </tr>
    </thead>
    <tbody>
      {#each transactions as t (t.fitid ?? `${t.date}-${t.amount}-${t.description}`)}
        {@const isDup = !!(t.fitid && duplicateFitids.has(t.fitid))}
        {@const isSel = !!(t.fitid && selected.has(t.fitid))}
        <tr class="border-t border-border-subtle {isDup ? 'opacity-60' : ''}">
          <td class="px-3 py-2">
            <input
              type="checkbox"
              checked={isSel}
              disabled={!t.fitid}
              onchange={() => t.fitid && ontoggle(t.fitid)}
            />
          </td>
          <td class="px-3 py-2 text-fg-muted tabular">{t.date}</td>
          <td class="px-3 py-2">
            {t.description}
            {#if isDup}
              <span class="ml-2 text-[10px] text-fg-faint uppercase tracking-wider">duplicada</span>
            {/if}
          </td>
          <td class="px-3 py-2 text-right tabular font-medium {Number(t.amount) >= 0 ? 'text-pos' : 'text-fg'}">
            {formatMoney(t.amount)}
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>
```

- [ ] **Step 2: Replace `src/routes/Import.svelte`**

```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import { push } from "svelte-spa-router";
  import { Button } from "$lib/components/ui/button";
  import DropZone from "$lib/components/import/DropZone.svelte";
  import ImportPreview from "$lib/components/import/ImportPreview.svelte";
  import { formatMoney } from "$lib/format/money";
  import { createOrGetAccount } from "$lib/api/accounts";
  import { checkExistingFitids, insertTransactions } from "$lib/api/transactions";
  import type { ParsedOfx } from "$lib/ofx/types";
  import type { Account, NewTransaction } from "$lib/bindings";

  type PendingImport = { file: File; parsed: ParsedOfx };

  let pending = $state<PendingImport | null>(null);
  let account = $state<Account | null>(null);
  let duplicateFitids = $state<Set<string>>(new Set());
  let selected = $state<Set<string>>(new Set());
  let busy = $state(false);
  let error = $state<string | null>(null);

  onMount(() => {
    const stash = (window as unknown as { __finanPending?: PendingImport }).__finanPending;
    if (stash) {
      pending = stash;
      (window as unknown as { __finanPending?: PendingImport }).__finanPending = undefined;
      void prepareImport(stash.parsed);
    }
  });

  async function prepareImport(parsed: ParsedOfx) {
    busy = true;
    try {
      account = await createOrGetAccount({
        name: parsed.account.displayName,
        bank: parsed.account.bank === "unknown" ? null : parsed.account.bank,
        ofx_acctid: parsed.account.ofxAcctid,
      });
      const fitids = parsed.transactions
        .map((t) => t.fitid)
        .filter((f): f is string => !!f);
      const existing = await checkExistingFitids(account.id, fitids);
      duplicateFitids = new Set(existing);
      selected = new Set(fitids.filter((f) => !duplicateFitids.has(f)));
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }

  function onparsed(detail: PendingImport) {
    pending = detail;
    void prepareImport(detail.parsed);
  }
  function onerror(msg: string) {
    error = msg;
  }

  function toggle(fitid: string) {
    if (selected.has(fitid)) selected.delete(fitid);
    else selected.add(fitid);
    selected = new Set(selected);
  }

  function toggleAll(checked: boolean) {
    if (!pending) return;
    selected = checked
      ? new Set(pending.parsed.transactions.map((t) => t.fitid).filter((f): f is string => !!f))
      : new Set();
  }

  async function confirmImport() {
    if (!pending || !account) return;
    busy = true;
    error = null;
    try {
      const toInsert: NewTransaction[] = pending.parsed.transactions
        .filter((t) => t.fitid && selected.has(t.fitid))
        .map((t) => ({
          date: t.date,
          amount: t.amount,
          description: t.description,
          ofx_fitid: t.fitid,
        }));
      const result = await insertTransactions(account.id, toInsert);
      console.log("[finan] import:", result);
      push("/transactions");
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }
</script>

<section class="p-8 max-w-5xl mx-auto flex flex-col gap-5">
  <header class="flex items-baseline justify-between">
    <h2 class="text-xl font-semibold tracking-tight" style="font-family: var(--font-display)">
      Importar OFX
    </h2>
    {#if pending}
      <span class="text-xs text-fg-faint tabular">{pending.file.name}</span>
    {/if}
  </header>

  {#if !pending}
    <DropZone {onparsed} {onerror} />
  {:else}
    {@const p = pending.parsed}
    <div class="grid grid-cols-[1fr_280px] gap-4 items-start">
      <div class="flex flex-col gap-3">
        <div class="rounded-lg border border-border-subtle bg-surface p-4 flex items-center gap-3">
          <div class="text-[10.5px] uppercase tracking-wider font-semibold text-accent-hi bg-accent-soft border border-accent/30 rounded-full px-2 py-0.5">
            {p.account.bank === "unknown" ? "Desconhecido" : p.account.bank}
          </div>
          <div class="text-sm font-medium">{p.account.displayName}</div>
          <div class="ml-auto text-xs text-fg-faint tabular">
            {p.transactions.length} transações ·
            {p.summary.earliest ?? "?"} → {p.summary.latest ?? "?"}
          </div>
        </div>

        <ImportPreview
          transactions={p.transactions}
          {duplicateFitids}
          {selected}
          ontoggle={toggle}
          ontoggleAll={toggleAll}
        />
      </div>

      <aside class="rounded-lg border border-border-subtle bg-surface p-4 flex flex-col gap-2 text-[12px]">
        <div class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint mb-1">Resumo</div>
        <div class="flex justify-between"><span class="text-fg-muted">Entradas</span><span class="tabular text-pos">{formatMoney(p.summary.totalIn)}</span></div>
        <div class="flex justify-between"><span class="text-fg-muted">Saídas</span><span class="tabular">{formatMoney(p.summary.totalOut)}</span></div>
        <div class="flex justify-between border-t border-border-subtle pt-2 mt-1"><span class="text-fg-muted">Líquido</span><span class="tabular font-semibold">{formatMoney(p.summary.net)}</span></div>
        <div class="flex justify-between mt-2"><span class="text-fg-muted">Selecionadas</span><span class="tabular">{selected.size}</span></div>
        <div class="flex justify-between"><span class="text-fg-muted">Duplicadas</span><span class="tabular text-fg-faint">{duplicateFitids.size}</span></div>
      </aside>
    </div>

    {#if error}
      <div class="rounded-lg border border-border bg-surface p-3 text-sm text-neg">{error}</div>
    {/if}

    <div class="flex justify-end gap-2 sticky bottom-0 bg-bg pt-3 border-t border-border-subtle">
      <Button variant="ghost" onclick={() => { pending = null; account = null; selected = new Set(); duplicateFitids = new Set(); }}>Cancelar</Button>
      <Button onclick={confirmImport} disabled={busy || selected.size === 0}>
        {busy ? "Importando…" : `Importar ${selected.size} ${selected.size === 1 ? "transação" : "transações"}`}
      </Button>
    </div>
  {/if}
</section>
```

- [ ] **Step 3: Type-check**

```bash
pnpm check 2>&1 | tail -5
```
Expected: 0 errors.

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/import/ImportPreview.svelte src/routes/Import.svelte
git commit -m "$(cat <<'EOF'
feat(import): preview com dedup check, summary, confirmação

- ImportPreview.svelte: tabela com checkbox, badge "duplicada" pros fitids existentes
- /import recupera handoff de Onboarding ou aceita drop direto
- chama createOrGetAccount → checkExistingFitids → insertTransactions
- summary card (entradas/saídas/líquido + selecionadas/duplicadas)
- pós-import, push pra /transactions

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 11: TxTable component + Transactions route

**Files:**
- Create: `src/lib/components/transactions/TxTable.svelte`
- Modify: `src/routes/Transactions.svelte`

- [ ] **Step 1: Create `src/lib/components/transactions/TxTable.svelte`**

```svelte
<script lang="ts">
  import { formatMoney } from "$lib/format/money";
  import type { Transaction } from "$lib/bindings";

  let { transactions }: { transactions: Transaction[] } = $props();
</script>

<div class="rounded-lg border border-border-subtle bg-surface overflow-hidden">
  <table class="w-full text-[12px]">
    <thead class="bg-surface-2">
      <tr>
        <th class="text-left px-4 py-2 font-medium text-fg-faint uppercase tracking-wider text-[10.5px] w-[100px]">Data</th>
        <th class="text-left px-4 py-2 font-medium text-fg-faint uppercase tracking-wider text-[10.5px]">Descrição</th>
        <th class="text-right px-4 py-2 font-medium text-fg-faint uppercase tracking-wider text-[10.5px] w-[140px]">Valor</th>
      </tr>
    </thead>
    <tbody>
      {#each transactions as t (t.id)}
        <tr class="border-t border-border-subtle hover:bg-hover">
          <td class="px-4 py-2.5 text-fg-muted tabular">{t.date}</td>
          <td class="px-4 py-2.5">{t.description}</td>
          <td class="px-4 py-2.5 text-right tabular font-medium {Number(t.amount) >= 0 ? 'text-pos' : 'text-fg'}">
            {formatMoney(t.amount)}
          </td>
        </tr>
      {:else}
        <tr>
          <td colspan="3" class="px-4 py-10 text-center text-fg-faint">
            Nenhuma transação ainda. <a href="#/import" class="text-accent hover:underline">Importar um OFX</a>?
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>
```

- [ ] **Step 2: Replace `src/routes/Transactions.svelte`**

```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import TxTable from "$lib/components/transactions/TxTable.svelte";
  import { listTransactions } from "$lib/api/transactions";
  import type { Transaction } from "$lib/bindings";

  let transactions = $state<Transaction[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      transactions = await listTransactions();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  });
</script>

<section class="p-8 max-w-5xl mx-auto flex flex-col gap-5">
  <header class="flex items-baseline justify-between">
    <h2 class="text-xl font-semibold tracking-tight" style="font-family: var(--font-display)">
      Transações
    </h2>
    <span class="text-xs text-fg-faint tabular">
      {transactions.length} {transactions.length === 1 ? "transação" : "transações"}
    </span>
  </header>

  {#if loading}
    <div class="text-fg-faint text-sm">Carregando…</div>
  {:else if error}
    <div class="rounded-lg border border-border bg-surface p-3 text-sm text-neg">{error}</div>
  {:else}
    <TxTable {transactions} />
  {/if}
</section>
```

- [ ] **Step 3: Type-check**

```bash
pnpm check 2>&1 | tail -5
```
Expected: 0 errors.

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/transactions/ src/routes/Transactions.svelte
git commit -m "$(cat <<'EOF'
feat(transactions): TxTable component + listagem em /transactions

- TxTable: tabela simples (data, descrição, valor), empty state com link pra import
- /transactions carrega listTransactions() no mount
- valor positivo em verde (--color-pos), negativo no fg padrão

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 12: Acceptance verification + Fase 1 close

**Files:**
- (verificação só)
- Possivelmente: Modify: `README.md` (atualiza seção Status)

- [ ] **Step 1: Full test suite**

```bash
pnpm check && cd src-tauri && cargo test --lib && cargo clippy --all-targets -- -D warnings && cargo fmt --check && cd .. && pnpm test
```

Expected: pnpm check 0 erros / 16 cargo tests passing / clippy clean / fmt clean / 19 vitest passing.

- [ ] **Step 2: Full smoke test (manual com OFX real)**

Boot dev server:
```bash
pkill -f "tauri dev" 2>/dev/null; pkill -f "vite" 2>/dev/null
rm -f ~/Library/Application\ Support/app.finan/finan.db
pnpm tauri dev > /tmp/finan-dev.log 2>&1 &
until lsof -iTCP:1420 -sTCP:LISTEN >/dev/null 2>&1; do sleep 2; done
echo "ready"
```

Manual checklist (precisa de um arquivo OFX real do seu banco; sem ele, valide ao menos parser+commands com o fixture sintético):

- [ ] Abre em `/dashboard` (DB recriada → contagem de categorias = 9).
- [ ] Click sidebar `Onboarding` → vê DropZone.
- [ ] Arrasta um OFX real do seu banco → redireciona pra `/import`.
- [ ] `/import` mostra: badge do banco detectado, displayName, contagem, período, summary com entradas/saídas/líquido.
- [ ] Tabela lista todas as transações; nenhuma marcada como duplicada (primeira importação); todas selecionadas por padrão.
- [ ] Clica `Importar N transações`.
- [ ] Redireciona pra `/transactions`; lista vê as N transações.
- [ ] Volta pra `/onboarding`, arrasta o MESMO OFX.
- [ ] `/import` agora marca TODAS como `duplicada` (badge), nenhuma selecionada por padrão; summary mostra `Duplicadas: N`.
- [ ] Botão `Importar` fica desabilitado (selecionadas = 0).
- [ ] `cmd+Q` fecha sem erros.

Kill server:
```bash
pkill -f "tauri dev"; pkill -f "vite"; pkill -f "target/debug/finan"
```

- [ ] **Step 3: Verify DB content**

```bash
sqlite3 ~/Library/Application\ Support/app.finan/finan.db "SELECT COUNT(*) AS accounts FROM accounts; SELECT COUNT(*) AS transactions FROM transactions; SELECT id, bank, ofx_acctid FROM accounts; SELECT date, amount, description, ofx_fitid FROM transactions ORDER BY date DESC LIMIT 5;"
```
Expected: 1+ account, N+ transactions, no duplicates por (account_id, ofx_fitid).

- [ ] **Step 4: Update README status**

Edit `README.md`:
```
- ✅ Fase 0 — Scaffold (Tauri + Svelte + DB + sidebar + IPC tipado)
- ✅ Fase 1 — Importar OFX (parser + dedup + listagem)
- 🚧 Fase 2 — Categorização manual (próximo)
- ⏳ Fase 3-5 — Regras, dashboard, polish
```

- [ ] **Step 5: Closing commit**

```bash
git add README.md
git commit -m "$(cat <<'EOF'
chore(fase-1): close OFX import phase — acceptance criteria batem

- pnpm tauri dev: drag OFX → preview → confirmar → vê transações em /transactions
- Reimport do mesmo OFX: todas marcadas duplicadas, nenhuma reinserida
- DB tem accounts (auto-criadas via OFX header) + transactions sem dups
- Tests: 16 cargo / 19 vitest / pnpm check 0 / clippy/fmt limpos

Próximo: plano da Fase 2 (Categorização manual inline + filtros).

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

- [ ] **Step 6: (Opcional) Tag**

```bash
git tag -a v0.2.0-import -m "Fase 1 OFX import complete"
```

---

## Self-Review

### Spec coverage

| Spec item | Task |
|---|---|
| §3 fronteiras (UI nunca toca SQL, IPC tipado) | T2-T4, T7 |
| §5 schema (account auto-criada via OFX_ACCTID, UNIQUE dedup) | T2, T3 |
| §5 valores monetários (Decimal/string IPC) | T1, T3, T7 |
| §8.2 detecção bank (badge) | T5, T10 |
| §8.2 preview com checkbox + duplicada marcada | T10 |
| §8.2 summary card (entradas, saídas, líquido, novas vs dups) | T10 |
| §8.2 footer "Importar N transações" + Cancelar | T10 |
| §8.2 pós-import redirect | T10 → /transactions |
| §8.2 dedup por (account_id, ofx_fitid) | T3 |
| §6 OFX brasileiro: encoding ISO-8859-1 (windows-1252) | T6 |
| §8.3 lista crua de transações | T11 |
| §10 tests Vitest + cargo | T1, T3, T5, T6 |
| §11 nada de network | (mantido — nenhum fetch externo) |

### Placeholder scan

Reli o plano. Nenhum "TBD"/"TODO"/"add error handling"/"similar to". Cada step tem código completo.

### Type consistency

- `NewAccount { name, bank, ofx_acctid }` — usado em T2 (Rust) e T10 (TS via bindings). Field names match.
- `NewTransaction { date, amount, description, ofx_fitid }` — T1, T3 (Rust), T10 (TS). Match.
- `InsertResult { inserted, skipped_duplicates }` — T1 (Rust), T10 console.log (TS). Match.
- `ParsedOfx`, `ParsedAccount`, `ParsedTransaction`, `ParsedSummary` — TS only, defined T5, used T6, T8, T9, T10. Consistent.
- `commands.listAccounts`, `commands.createOrGetAccount`, `commands.listTransactions`, `commands.insertTransactions`, `commands.checkExistingFitids` — camelCase TS names auto-generated by tauri-specta from snake_case Rust names. Consistent.

### Risks documented inline

- `ofx-data-extractor` API uncertainty (T5 step 4 nota: implementer reads `.d.ts` if signature differs).
- macOS file drop into Tauri WebView: standard HTML5 drag-drop should work; if Tauri intercepts, fallback to `<input type="file">` click in T8 (already implemented).
- Sticky-bottom action bar might overlay content if list is short — acceptable trade-off; spec §8.2 calls for it.
