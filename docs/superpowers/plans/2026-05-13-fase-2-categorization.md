# Fase 2 — Categorização manual + filtros Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Tornar `/transactions` usável pra categorizar um mês inteiro de transações em poucos minutos. Usuário clica numa célula "Categoria" → popover abre com lista pesquisável + opção "Criar nova"; seleciona → grava direto no SQLite via command tipado. Toolbar tem filtros por mês (default = mês atual) e categoria. Clicar na linha abre painel lateral com notes editáveis. Store de filtros persiste durante a sessão.

**Architecture:** State global em `$lib/stores/filters.svelte.ts` (Svelte 5 rune store) mantém mês + categoria selecionados. `CategoryPicker.svelte` é um popover headless com input de busca + lista navegável por teclado (↑↓/Enter/Esc). Commands Rust novos: `list_categories`, `create_category`, `update_transaction_category`, `update_transaction_notes`. Filtragem no backend via `list_transactions(filters)` aceitando `month` (`YYYY-MM`) e `category_id` opcionais.

**Tech Stack:** já instalado: Tauri + Svelte 5 runes + Tailwind 4 + shadcn-svelte (Button, popover virá nesta fase) + tauri-specta. Nada novo a instalar (popover usa bits-ui que já temos).

**Acceptance criteria (Fase 2):**
1. `/transactions` mostra coluna "Categoria" pra cada transação.
2. Clicar na célula abre popover com lista de categorias seed (Mercado, Restaurante, …). Busca por texto + ↑↓ + Enter funciona.
3. Selecionar uma categoria persiste imediatamente (chama `update_transaction_category`) e fecha o popover. Próxima ação do usuário (clicar/Tab) deve sentir rápida (<200ms).
4. "Criar nova categoria" no popover cria a categoria + atribui à transação na mesma ação.
5. Toolbar tem chips "Mês" + "Categoria" com selectores. Mudar filtro re-roda `listTransactions` e a tabela atualiza.
6. Estado dos filtros sobrevive a navegação entre rotas dentro da sessão.
7. Clicar numa linha abre painel lateral (overlay ou drawer) mostrando description + notes editáveis. Salvar grava via `update_transaction_notes`.
8. Tests: `cargo test --lib` ≥ 21 (16 anteriores + 5 novos), `pnpm test` ≥ 19 (sem novos obrigatórios), `pnpm check` 0, clippy/fmt limpos.

**Out of scope (próximas fases):**
- Regras automáticas (fase 3)
- CRUD completo de categorias em `/categories` (fase 3 ou 5)
- Search global ⌘F (fase 5)
- Ordenação por coluna clicando no header (fase 5)
- Dashboard (fase 4)

---

## Estrutura de arquivos

```
src-tauri/
└── src/
    ├── domain/
    │   ├── category.rs                  T1 (novo)
    │   └── mod.rs                       T1 (export adicional)
    ├── commands/
    │   ├── categories.rs                T1 (novo)
    │   ├── transactions.rs              T1 (estende: update_category, update_notes; modifica list_transactions pra aceitar filtros)
    │   └── mod.rs                       T1 (declara categories)
    └── lib.rs                           T2 (registra novos commands)

src/
└── lib/
    ├── stores/
    │   └── filters.svelte.ts            T4 (novo)
    ├── api/
    │   ├── categories.ts                T3 (novo)
    │   └── transactions.ts              T3 (estende: updateTransactionCategory, updateTransactionNotes; novo signature de listTransactions)
    └── components/
        └── transactions/
            ├── CategoryPicker.svelte    T5 (novo)
            ├── TxFilterBar.svelte       T7 (novo)
            ├── TxNotesPanel.svelte      T9 (novo)
            └── TxTable.svelte           T6 (estende com coluna categoria + click handlers)
└── routes/
    └── Transactions.svelte              T8 (estende com FilterBar + NotesPanel)
```

---

## Task 1: Rust — domain::category + commands (categories + tx updates)

**Files:**
- Create: `src-tauri/src/domain/category.rs`
- Modify: `src-tauri/src/domain/mod.rs`
- Create: `src-tauri/src/commands/categories.rs`
- Modify: `src-tauri/src/commands/transactions.rs`
- Modify: `src-tauri/src/commands/mod.rs`

- [ ] **Step 1: Create `src-tauri/src/domain/category.rs`**

```rust
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub color_token: Option<String>,
    pub kind: String, // 'expense' | 'income' | 'transfer'
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct NewCategory {
    pub name: String,
    pub color_token: Option<String>,
    pub kind: String,
}
```

- [ ] **Step 2: Update `src-tauri/src/domain/mod.rs`**

```rust
pub mod account;
pub mod category;
pub mod transaction;
```

- [ ] **Step 3: Create `src-tauri/src/commands/categories.rs`**

```rust
use rusqlite::params;
use tauri::State;

use crate::db::Db;
use crate::domain::category::{Category, NewCategory};
use crate::error::{AppError, AppResult};

#[tauri::command]
#[specta::specta]
pub fn list_categories(db: State<'_, Db>) -> AppResult<Vec<Category>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    let mut stmt = conn.prepare(
        "SELECT id, name, color_token, kind, created_at FROM categories ORDER BY kind, name",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Category {
            id: row.get(0)?,
            name: row.get(1)?,
            color_token: row.get(2)?,
            kind: row.get(3)?,
            created_at: row.get(4)?,
        })
    })?;
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(AppError::from)
}

#[tauri::command]
#[specta::specta]
pub fn create_category(db: State<'_, Db>, input: NewCategory) -> AppResult<Category> {
    if !matches!(input.kind.as_str(), "expense" | "income" | "transfer") {
        return Err(AppError::Invalid(format!(
            "invalid kind '{}' (must be expense|income|transfer)",
            input.kind
        )));
    }
    let conn = db.conn.lock().expect("db mutex poisoned");
    conn.execute(
        "INSERT INTO categories (name, color_token, kind) VALUES (?1, ?2, ?3)",
        params![input.name, input.color_token, input.kind],
    )?;
    let id = conn.last_insert_rowid();
    conn.query_row(
        "SELECT id, name, color_token, kind, created_at FROM categories WHERE id = ?1",
        params![id],
        |row| {
            Ok(Category {
                id: row.get(0)?,
                name: row.get(1)?,
                color_token: row.get(2)?,
                kind: row.get(3)?,
                created_at: row.get(4)?,
            })
        },
    )
    .map_err(AppError::from)
}

#[cfg(test)]
mod tests {
    use crate::db::migrations;
    use rusqlite::{params, Connection};

    fn fresh_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        migrations::apply(&conn).unwrap();
        conn
    }

    #[test]
    fn seed_categories_listed() {
        let conn = fresh_conn();
        let mut stmt = conn
            .prepare("SELECT name, kind FROM categories ORDER BY name")
            .unwrap();
        let rows: Vec<(String, String)> = stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        assert_eq!(rows.len(), 9);
        assert!(rows.iter().any(|(n, _)| n == "Mercado"));
        assert!(rows.iter().any(|(n, k)| n == "Renda" && k == "income"));
    }

    #[test]
    fn create_category_inserts_row() {
        let conn = fresh_conn();
        conn.execute(
            "INSERT INTO categories (name, color_token, kind) VALUES (?1, ?2, ?3)",
            params!["Pets", "--color-cat-outros", "expense"],
        )
        .unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM categories", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 10);
    }

    #[test]
    fn create_category_rejects_duplicate_name() {
        let conn = fresh_conn();
        let r = conn.execute(
            "INSERT INTO categories (name, color_token, kind) VALUES (?1, ?2, ?3)",
            params!["Mercado", "--color-cat-mercado", "expense"],
        );
        assert!(r.is_err(), "UNIQUE constraint on name should reject");
    }
}
```

- [ ] **Step 4: Update `src-tauri/src/commands/mod.rs`**

```rust
pub mod accounts;
pub mod categories;
pub mod health;
pub mod transactions;
```

- [ ] **Step 5: Modify `src-tauri/src/commands/transactions.rs`**

Add a `TransactionFilters` struct, change `list_transactions` to accept it (replacing the prior `account_id: Option<i64>` argument), and add two new commands `update_transaction_category` + `update_transaction_notes`.

Replace the imports + `list_transactions` function with this. Keep `insert_transactions`, `check_existing_fitids`, and the existing `#[cfg(test)] mod tests` unchanged. Append the two new commands after `check_existing_fitids` (before `#[cfg(test)]`).

```rust
use rusqlite::params;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

use crate::db::Db;
use crate::domain::transaction::{InsertResult, NewTransaction, Transaction};
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
pub struct TransactionFilters {
    pub account_id: Option<i64>,
    /// ISO month "YYYY-MM" — restricts to transactions whose date starts with this prefix.
    pub month: Option<String>,
    pub category_id: Option<i64>,
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
    let sql = format!(
        "SELECT id, account_id, date, amount, description, category_id, notes, ofx_fitid, imported_at
         FROM transactions{where_sql} ORDER BY date DESC, id DESC",
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
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(AppError::from)
}
```

After `check_existing_fitids` add (still inside the same file, before the `#[cfg(test)]` block):

```rust
/// Set or clear the category of a transaction. Pass null/None to clear.
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

/// Set or clear the notes of a transaction.
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
```

Add 2 tests inside the existing `mod tests` block (or create a new block for these):

```rust
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
    fn update_transaction_category_changes_value() {
        let mut conn = fresh_conn();
        let acc = insert_account(&conn, "test", Some("ACC1"));
        let txs = vec![mk("F1", "10")];
        raw_insert_batch(&mut conn, acc, &txs);
        let tx_id: i64 = conn
            .query_row("SELECT id FROM transactions WHERE ofx_fitid = 'F1'", [], |r| r.get(0))
            .unwrap();
        let cat_id: i64 = conn
            .query_row("SELECT id FROM categories WHERE name = 'Mercado'", [], |r| r.get(0))
            .unwrap();

        conn.execute(
            "UPDATE transactions SET category_id = ?1 WHERE id = ?2",
            params![cat_id, tx_id],
        )
        .unwrap();

        let stored: i64 = conn
            .query_row("SELECT category_id FROM transactions WHERE id = ?1", params![tx_id], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(stored, cat_id);
    }
```

- [ ] **Step 6: Run cargo test**

```bash
cd src-tauri && cargo test --lib 2>&1 | tail -20 && cd ..
```
Expected: **21 tests pass** (16 + 3 categories + 2 transactions).

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/
git commit -m "$(cat <<'EOF'
feat(commands): categories CRUD + tx update_category + update_notes + filter

- domain/category.rs: Category, NewCategory
- commands/categories.rs: list_categories, create_category (valida kind)
- commands/transactions.rs:
  - TransactionFilters { account_id, month YYYY-MM, category_id }
  - list_transactions(filters?) (substitui signature antiga account_id)
  - update_transaction_category(tx_id, category_id?)
  - update_transaction_notes(tx_id, notes?)
- 5 testes novos (seeds list, create insert, UNIQUE name, filter by month, update category)

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 2: Register new commands + regenerate bindings

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Edit the `collect_commands!` macro call**

Replace it with:
```rust
let specta_builder = Builder::<tauri::Wry>::new().commands(collect_commands![
    commands::health::health_check,
    commands::accounts::list_accounts,
    commands::accounts::create_or_get_account,
    commands::categories::list_categories,
    commands::categories::create_category,
    commands::transactions::list_transactions,
    commands::transactions::insert_transactions,
    commands::transactions::check_existing_fitids,
    commands::transactions::update_transaction_category,
    commands::transactions::update_transaction_notes,
]);
```

Keep everything else (`#[cfg(debug_assertions)]` block with @ts-nocheck prepend, `BigIntExportBehavior::Number`, setup, etc.) exactly as before.

- [ ] **Step 2: Boot dev to regenerate bindings**

```bash
pkill -f "tauri dev" 2>/dev/null; pkill -f "vite" 2>/dev/null; pkill -f "target/debug/finan" 2>/dev/null
pnpm tauri dev > /tmp/finan-dev.log 2>&1 &
until grep -q "updateTransactionCategory" src/lib/bindings.ts 2>/dev/null && \
      grep -q "Category" src/lib/bindings.ts 2>/dev/null && \
      grep -q "TransactionFilters" src/lib/bindings.ts 2>/dev/null; do
  sleep 3
done
pkill -f "tauri dev"; pkill -f "vite"; pkill -f "target/debug/finan"
sleep 2
grep "export type" src/lib/bindings.ts
grep "async " src/lib/bindings.ts
```

Expected: types `Category`, `NewCategory`, `TransactionFilters` appear. Functions `listCategories`, `createCategory`, `updateTransactionCategory`, `updateTransactionNotes` appear.

- [ ] **Step 3: Verify pnpm check**

The TS code from Fase 1 calls `commands.listTransactions(accountId)` with a number/null directly. After this change, the signature becomes `commands.listTransactions(filters?)`. **Expect breakage on `src/lib/api/transactions.ts`**, which we'll fix in T3. For now, run:

```bash
pnpm check 2>&1 | tail -10
```

Expected: 1-2 errors on `src/lib/api/transactions.ts` complaining about the listTransactions signature mismatch. This is OK — we fix it in T3.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "$(cat <<'EOF'
feat(ipc): registra commands de categories + tx updates no specta builder

- 10 commands totais
- bindings.ts regenerado com Category, NewCategory, TransactionFilters
- BREAKING: listTransactions agora aceita filters?: TransactionFilters
  (TS wrappers serão atualizados em T3)

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: TS API wrappers — categories + tx updates + new listTransactions signature

**Files:**
- Create: `src/lib/api/categories.ts`
- Modify: `src/lib/api/transactions.ts`

- [ ] **Step 1: Create `src/lib/api/categories.ts`**

```ts
import { commands } from "../bindings";
import type { Category, NewCategory } from "../bindings";

function unwrap<T>(result: { status: "ok"; data: T } | { status: "error"; error: string }): T {
  if (result.status === "error") throw new Error(result.error);
  return result.data;
}

export async function listCategories(): Promise<Category[]> {
  return unwrap(await commands.listCategories());
}

export async function createCategory(input: NewCategory): Promise<Category> {
  return unwrap(await commands.createCategory(input));
}
```

- [ ] **Step 2: Replace `src/lib/api/transactions.ts`**

```ts
import { commands } from "../bindings";
import type {
  InsertResult,
  NewTransaction,
  Transaction,
  TransactionFilters,
} from "../bindings";

function unwrap<T>(result: { status: "ok"; data: T } | { status: "error"; error: string }): T {
  if (result.status === "error") throw new Error(result.error);
  return result.data;
}

export async function listTransactions(
  filters: TransactionFilters | null = null,
): Promise<Transaction[]> {
  return unwrap(await commands.listTransactions(filters));
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

export async function updateTransactionCategory(
  transactionId: number,
  categoryId: number | null,
): Promise<void> {
  return unwrap(await commands.updateTransactionCategory(transactionId, categoryId));
}

export async function updateTransactionNotes(
  transactionId: number,
  notes: string | null,
): Promise<void> {
  return unwrap(await commands.updateTransactionNotes(transactionId, notes));
}
```

- [ ] **Step 3: Update existing callers of `listTransactions`**

`src/routes/Transactions.svelte` previously called `listTransactions()` (no args). The new signature accepts a single `filters` arg (default `null`). Calling `listTransactions()` still works because of the default. No code change needed in Transactions.svelte for this step.

`src/routes/Import.svelte` doesn't call `listTransactions` — no change.

- [ ] **Step 4: Type-check**

```bash
pnpm check 2>&1 | tail -5
```
Expected: 0 errors.

- [ ] **Step 5: Commit**

```bash
git add src/lib/api/
git commit -m "$(cat <<'EOF'
feat(api): listCategories + createCategory + tx update wrappers

- listTransactions agora aceita filters?: TransactionFilters
- updateTransactionCategory (categoryId?: number | null pra clear)
- updateTransactionNotes (notes?: string | null)
- listCategories + createCategory

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 4: Filter store (Svelte 5 rune-based)

**Files:**
- Create: `src/lib/stores/filters.svelte.ts`

- [ ] **Step 1: Create `src/lib/stores/filters.svelte.ts`**

```ts
function currentMonth(): string {
  const now = new Date();
  const y = now.getFullYear();
  const m = String(now.getMonth() + 1).padStart(2, "0");
  return `${y}-${m}`;
}

function createFilterStore() {
  let month = $state<string | null>(currentMonth());
  let categoryId = $state<number | null>(null);

  return {
    get month() {
      return month;
    },
    set month(v: string | null) {
      month = v;
    },
    get categoryId() {
      return categoryId;
    },
    set categoryId(v: number | null) {
      categoryId = v;
    },
    clear() {
      month = null;
      categoryId = null;
    },
    resetToCurrentMonth() {
      month = currentMonth();
      categoryId = null;
    },
  };
}

export const filters = createFilterStore();
```

- [ ] **Step 2: Type-check**

```bash
pnpm check 2>&1 | tail -5
```
Expected: 0 errors.

> **Nota:** `.svelte.ts` é a extensão que habilita runes em arquivos não-componentes. O Svelte 5 compila `$state` etc. nessas arquivos.

- [ ] **Step 3: Commit**

```bash
git add src/lib/stores/
git commit -m "$(cat <<'EOF'
feat(store): filter store rune-based (month + categoryId)

- filters.month default = mês atual no formato YYYY-MM
- filters.categoryId default = null (todas)
- Singleton exportado: estado sobrevive a navegação entre rotas

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 5: CategoryPicker component

**Files:**
- Create: `src/lib/components/transactions/CategoryPicker.svelte`

- [ ] **Step 1: Create `src/lib/components/transactions/CategoryPicker.svelte`**

```svelte
<script lang="ts">
  import { onMount, tick } from "svelte";
  import type { Category } from "$lib/bindings";

  type Props = {
    categories: Category[];
    currentId: number | null;
    onselect: (categoryId: number | null) => void | Promise<void>;
    /** When user wants to create a new category from the typed text. */
    oncreate: (name: string) => Promise<Category>;
  };

  let { categories, currentId, onselect, oncreate }: Props = $props();

  let open = $state(false);
  let query = $state("");
  let highlighted = $state(0);
  let triggerEl: HTMLButtonElement | undefined = $state();
  let inputEl: HTMLInputElement | undefined = $state();

  let current = $derived(categories.find((c) => c.id === currentId));

  let filtered = $derived(
    categories.filter((c) => c.name.toLowerCase().includes(query.toLowerCase())),
  );

  /** Options shown in the list: filtered categories + optional "clear" + optional "create". */
  type Option =
    | { kind: "clear" }
    | { kind: "category"; category: Category }
    | { kind: "create"; name: string };

  let options = $derived<Option[]>(buildOptions(query, filtered, currentId));

  function buildOptions(q: string, list: Category[], cur: number | null): Option[] {
    const out: Option[] = [];
    if (cur !== null) out.push({ kind: "clear" });
    for (const c of list) out.push({ kind: "category", category: c });
    const trimmed = q.trim();
    if (trimmed && !list.some((c) => c.name.toLowerCase() === trimmed.toLowerCase())) {
      out.push({ kind: "create", name: trimmed });
    }
    return out;
  }

  async function openPicker() {
    open = true;
    query = "";
    highlighted = 0;
    await tick();
    inputEl?.focus();
  }

  function closePicker() {
    open = false;
    triggerEl?.focus();
  }

  async function choose(opt: Option) {
    if (opt.kind === "clear") {
      await onselect(null);
    } else if (opt.kind === "category") {
      await onselect(opt.category.id);
    } else {
      const created = await oncreate(opt.name);
      await onselect(created.id);
    }
    closePicker();
  }

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      closePicker();
      return;
    }
    if (e.key === "ArrowDown") {
      e.preventDefault();
      highlighted = Math.min(highlighted + 1, options.length - 1);
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      highlighted = Math.max(highlighted - 1, 0);
    } else if (e.key === "Enter") {
      e.preventDefault();
      const opt = options[highlighted];
      if (opt) void choose(opt);
    }
  }

  function colorStyle(token: string | null | undefined): string {
    return token ? `background: var(${token})` : "background: var(--color-cat-outros)";
  }

  onMount(() => {
    function clickOutside(e: MouseEvent) {
      if (!open) return;
      const target = e.target as Node | null;
      if (target && !triggerEl?.contains(target) && !inputEl?.parentElement?.parentElement?.contains(target)) {
        closePicker();
      }
    }
    document.addEventListener("mousedown", clickOutside);
    return () => document.removeEventListener("mousedown", clickOutside);
  });
</script>

<div class="relative inline-block">
  <button
    bind:this={triggerEl}
    type="button"
    onclick={openPicker}
    class="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full border border-border bg-surface-2 hover:bg-surface-3 text-[11px] font-medium text-fg-muted"
  >
    {#if current}
      <span class="w-2 h-2 rounded-full" style={colorStyle(current.color_token)}></span>
      <span class="text-fg">{current.name}</span>
    {:else}
      <span class="text-fg-faint">Sem categoria</span>
    {/if}
  </button>

  {#if open}
    <div
      class="absolute z-30 mt-1 w-56 rounded-lg border border-border bg-surface shadow-pop overflow-hidden"
      style="box-shadow: 0 12px 32px -8px rgba(0,0,0,.55), 0 0 0 1px var(--color-border)"
    >
      <div class="border-b border-border-subtle p-1.5">
        <input
          bind:this={inputEl}
          bind:value={query}
          {onkeydown}
          placeholder="Buscar ou criar…"
          class="w-full bg-transparent border-0 outline-none text-[12px] px-1.5 py-1"
        />
      </div>
      <ul class="max-h-60 overflow-y-auto py-1 text-[12px]">
        {#each options as opt, i}
          <li>
            <button
              type="button"
              tabindex="-1"
              onmouseenter={() => (highlighted = i)}
              onclick={() => choose(opt)}
              class="w-full flex items-center gap-2 px-2.5 py-1.5 text-left
                     {i === highlighted ? 'bg-accent-soft text-fg' : 'text-fg-muted hover:bg-hover'}"
            >
              {#if opt.kind === "clear"}
                <span class="w-2 h-2 rounded-full bg-transparent border border-border"></span>
                <span class="italic">Remover categoria</span>
              {:else if opt.kind === "category"}
                <span class="w-2 h-2 rounded-full" style={colorStyle(opt.category.color_token)}></span>
                <span>{opt.category.name}</span>
                <span class="ml-auto text-[10px] text-fg-faint">{opt.category.kind}</span>
              {:else}
                <span class="w-2 h-2 rounded-full bg-accent"></span>
                <span>Criar <strong class="text-fg">"{opt.name}"</strong></span>
              {/if}
            </button>
          </li>
        {:else}
          <li class="px-2.5 py-2 text-fg-faint italic">Nenhuma categoria.</li>
        {/each}
      </ul>
    </div>
  {/if}
</div>
```

> **Notas de design:**
> - Atalho de teclado dentro do input: ↑/↓ navega, Enter confirma, Esc fecha.
> - Click-outside fecha via listener no document (limpa no `onMount` cleanup).
> - "Remover categoria" só aparece se já houver categoria atribuída.
> - "Criar X" só aparece se a query não bater exatamente com uma categoria existente.
> - Cor da bolinha vem do `--color-cat-*` token armazenado em `color_token`.

- [ ] **Step 2: Type-check**

```bash
pnpm check 2>&1 | tail -5
```
Expected: 0 errors.

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/transactions/CategoryPicker.svelte
git commit -m "$(cat <<'EOF'
feat(ui): CategoryPicker popover com busca + teclado + criar nova

- Trigger inline (chip) com bolinha colorida + nome
- Popover absoluto com input de busca
- Navegação por teclado: ↑/↓ Enter Esc
- "Remover categoria" e "Criar X" como opções dinâmicas
- Click-outside fecha

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 6: TxTable com coluna Categoria + integração CategoryPicker

**Files:**
- Modify: `src/lib/components/transactions/TxTable.svelte`

- [ ] **Step 1: Replace `src/lib/components/transactions/TxTable.svelte`**

```svelte
<script lang="ts">
  import { formatMoney } from "$lib/format/money";
  import CategoryPicker from "./CategoryPicker.svelte";
  import type { Category, Transaction } from "$lib/bindings";

  type Props = {
    transactions: Transaction[];
    categories: Category[];
    onCategoryChange: (transactionId: number, categoryId: number | null) => Promise<void>;
    onCategoryCreate: (name: string) => Promise<Category>;
    onRowClick?: (transaction: Transaction) => void;
    selectedId?: number | null;
  };

  let {
    transactions,
    categories,
    onCategoryChange,
    onCategoryCreate,
    onRowClick,
    selectedId,
  }: Props = $props();
</script>

<div class="rounded-lg border border-border-subtle bg-surface overflow-hidden">
  <table class="w-full text-[12px]">
    <thead class="bg-surface-2">
      <tr>
        <th class="text-left px-4 py-2 font-medium text-fg-faint uppercase tracking-wider text-[10.5px] w-[100px]">Data</th>
        <th class="text-left px-4 py-2 font-medium text-fg-faint uppercase tracking-wider text-[10.5px]">Descrição</th>
        <th class="text-left px-4 py-2 font-medium text-fg-faint uppercase tracking-wider text-[10.5px] w-[180px]">Categoria</th>
        <th class="text-right px-4 py-2 font-medium text-fg-faint uppercase tracking-wider text-[10.5px] w-[140px]">Valor</th>
      </tr>
    </thead>
    <tbody>
      {#each transactions as t (t.id)}
        <tr
          class="border-t border-border-subtle hover:bg-hover {selectedId === t.id ? 'bg-accent-soft' : ''}"
        >
          <td class="px-4 py-2.5 text-fg-muted tabular cursor-pointer" onclick={() => onRowClick?.(t)}>
            {t.date}
          </td>
          <td class="px-4 py-2.5 cursor-pointer" onclick={() => onRowClick?.(t)}>
            {t.description}
          </td>
          <td class="px-4 py-2.5">
            <CategoryPicker
              categories={categories}
              currentId={t.category_id}
              onselect={(catId) => onCategoryChange(t.id, catId)}
              oncreate={onCategoryCreate}
            />
          </td>
          <td
            class="px-4 py-2.5 text-right tabular font-medium cursor-pointer {Number(t.amount) >= 0 ? 'text-pos' : 'text-fg'}"
            onclick={() => onRowClick?.(t)}
          >
            {formatMoney(t.amount)}
          </td>
        </tr>
      {:else}
        <tr>
          <td colspan="4" class="px-4 py-10 text-center text-fg-faint">
            Nenhuma transação ainda. <a href="#/import" class="text-accent hover:underline">Importar um OFX</a>?
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>
```

- [ ] **Step 2: Type-check**

```bash
pnpm check 2>&1 | tail -5
```
Expected: 0 errors. (Transactions.svelte ainda não passa `categories` etc. — vai dar erro de prop required.) **Antecipado**: a partir daqui o `pnpm check` pode quebrar em `Transactions.svelte`. Será corrigido em T8.

Se o erro for só nessa rota, ok. Se for em outro lugar, investigar.

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/transactions/TxTable.svelte
git commit -m "$(cat <<'EOF'
feat(ui): TxTable coluna Categoria + CategoryPicker inline + onRowClick

- 4 colunas: Data | Descrição | Categoria | Valor
- CategoryPicker integrado por linha (via props onCategoryChange, onCategoryCreate)
- onRowClick (opcional) dispara em qualquer célula que não a de categoria
- selectedId destaca linha ativa (bg-accent-soft)

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 7: TxFilterBar component (chips de mês + categoria)

**Files:**
- Create: `src/lib/components/transactions/TxFilterBar.svelte`

- [ ] **Step 1: Create `src/lib/components/transactions/TxFilterBar.svelte`**

```svelte
<script lang="ts">
  import type { Category } from "$lib/bindings";

  type Props = {
    categories: Category[];
    /** YYYY-MM or null */
    month: string | null;
    categoryId: number | null;
    onMonthChange: (m: string | null) => void;
    onCategoryChange: (id: number | null) => void;
  };

  let { categories, month, categoryId, onMonthChange, onCategoryChange }: Props = $props();

  function monthLabel(m: string | null): string {
    if (!m) return "Todos os meses";
    const [y, mo] = m.split("-");
    const names = ["Jan", "Fev", "Mar", "Abr", "Mai", "Jun", "Jul", "Ago", "Set", "Out", "Nov", "Dez"];
    return `${names[Number(mo) - 1]}/${y.slice(-2)}`;
  }

  function shiftMonth(m: string | null, delta: number): string | null {
    if (!m) {
      const now = new Date();
      now.setMonth(now.getMonth() + delta);
      const y = now.getFullYear();
      const mo = String(now.getMonth() + 1).padStart(2, "0");
      return `${y}-${mo}`;
    }
    const [y, mo] = m.split("-").map((s) => Number(s));
    const d = new Date(y, mo - 1 + delta, 1);
    return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}`;
  }

  let currentCategory = $derived(categories.find((c) => c.id === categoryId));
</script>

<div class="flex items-center gap-2 flex-wrap">
  <!-- Month stepper -->
  <div class="inline-flex items-center gap-px rounded-md border border-border bg-surface-2">
    <button
      type="button"
      class="px-2 py-1 text-fg-muted hover:bg-hover rounded-l-md"
      onclick={() => onMonthChange(shiftMonth(month, -1))}
      aria-label="Mês anterior"
    >
      ‹
    </button>
    <span class="px-2.5 text-[12px] font-medium tabular min-w-[88px] text-center">
      {monthLabel(month)}
    </span>
    <button
      type="button"
      class="px-2 py-1 text-fg-muted hover:bg-hover rounded-r-md"
      onclick={() => onMonthChange(shiftMonth(month, +1))}
      aria-label="Próximo mês"
    >
      ›
    </button>
  </div>

  {#if month}
    <button
      type="button"
      onclick={() => onMonthChange(null)}
      class="text-[11px] text-fg-faint hover:text-fg-muted underline-offset-2 hover:underline"
    >
      Todos os meses
    </button>
  {/if}

  <!-- Category dropdown -->
  <select
    value={categoryId === null ? "" : String(categoryId)}
    onchange={(e) => {
      const v = (e.currentTarget as HTMLSelectElement).value;
      onCategoryChange(v === "" ? null : Number(v));
    }}
    class="text-[12px] rounded-md border border-border bg-surface-2 px-2 py-1 text-fg"
  >
    <option value="">Todas as categorias</option>
    {#each categories as c}
      <option value={String(c.id)}>{c.name}</option>
    {/each}
  </select>

  {#if currentCategory}
    <span class="text-[11px] text-fg-faint">· {currentCategory.kind}</span>
  {/if}
</div>
```

- [ ] **Step 2: Type-check**

```bash
pnpm check 2>&1 | tail -5
```
The error from T6 about missing props in Transactions.svelte may persist. Acceptable; will be resolved in T8.

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/transactions/TxFilterBar.svelte
git commit -m "$(cat <<'EOF'
feat(ui): TxFilterBar com mês stepper + dropdown categoria

- Stepper ‹ Mai/26 › navega mês a mês
- Link "Todos os meses" limpa filtro de mês
- Dropdown de categoria nativo
- Callbacks onMonthChange/onCategoryChange tipados

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 8: Transactions route — wire filters + categories + picker

**Files:**
- Modify: `src/routes/Transactions.svelte`

- [ ] **Step 1: Replace `src/routes/Transactions.svelte`**

```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import TxTable from "$lib/components/transactions/TxTable.svelte";
  import TxFilterBar from "$lib/components/transactions/TxFilterBar.svelte";
  import { filters } from "$lib/stores/filters.svelte";
  import { listCategories, createCategory } from "$lib/api/categories";
  import { listTransactions, updateTransactionCategory } from "$lib/api/transactions";
  import type { Category, Transaction } from "$lib/bindings";

  let transactions = $state<Transaction[]>([]);
  let categories = $state<Category[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  async function refresh() {
    try {
      transactions = await listTransactions({
        account_id: null,
        month: filters.month,
        category_id: filters.categoryId,
      });
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  onMount(async () => {
    try {
      categories = await listCategories();
      await refresh();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  });

  async function onCategoryChange(transactionId: number, categoryId: number | null) {
    await updateTransactionCategory(transactionId, categoryId);
    // Optimistic: patch the local list.
    transactions = transactions.map((t) =>
      t.id === transactionId ? { ...t, category_id: categoryId } : t,
    );
    // If a category filter is active and this row no longer matches, refresh.
    if (filters.categoryId !== null && filters.categoryId !== categoryId) {
      await refresh();
    }
  }

  async function onCategoryCreate(name: string): Promise<Category> {
    const created = await createCategory({
      name,
      color_token: "--color-cat-outros",
      kind: "expense",
    });
    categories = [...categories, created];
    return created;
  }

  async function onMonthChange(m: string | null) {
    filters.month = m;
    await refresh();
  }
  async function onCategoryFilterChange(id: number | null) {
    filters.categoryId = id;
    await refresh();
  }
</script>

<section class="p-8 max-w-5xl mx-auto flex flex-col gap-5">
  <header class="flex items-baseline justify-between gap-4 flex-wrap">
    <h2 class="text-xl font-semibold tracking-tight" style="font-family: var(--font-display)">
      Transações
    </h2>
    <span class="text-xs text-fg-faint tabular">
      {transactions.length} {transactions.length === 1 ? "transação" : "transações"}
    </span>
  </header>

  <TxFilterBar
    {categories}
    month={filters.month}
    categoryId={filters.categoryId}
    {onMonthChange}
    onCategoryChange={onCategoryFilterChange}
  />

  {#if loading}
    <div class="text-fg-faint text-sm">Carregando…</div>
  {:else if error}
    <div class="rounded-lg border border-border bg-surface p-3 text-sm text-neg">{error}</div>
  {:else}
    <TxTable
      {transactions}
      {categories}
      {onCategoryChange}
      {onCategoryCreate}
    />
  {/if}
</section>
```

> **Notas:**
> - `onCategoryChange` é otimista (patcha estado local antes/sem esperar reload completo). Se houver filtro de categoria ativo e a transação deixar de bater, refaz a query.
> - `onCategoryCreate` cria sempre como `expense` com cor "outros" por default — fase 5 pode adicionar picker de kind/cor.

- [ ] **Step 2: Type-check**

```bash
pnpm check 2>&1 | tail -5
```
Expected: 0 errors.

- [ ] **Step 3: Smoke test**

```bash
pkill -f "tauri dev" 2>/dev/null; pkill -f "vite" 2>/dev/null; pkill -f "target/debug/finan" 2>/dev/null
pnpm tauri dev > /tmp/finan-dev.log 2>&1 &
until lsof -iTCP:1420 -sTCP:LISTEN >/dev/null 2>&1; do sleep 2; done
grep -i error /tmp/finan-dev.log | head -5
pkill -f "tauri dev"; pkill -f "vite"; pkill -f "target/debug/finan"
```
Expected: dev boots clean, no errors in log.

- [ ] **Step 4: Commit**

```bash
git add src/routes/Transactions.svelte
git commit -m "$(cat <<'EOF'
feat(transactions): wire FilterBar, CategoryPicker e store de filtros

- FilterBar + TxTable conectados via callbacks
- Filtros (mês + categoria) persistidos em $lib/stores/filters
- updateTransactionCategory otimista (patch local + refresh se filter conflitar)
- createCategory cria como 'expense' + outros (fase 5: picker de kind/cor)

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 9: Notes detail panel (right pane)

**Files:**
- Create: `src/lib/components/transactions/TxNotesPanel.svelte`
- Modify: `src/routes/Transactions.svelte`

- [ ] **Step 1: Create `src/lib/components/transactions/TxNotesPanel.svelte`**

```svelte
<script lang="ts">
  import { tick } from "svelte";
  import { Button } from "$lib/components/ui/button";
  import { formatMoney } from "$lib/format/money";
  import type { Transaction } from "$lib/bindings";

  type Props = {
    transaction: Transaction;
    onClose: () => void;
    onSave: (transactionId: number, notes: string | null) => Promise<void>;
  };

  let { transaction, onClose, onSave }: Props = $props();

  let draft = $state(transaction.notes ?? "");
  let busy = $state(false);
  let textarea: HTMLTextAreaElement | undefined = $state();

  $effect(() => {
    draft = transaction.notes ?? "";
  });

  async function save() {
    busy = true;
    try {
      const value = draft.trim();
      await onSave(transaction.id, value === "" ? null : value);
      onClose();
    } finally {
      busy = false;
    }
  }

  async function focusTextarea() {
    await tick();
    textarea?.focus();
  }

  $effect(() => {
    void focusTextarea();
  });
</script>

<!-- Overlay backdrop -->
<button
  type="button"
  aria-label="Fechar"
  onclick={onClose}
  class="fixed inset-0 z-20 bg-black/30"
></button>

<aside
  class="fixed right-0 top-0 bottom-0 z-30 w-[360px] bg-surface border-l border-border-subtle shadow-pop flex flex-col"
  style="box-shadow: -12px 0 32px -8px rgba(0,0,0,.55)"
>
  <header class="flex items-center justify-between px-4 py-3 border-b border-border-subtle">
    <span class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint">Detalhe</span>
    <button type="button" onclick={onClose} class="text-fg-muted hover:text-fg" aria-label="Fechar">
      ✕
    </button>
  </header>

  <div class="p-4 flex flex-col gap-3 text-[12px]">
    <div class="flex justify-between">
      <span class="text-fg-muted">Data</span>
      <span class="tabular">{transaction.date}</span>
    </div>
    <div class="flex justify-between">
      <span class="text-fg-muted">Valor</span>
      <span class="tabular font-semibold {Number(transaction.amount) >= 0 ? 'text-pos' : 'text-fg'}">
        {formatMoney(transaction.amount)}
      </span>
    </div>
    <div class="flex flex-col gap-1">
      <span class="text-fg-muted">Descrição</span>
      <span class="text-fg">{transaction.description}</span>
    </div>
    {#if transaction.ofx_fitid}
      <div class="flex justify-between">
        <span class="text-fg-muted">FITID</span>
        <span class="font-mono text-[11px] text-fg-faint">{transaction.ofx_fitid}</span>
      </div>
    {/if}
  </div>

  <div class="px-4 pb-2">
    <label class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint mb-1 block" for="tx-notes">
      Notes
    </label>
    <textarea
      id="tx-notes"
      bind:this={textarea}
      bind:value={draft}
      placeholder="Anotações sobre essa transação…"
      rows="6"
      class="w-full rounded-md border border-border bg-surface-2 p-2 text-[12px] text-fg resize-none focus:outline-none focus:border-accent focus:bg-bg"
    ></textarea>
  </div>

  <footer class="mt-auto px-4 py-3 border-t border-border-subtle flex justify-end gap-2">
    <Button variant="ghost" onclick={onClose}>Cancelar</Button>
    <Button onclick={save} disabled={busy}>{busy ? "Salvando…" : "Salvar"}</Button>
  </footer>
</aside>
```

- [ ] **Step 2: Update `src/routes/Transactions.svelte` to mount the panel**

Add an import:
```ts
import TxNotesPanel from "$lib/components/transactions/TxNotesPanel.svelte";
import { updateTransactionNotes } from "$lib/api/transactions";
```

Add state + handlers after `categories` declaration:
```ts
let selectedTx = $state<Transaction | null>(null);

function onRowClick(t: Transaction) {
  selectedTx = t;
}
function closePanel() {
  selectedTx = null;
}
async function onSaveNotes(transactionId: number, notes: string | null) {
  await updateTransactionNotes(transactionId, notes);
  transactions = transactions.map((t) =>
    t.id === transactionId ? { ...t, notes } : t,
  );
}
```

Pass `onRowClick={onRowClick} selectedId={selectedTx?.id ?? null}` to `<TxTable>`. Below the `<TxTable>`, conditionally render:
```svelte
{#if selectedTx}
  <TxNotesPanel transaction={selectedTx} onClose={closePanel} onSave={onSaveNotes} />
{/if}
```

For clarity, here's the complete final `src/routes/Transactions.svelte`:

```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import TxTable from "$lib/components/transactions/TxTable.svelte";
  import TxFilterBar from "$lib/components/transactions/TxFilterBar.svelte";
  import TxNotesPanel from "$lib/components/transactions/TxNotesPanel.svelte";
  import { filters } from "$lib/stores/filters.svelte";
  import { listCategories, createCategory } from "$lib/api/categories";
  import {
    listTransactions,
    updateTransactionCategory,
    updateTransactionNotes,
  } from "$lib/api/transactions";
  import type { Category, Transaction } from "$lib/bindings";

  let transactions = $state<Transaction[]>([]);
  let categories = $state<Category[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let selectedTx = $state<Transaction | null>(null);

  async function refresh() {
    try {
      transactions = await listTransactions({
        account_id: null,
        month: filters.month,
        category_id: filters.categoryId,
      });
      if (selectedTx) {
        const fresh = transactions.find((t) => t.id === selectedTx?.id);
        selectedTx = fresh ?? null;
      }
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  onMount(async () => {
    try {
      categories = await listCategories();
      await refresh();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  });

  async function onCategoryChange(transactionId: number, categoryId: number | null) {
    await updateTransactionCategory(transactionId, categoryId);
    transactions = transactions.map((t) =>
      t.id === transactionId ? { ...t, category_id: categoryId } : t,
    );
    if (filters.categoryId !== null && filters.categoryId !== categoryId) {
      await refresh();
    }
  }

  async function onCategoryCreate(name: string): Promise<Category> {
    const created = await createCategory({
      name,
      color_token: "--color-cat-outros",
      kind: "expense",
    });
    categories = [...categories, created];
    return created;
  }

  async function onMonthChange(m: string | null) {
    filters.month = m;
    await refresh();
  }
  async function onCategoryFilterChange(id: number | null) {
    filters.categoryId = id;
    await refresh();
  }

  function onRowClick(t: Transaction) {
    selectedTx = t;
  }
  function closePanel() {
    selectedTx = null;
  }
  async function onSaveNotes(transactionId: number, notes: string | null) {
    await updateTransactionNotes(transactionId, notes);
    transactions = transactions.map((t) =>
      t.id === transactionId ? { ...t, notes } : t,
    );
  }
</script>

<section class="p-8 max-w-5xl mx-auto flex flex-col gap-5">
  <header class="flex items-baseline justify-between gap-4 flex-wrap">
    <h2 class="text-xl font-semibold tracking-tight" style="font-family: var(--font-display)">
      Transações
    </h2>
    <span class="text-xs text-fg-faint tabular">
      {transactions.length} {transactions.length === 1 ? "transação" : "transações"}
    </span>
  </header>

  <TxFilterBar
    {categories}
    month={filters.month}
    categoryId={filters.categoryId}
    {onMonthChange}
    onCategoryChange={onCategoryFilterChange}
  />

  {#if loading}
    <div class="text-fg-faint text-sm">Carregando…</div>
  {:else if error}
    <div class="rounded-lg border border-border bg-surface p-3 text-sm text-neg">{error}</div>
  {:else}
    <TxTable
      {transactions}
      {categories}
      {onCategoryChange}
      {onCategoryCreate}
      {onRowClick}
      selectedId={selectedTx?.id ?? null}
    />
  {/if}
</section>

{#if selectedTx}
  <TxNotesPanel transaction={selectedTx} onClose={closePanel} onSave={onSaveNotes} />
{/if}
```

- [ ] **Step 3: Type-check**

```bash
pnpm check 2>&1 | tail -5
```
Expected: 0 errors.

- [ ] **Step 4: Smoke test boot**

```bash
pkill -f "tauri dev" 2>/dev/null; pkill -f "vite" 2>/dev/null; pkill -f "target/debug/finan" 2>/dev/null
pnpm tauri dev > /tmp/finan-dev.log 2>&1 &
until lsof -iTCP:1420 -sTCP:LISTEN >/dev/null 2>&1; do sleep 2; done
grep -i error /tmp/finan-dev.log | head -5
pkill -f "tauri dev"; pkill -f "vite"; pkill -f "target/debug/finan"
```

- [ ] **Step 5: Commit**

```bash
git add src/lib/components/transactions/TxNotesPanel.svelte src/routes/Transactions.svelte
git commit -m "$(cat <<'EOF'
feat(transactions): right pane (drawer) com notes editáveis

- TxNotesPanel.svelte: drawer fixo direita com data/valor/descrição/FITID + textarea
- Click numa linha (exceto célula de categoria) abre o panel
- updateTransactionNotes grava ao clicar Salvar
- Atualiza tanto a lista quanto o selectedTx (mantém estado consistente)

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 10: Acceptance verification + Fase 2 close

**Files:**
- (verificação)
- Modify: `README.md`

- [ ] **Step 1: Full test suite**

```bash
pnpm check 2>&1 | tail -3
echo "=== pnpm test ==="
pnpm test 2>&1 | tail -5
echo "=== cargo test ==="
cd src-tauri && cargo test --lib 2>&1 | tail -3 && cd ..
echo "=== clippy ==="
cd src-tauri && cargo clippy --all-targets -- -D warnings 2>&1 | tail -5 && cd ..
echo "=== fmt ==="
cd src-tauri && cargo fmt --check 2>&1 | tail -3 && cd ..
echo "=== build ==="
pnpm build 2>&1 | tail -5
```

Expected: pnpm check 0 / pnpm test 19+ / cargo test 21+ / clippy clean / fmt clean / build OK.

- [ ] **Step 2: Manual UX walkthrough**

```bash
pkill -f "tauri dev" 2>/dev/null
pnpm tauri dev > /tmp/finan-dev.log 2>&1 &
until lsof -iTCP:1420 -sTCP:LISTEN >/dev/null 2>&1; do sleep 2; done
echo "ready"
```

Manual checklist (precisa de ter importado um OFX antes ou usar o fixture):
- [ ] `/transactions` mostra 4 colunas (data, descrição, categoria, valor).
- [ ] Coluna Categoria mostra "Sem categoria" pra tx ainda não atribuídas.
- [ ] Clicar na célula Categoria abre o popover.
- [ ] Buscar "merc" no popover filtra pra Mercado.
- [ ] ↑/↓ navegam; Enter confirma; Esc fecha.
- [ ] Após selecionar, o chip da linha mostra a categoria (e o valor é persistido).
- [ ] Recarregar `/transactions` mantém a atribuição.
- [ ] Digitar um nome novo (ex "Pets") e clicar "Criar Pets" cria a categoria e atribui.
- [ ] FilterBar: ‹/› stepper navega meses. "Todos os meses" limpa.
- [ ] Dropdown de categoria filtra a tabela.
- [ ] Navegar pra `/dashboard` e voltar pra `/transactions` mantém os filtros aplicados (store).
- [ ] Clicar numa linha (não na célula categoria) abre o drawer direita.
- [ ] Editar notes + Salvar persiste.
- [ ] Recarregar a página mostra notes salvas.
- [ ] Fechar a janela (Cmd+Q) sem crash.

```bash
pkill -f "tauri dev"; pkill -f "vite"; pkill -f "target/debug/finan"
```

- [ ] **Step 3: Update README**

Edit `README.md`:
```
## Status

- ✅ Fase 0 — Scaffold (Tauri + Svelte + DB + sidebar + IPC tipado)
- ✅ Fase 1 — Importar OFX (parser TS + dedup por FITID + listagem)
- ✅ Fase 2 — Categorização manual inline + filtros + notes
- 🚧 Fase 3 — Regras automáticas (próximo)
- ⏳ Fase 4-5 — Dashboard, polish
```

- [ ] **Step 4: Closing commit**

```bash
git add README.md
git commit -m "$(cat <<'EOF'
chore(fase-2): close categorization phase — acceptance criteria batem

- Inline CategoryPicker (search + arrow keys + create new)
- TxFilterBar com mês stepper + dropdown categoria
- Store de filtros (filters.svelte.ts) persiste durante a sessão
- Right pane drawer com notes editáveis
- Commands novos: list_categories, create_category, update_transaction_category,
  update_transaction_notes + list_transactions com filtros
- Tests: 21 cargo / 19 vitest / pnpm check 0 / clippy/fmt limpos
- Build: 145 KB JS + componentes novos

Próximo: plano da Fase 3 (Regras automáticas description-contains).

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Self-Review

### Spec coverage

| Spec item | Task |
|---|---|
| §8.3 Categoria editável inline (CategoryPicker popover) | T5, T6 |
| §8.3 Filtros mês (default atual) | T4 (default), T7, T8 |
| §8.3 Filtro categoria | T7, T8 |
| §8.3 Notes editável (right pane) | T9 |
| §9 Fase 2 entregável (inline picker + filtros + right pane) | T5–T9 |
| §9 Critério "filtros mantêm estado em store" | T4 |
| §5 update_transaction_category/notes commands (UI nunca toca SQL) | T1 |
| §5 categories seed presence (já existe — testes confirmam) | T1 |
| §10 cargo tests aumentam | T1 (5 novos) |

### Placeholder scan

Reli todos os blocos. Nenhum "TBD" / "TODO" / "implement later" / "add error handling". Cada step tem código completo.

### Type consistency

- `TransactionFilters { account_id, month, category_id }` — usado em Rust (T1), bindings auto-geradas (T2), TS wrapper (T3), Transactions.svelte (T8). Match.
- `Category { id, name, color_token, kind, created_at }` — T1 → bindings → CategoryPicker (T5), TxFilterBar (T7), Transactions (T8). Match.
- `update_transaction_category(transaction_id, category_id)` — Rust snake_case (T1), bindings auto camelCase (`updateTransactionCategory`), TS wrapper (T3), called from Transactions.svelte (T8). Match.
- `onCategoryChange` é usado com DUAS assinaturas distintas em T8: a passada pra `TxFilterBar` (toma `id`) e a passada pra `TxTable` (toma `txId, catId`). Renomeei a do filter pra `onCategoryFilterChange` na rota pra evitar confusão. Match.
- `filters.month` é `string | null`. `TransactionFilters.month` é `Option<String>` Rust → `string | null` TS. Match.
- `filters` store: exported singleton. Re-importing in different routes gives the same instance — that's how Svelte 5 rune modules work. Verified by Svelte 5 docs.

### Risks documented inline

- **T2 step 3 break:** intencional. Solucionado em T3.
- **T6 step 2 break:** intencional. Solucionado em T8.
- **CategoryPicker click-outside detection:** depende de `inputEl.parentElement.parentElement` — frágil se a estrutura mudar. Aceitável pro MVP; refatorar com `bits-ui` Popover em fase 5 se virar bug.
- **Notes drawer overlay:** `<button>` como backdrop é a forma a11y-correta (focusable, clicável, role implícito). Cobre teclado e mouse.
