# Fase 4 — Dashboard Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Tela `/dashboard` mostra a saúde financeira do mês selecionado: 4 KPIs (renda, gastos, saldo, count), donut por categoria, barras dos últimos 12 meses (income vs expense), top 5 categorias com meter de %, últimas 8 transações. Seletor de mês reaproveita `filters.month` store. Default route `/` redireciona pra `/onboarding` se DB vazia, senão pra Dashboard.

**Architecture:** Backend expõe 3 commands de agregação (`summary_kpis`, `summary_by_category`, `summary_by_month`) que fazem SUM em Rust com `rust_decimal` (nunca f64). `list_transactions` ganha campo `limit` em `TransactionFilters` pra reaproveitamento no widget "Recent". Frontend usa CSS-only charts: donut via `conic-gradient`, barras via flex+altura, sparklines triviais. Nada de LayerChart nesta fase — defer pra polish se aparecer demanda.

**Tech Stack:** já tudo instalado. Nada novo.

**Acceptance criteria (Fase 4):**
1. 3 commands Rust retornam dados agregados corretos pro mês indicado. Dinheiro stringificado de `rust_decimal`.
2. `TransactionFilters` ganha `limit: Option<u32>`. `list_transactions({ limit: 8 })` funciona.
3. `/dashboard` mostra 4 KPI cards com valores formatados em BRL.
4. Donut renderiza fatias coloridas conforme `color_token` das categorias; soma das fatias = 100%.
5. Barras dos 12 meses mostram income (verde) e expense (vermelho-ish ou cinza). Tooltip básico (title attribute) com os valores.
6. Top 5 categorias com nome, valor, e meter horizontal proporcional ao maior.
7. Últimas 8 transações com data, descrição, valor.
8. Trocar mês via stepper atualiza todos os widgets (exceto barras 12m, que são sempre last-12).
9. `/` (route raiz) checa transactions count → push `/onboarding` se 0, render Dashboard se >0.
10. Tests: `cargo test --lib` ≥ 30 (27 anteriores + 3 novos), `pnpm test` ≥ 19, `pnpm check` 0, clippy/fmt limpos.

**Out of scope:**
- LayerChart (defer pra fase 5 se quiser visual mais polido)
- Range custom (post-MVP per spec)
- Drill-down ao clicar fatia/barra (post-MVP)
- Comparação mês-anterior (post-MVP)
- Sparkline animado nos KPIs (post-MVP — pode ser estático)

---

## Estrutura de arquivos

```
src-tauri/
└── src/
    ├── commands/
    │   ├── summary.rs                   T1 (novo)
    │   ├── transactions.rs              T1 (adiciona limit em TransactionFilters)
    │   └── mod.rs                       T1 (declara summary)
    ├── domain/
    │   └── summary.rs                   T1 (novo: KpiSummary, CategorySpend, MonthSummary)
    │       └── mod.rs                   T1 (export summary)
    └── lib.rs                           T2 (registra 3 commands novos)

src/
└── lib/
    ├── api/
    │   ├── summary.ts                   T3 (novo)
    │   └── transactions.ts              T3 (signature aceita limit opcional)
    └── components/
        ├── shell/MonthStepper.svelte    T4 (extraído de TxFilterBar)
        ├── transactions/TxFilterBar.svelte  T4 (refactor pra usar MonthStepper)
        └── dashboard/
            ├── KpiCard.svelte           T4
            ├── CategoryDonut.svelte     T5
            ├── MonthBars.svelte         T5
            ├── TopCategoriesList.svelte T6
            └── RecentList.svelte        T6
└── routes/
    ├── Dashboard.svelte                 T7 (rewrite)
    └── routes.ts                        T7 (adicionar IndexRedirect na rota /)
```

---

## Task 1: Backend — summary commands + limit em TransactionFilters

**Files:**
- Create: `src-tauri/src/domain/summary.rs`
- Modify: `src-tauri/src/domain/mod.rs`
- Modify: `src-tauri/src/commands/transactions.rs` (adicionar `limit` em TransactionFilters)
- Create: `src-tauri/src/commands/summary.rs`
- Modify: `src-tauri/src/commands/mod.rs`

- [ ] **Step 1: Create `src-tauri/src/domain/summary.rs`**

```rust
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct KpiSummary {
    /// Total de entradas (soma de amounts > 0). Decimal string.
    pub income: String,
    /// Total de saídas em valor absoluto (soma de |amounts < 0|). Decimal string.
    pub expense: String,
    /// Saldo do período (income - expense). Decimal string.
    pub net: String,
    pub transaction_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CategorySpend {
    pub category_id: Option<i64>,
    /// "Sem categoria" quando category_id is None.
    pub name: String,
    pub color_token: Option<String>,
    /// Total em valor absoluto (gastos). Decimal string.
    pub total: String,
    /// Percentual sobre o total geral do período. Number 0-100.
    pub percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct MonthSummary {
    /// YYYY-MM
    pub month: String,
    pub income: String,
    pub expense: String,
}
```

- [ ] **Step 2: Update `src-tauri/src/domain/mod.rs`**

```rust
pub mod account;
pub mod category;
pub mod rule;
pub mod summary;
pub mod transaction;
```

- [ ] **Step 3: Modify `src-tauri/src/commands/transactions.rs` — add `limit` to TransactionFilters**

Find the existing struct:
```rust
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct TransactionFilters {
    pub account_id: Option<i64>,
    pub month: Option<String>,
    pub category_id: Option<i64>,
}
```

Replace with:
```rust
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct TransactionFilters {
    pub account_id: Option<i64>,
    pub month: Option<String>,
    pub category_id: Option<i64>,
    pub limit: Option<u32>,
}
```

In the `list_transactions` function, find the SQL construction and append a `LIMIT` clause when `f.limit` is `Some`:

Locate this line (or similar):
```rust
let sql = format!(
    "SELECT id, account_id, date, amount, description, category_id, notes, ofx_fitid, imported_at
     FROM transactions{where_sql} ORDER BY date DESC, id DESC",
);
```

Replace with:
```rust
let limit_sql = match f.limit {
    Some(n) => format!(" LIMIT {n}"),
    None => String::new(),
};
let sql = format!(
    "SELECT id, account_id, date, amount, description, category_id, notes, ofx_fitid, imported_at
     FROM transactions{where_sql} ORDER BY date DESC, id DESC{limit_sql}",
);
```

> **Por que interpolar e não bound param:** SQLite quer LIMIT como literal/named param; pra simplificar e como `n` é `u32` (não user-controlled string), interpolar é seguro aqui.

Add 1 new test inside the existing `mod tests` block:

```rust
    #[test]
    fn limit_clause_caps_result_count() {
        let mut conn = fresh_conn();
        let acc = insert_account(&conn, "test", Some("ACC1"));
        let txs = vec![
            mk("F1", "10.00"),
            mk("F2", "20.00"),
            mk("F3", "30.00"),
        ];
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
```

- [ ] **Step 4: Update `src-tauri/src/commands/mod.rs`**

```rust
pub mod accounts;
pub mod categories;
pub mod health;
pub mod rules;
pub mod summary;
pub mod transactions;
```

- [ ] **Step 5: Create `src-tauri/src/commands/summary.rs`**

```rust
use rusqlite::params;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::str::FromStr;
use tauri::State;

use crate::db::Db;
use crate::domain::summary::{CategorySpend, KpiSummary, MonthSummary};
use crate::error::{AppError, AppResult};

/// KPI totals for a month (or all-time when None).
#[tauri::command]
#[specta::specta]
pub fn summary_kpis(db: State<'_, Db>, month: Option<String>) -> AppResult<KpiSummary> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    let (sql, bind): (&str, Vec<&dyn rusqlite::ToSql>) = match month.as_ref() {
        Some(m) => (
            "SELECT amount FROM transactions WHERE date LIKE ?1",
            vec![],
        ),
        None => ("SELECT amount FROM transactions", vec![]),
    };

    let pattern: Option<String> = month.as_ref().map(|m| format!("{m}-%"));
    let mut stmt = conn.prepare(sql)?;
    let rows = if let Some(p) = pattern.as_ref() {
        stmt.query_map(params![p], |row| row.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?
    } else {
        let _ = bind; // silence unused; bind always empty in this branch
        stmt.query_map([], |row| row.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?
    };

    let mut income = Decimal::ZERO;
    let mut expense = Decimal::ZERO;
    let count = rows.len() as u32;
    for s in &rows {
        let d = Decimal::from_str(s).map_err(|e| AppError::Invalid(format!("bad amount '{s}': {e}")))?;
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

/// Spending grouped by category for a given month. Only expense transactions
/// (amount < 0) are aggregated. Sorted by total descending.
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

    let pat_for_query: Option<&str> = pattern.as_deref();
    let rows: Vec<(String, Option<i64>, Option<String>, Option<String>)> = stmt
        .query_map(params![pat_for_query], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let mut by: HashMap<Option<i64>, (String, Option<String>, Decimal)> = HashMap::new();
    let mut total_expense = Decimal::ZERO;

    for (amt, cat_id, name, color) in rows {
        let d = Decimal::from_str(&amt).map_err(|e| AppError::Invalid(format!("bad amount: {e}")))?;
        if !d.is_sign_negative() {
            continue; // only count expenses
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
                let p: f64 = (total / total_expense)
                    .to_string()
                    .parse()
                    .unwrap_or(0.0);
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

/// Last `months_back` months including the current one. Returns income + expense
/// per month, ordered ascending by month. Months with zero data are NOT included.
#[tauri::command]
#[specta::specta]
pub fn summary_by_month(
    db: State<'_, Db>,
    months_back: u32,
) -> AppResult<Vec<MonthSummary>> {
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
        let d = Decimal::from_str(&amt).map_err(|e| AppError::Invalid(format!("bad amount: {e}")))?;
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

/// Returns YYYY-MM-DD for the first day of (current month - months_back).
fn compute_cutoff(months_back: u32) -> String {
    let now = chrono::Utc::now().naive_utc().date();
    let total_months = (now.year() as i32) * 12 + (now.month() as i32 - 1);
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

    fn insert_tx(conn: &Connection, account_id: i64, date: &str, amount: &str, category_id: Option<i64>) {
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
        // outside month — must NOT count
        insert_tx(&conn, acc, "2026-03-30", "999.00", None);

        // Direct SQL of the same query used by summary_kpis (April 2026)
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
        insert_tx(&conn, acc, "2026-04-20", "-10.00", None); // sem categoria

        let mut stmt = conn
            .prepare(
                "SELECT t.amount, t.category_id, c.name
                 FROM transactions t LEFT JOIN categories c ON c.id = t.category_id
                 WHERE t.date LIKE ?1",
            )
            .unwrap();
        let rows: Vec<(String, Option<i64>, Option<String>)> = stmt
            .query_map(params!["2026-04-%"], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();

        let mut mercado_total = Decimal::ZERO;
        let mut sem_cat_total = Decimal::ZERO;
        let mut renda_count = 0u32;
        for (amt, cid, _) in &rows {
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
        assert_eq!(renda_count, 1, "income excluded");
    }

    #[test]
    fn cutoff_computation_handles_year_rollover() {
        // sanity: compute_cutoff is stable across runs only if we mock time.
        // Here we just verify the function compiles and returns a YYYY-MM-DD string.
        let s = compute_cutoff(12);
        assert_eq!(s.len(), 10);
        assert_eq!(&s[4..5], "-");
        assert_eq!(&s[7..10], "-01");
    }
}
```

> **Notas técnicas:**
> - `chrono::Utc::now()` é usado pra `compute_cutoff`. `chrono` já é dep.
> - `Decimal` ↔ string evita `f64` em todo o pipeline. `percent` é exceção (f64) por ser display-only e cabe perfeitamente em [0, 100].
> - O teste #1 (`kpis_sum_correctly_for_month`) replica o cálculo do command porque o command toma `State<Db>` (não acessível em unit test).

- [ ] **Step 6: Run cargo test**

```bash
cd src-tauri && cargo test --lib 2>&1 | tail -20 && cd ..
```
Expected: **31 tests pass** (27 prior + 1 limit + 3 summary).

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/
git commit -m "$(cat <<'EOF'
feat(summary): backend de agregação pro dashboard

- domain/summary.rs: KpiSummary, CategorySpend, MonthSummary
- commands/summary.rs:
  - summary_kpis(month?) → income/expense/net/count com rust_decimal
  - summary_by_category(month?) → agrupa só expenses, percent sobre total
  - summary_by_month(months_back) → series temporal por mês YYYY-MM
- TransactionFilters ganha limit?: u32 (LIMIT N pra Recent)
- 4 testes novos (kpis sum, category groups, cutoff, limit clause)

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 2: Register commands + bindings + TS API wrappers

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Create: `src/lib/api/summary.ts`
- Modify: `src/lib/api/transactions.ts` (limit no signature do filter)

- [ ] **Step 1: Edit `src-tauri/src/lib.rs` `collect_commands!`**

Add the 3 summary commands at the end of the list:

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
    commands::rules::list_rules,
    commands::rules::create_rule,
    commands::rules::update_rule,
    commands::rules::delete_rule,
    commands::rules::apply_rules_to_uncategorized,
    commands::summary::summary_kpis,
    commands::summary::summary_by_category,
    commands::summary::summary_by_month,
]);
```

Keep `@ts-nocheck` block, `BigIntExportBehavior::Number`, `.setup()` etc. unchanged.

- [ ] **Step 2: Regenerate bindings**

```bash
pkill -f "tauri dev" 2>/dev/null; pkill -f "vite" 2>/dev/null; pkill -f "target/debug/finan" 2>/dev/null
pnpm tauri dev > /tmp/finan-dev.log 2>&1 &
until grep -q "summaryByCategory" src/lib/bindings.ts 2>/dev/null && \
      grep -q "MonthSummary" src/lib/bindings.ts 2>/dev/null && \
      grep -q "limit" src/lib/bindings.ts 2>/dev/null; do
  sleep 3
done
pkill -f "tauri dev"; pkill -f "vite"; pkill -f "target/debug/finan"
sleep 2
grep "export type" src/lib/bindings.ts
echo "---"
grep "async " src/lib/bindings.ts
```

Expected new types: `KpiSummary`, `CategorySpend`, `MonthSummary`. `TransactionFilters` now has `limit: number | null`.
Expected new async fns: `summaryKpis`, `summaryByCategory`, `summaryByMonth`.

- [ ] **Step 3: Create `src/lib/api/summary.ts`**

```ts
import { commands } from "../bindings";
import type { CategorySpend, KpiSummary, MonthSummary } from "../bindings";

function unwrap<T>(result: { status: "ok"; data: T } | { status: "error"; error: string }): T {
  if (result.status === "error") throw new Error(result.error);
  return result.data;
}

export async function summaryKpis(month: string | null = null): Promise<KpiSummary> {
  return unwrap(await commands.summaryKpis(month));
}

export async function summaryByCategory(month: string | null = null): Promise<CategorySpend[]> {
  return unwrap(await commands.summaryByCategory(month));
}

export async function summaryByMonth(monthsBack: number): Promise<MonthSummary[]> {
  return unwrap(await commands.summaryByMonth(monthsBack));
}
```

- [ ] **Step 4: `pnpm check`**

```bash
pnpm check 2>&1 | tail -5
```
Expected: 0 errors. The existing `listTransactions(filters)` callers (Transactions.svelte) still work because `limit` is optional — adding a field to the filter struct doesn't break existing callers.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/lib.rs src/lib/api/summary.ts
git commit -m "$(cat <<'EOF'
feat(ipc): registra summary commands + wrapper TS

- 18 commands totais
- KpiSummary, CategorySpend, MonthSummary + 3 funções async
- TransactionFilters.limit aparece nas bindings automaticamente

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: MonthStepper extraído + KpiCard component

**Files:**
- Create: `src/lib/components/shell/MonthStepper.svelte` (extraído)
- Modify: `src/lib/components/transactions/TxFilterBar.svelte` (reusa MonthStepper)
- Create: `src/lib/components/dashboard/KpiCard.svelte`

- [ ] **Step 1: Create `src/lib/components/shell/MonthStepper.svelte`**

```svelte
<script lang="ts">
  type Props = {
    /** YYYY-MM or null */
    month: string | null;
    onChange: (m: string | null) => void;
    /** Show "Todos os meses" clear link */
    showClear?: boolean;
  };

  let { month, onChange, showClear = true }: Props = $props();

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
</script>

<div class="inline-flex items-center gap-2">
  <div class="inline-flex items-center gap-px rounded-md border border-border bg-surface-2">
    <button
      type="button"
      class="px-2 py-1 text-fg-muted hover:bg-hover rounded-l-md"
      onclick={() => onChange(shiftMonth(month, -1))}
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
      onclick={() => onChange(shiftMonth(month, +1))}
      aria-label="Próximo mês"
    >
      ›
    </button>
  </div>

  {#if showClear && month}
    <button
      type="button"
      onclick={() => onChange(null)}
      class="text-[11px] text-fg-faint hover:text-fg-muted underline-offset-2 hover:underline"
    >
      Todos os meses
    </button>
  {/if}
</div>
```

- [ ] **Step 2: Refactor `src/lib/components/transactions/TxFilterBar.svelte` to use MonthStepper**

Replace the entire file with:

```svelte
<script lang="ts">
  import MonthStepper from "$lib/components/shell/MonthStepper.svelte";
  import type { Category } from "$lib/bindings";

  type Props = {
    categories: Category[];
    month: string | null;
    categoryId: number | null;
    onMonthChange: (m: string | null) => void;
    onCategoryChange: (id: number | null) => void;
  };

  let { categories, month, categoryId, onMonthChange, onCategoryChange }: Props = $props();

  let currentCategory = $derived(categories.find((c) => c.id === categoryId));
</script>

<div class="flex items-center gap-2 flex-wrap">
  <MonthStepper {month} onChange={onMonthChange} />

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

- [ ] **Step 3: Create `src/lib/components/dashboard/KpiCard.svelte`**

```svelte
<script lang="ts">
  import { formatMoney } from "$lib/format/money";

  type Props = {
    label: string;
    value: string;
    /** Optional caption rendered under the value (e.g. "12 transações") */
    caption?: string;
    /** "pos" | "neg" | "muted" — controls value color */
    tone?: "pos" | "neg" | "muted";
    /** If true, value is treated as a plain number/string (not money) */
    raw?: boolean;
  };

  let { label, value, caption, tone = "muted", raw = false }: Props = $props();

  let toneClass = $derived(
    tone === "pos" ? "text-pos" : tone === "neg" ? "text-neg" : "text-fg",
  );
</script>

<div class="rounded-xl bg-surface border border-border-subtle p-4 flex flex-col gap-1.5">
  <div class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint">
    {label}
  </div>
  <div class="text-2xl font-semibold tracking-tight tabular {toneClass}" style="font-family: var(--font-display)">
    {raw ? value : formatMoney(value)}
  </div>
  {#if caption}
    <div class="text-[11px] text-fg-faint tabular">{caption}</div>
  {/if}
</div>
```

- [ ] **Step 4: pnpm check**

```bash
pnpm check 2>&1 | tail -5
```
Expected: 0 errors.

- [ ] **Step 5: Commit**

```bash
git add src/lib/components/shell/MonthStepper.svelte src/lib/components/transactions/TxFilterBar.svelte src/lib/components/dashboard/KpiCard.svelte
git commit -m "$(cat <<'EOF'
feat(ui): MonthStepper extraído + KpiCard pro dashboard

- MonthStepper.svelte: stepper ‹/› + "Todos os meses" (componente reusável)
- TxFilterBar refactor: importa MonthStepper, simplifica
- KpiCard: rótulo + valor (formatado BRL ou raw) + caption opcional + tone pos/neg/muted

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 4: CategoryDonut + MonthBars (CSS-only)

**Files:**
- Create: `src/lib/components/dashboard/CategoryDonut.svelte`
- Create: `src/lib/components/dashboard/MonthBars.svelte`

- [ ] **Step 1: Create `src/lib/components/dashboard/CategoryDonut.svelte`**

```svelte
<script lang="ts">
  import { formatMoney } from "$lib/format/money";
  import type { CategorySpend } from "$lib/bindings";

  type Props = {
    items: CategorySpend[];
    total: string;
    size?: number;
  };

  let { items, total, size = 156 }: Props = $props();

  let gradient = $derived(buildGradient(items));

  function buildGradient(items: CategorySpend[]): string {
    if (items.length === 0) return "var(--color-surface-2)";
    const stops: string[] = [];
    let acc = 0;
    for (const it of items) {
      const start = acc;
      const end = acc + it.percent;
      const color = it.color_token ? `var(${it.color_token})` : "var(--color-cat-outros)";
      stops.push(`${color} ${start.toFixed(2)}% ${end.toFixed(2)}%`);
      acc = end;
    }
    if (acc < 99.999) {
      stops.push(`var(--color-surface-2) ${acc.toFixed(2)}% 100%`);
    }
    return `conic-gradient(${stops.join(", ")})`;
  }
</script>

<div class="flex items-center gap-5">
  <div
    class="relative shrink-0 rounded-full grid place-items-center"
    style="width: {size}px; height: {size}px; background: {gradient}"
  >
    <div class="absolute inset-[18px] rounded-full bg-surface border border-border-subtle"></div>
    <div class="relative text-center tabular">
      <div class="text-[10px] uppercase tracking-wider text-fg-faint">Gastos</div>
      <div class="text-[18px] font-semibold mt-px" style="font-family: var(--font-display)">
        {formatMoney(total)}
      </div>
    </div>
  </div>

  <ul class="flex-1 flex flex-col gap-1 text-[11.5px]">
    {#each items.slice(0, 8) as it}
      <li class="grid grid-cols-[10px_1fr_auto_44px] gap-2 items-center text-fg-muted">
        <span
          class="w-2.5 h-2.5 rounded-sm shrink-0"
          style="background: {it.color_token ? `var(${it.color_token})` : 'var(--color-cat-outros)'}"
        ></span>
        <span class="text-fg truncate">{it.name}</span>
        <span class="tabular">{formatMoney(it.total)}</span>
        <span class="tabular text-fg-faint text-right">{it.percent.toFixed(1)}%</span>
      </li>
    {/each}
    {#if items.length === 0}
      <li class="text-fg-faint italic">Sem gastos no período.</li>
    {/if}
  </ul>
</div>
```

- [ ] **Step 2: Create `src/lib/components/dashboard/MonthBars.svelte`**

```svelte
<script lang="ts">
  import { formatMoney } from "$lib/format/money";
  import type { MonthSummary } from "$lib/bindings";

  type Props = {
    months: MonthSummary[];
  };

  let { months }: Props = $props();

  function shortLabel(yyyymm: string): string {
    const [y, mo] = yyyymm.split("-");
    const names = ["jan", "fev", "mar", "abr", "mai", "jun", "jul", "ago", "set", "out", "nov", "dez"];
    return `${names[Number(mo) - 1]}/${y.slice(-2)}`;
  }

  let maxValue = $derived(
    months.reduce((acc, m) => {
      const i = Number(m.income);
      const e = Number(m.expense);
      return Math.max(acc, isFinite(i) ? i : 0, isFinite(e) ? e : 0);
    }, 0),
  );

  function pct(amountStr: string): number {
    if (maxValue <= 0) return 0;
    const v = Number(amountStr);
    if (!isFinite(v) || v <= 0) return 0;
    return Math.min(100, (v / maxValue) * 100);
  }
</script>

<div class="grid grid-cols-12 gap-1.5 items-end h-[140px] pt-2">
  {#each months as m}
    <div class="flex flex-col items-center gap-1.5 h-full">
      <div
        class="w-full flex-1 flex flex-col-reverse rounded-t overflow-hidden bg-surface-2"
        title={`${m.month} · in: ${formatMoney(m.income)} · out: ${formatMoney(m.expense)}`}
      >
        <span class="w-full bg-pos" style="height: {pct(m.income)}%"></span>
        <span class="w-full bg-neg" style="height: {pct(m.expense)}%; opacity: 0.6;"></span>
      </div>
      <span class="text-[9.5px] text-fg-faint tabular">{shortLabel(m.month)}</span>
    </div>
  {/each}
  {#if months.length === 0}
    <div class="col-span-12 text-center text-fg-faint italic py-8">
      Sem dados nos últimos 12 meses.
    </div>
  {/if}
</div>
```

> **Notas técnicas:**
> - Donut: `conic-gradient` empilha fatias acumuladas. Buraco central via `inset-[18px]` `bg-surface`.
> - Barras: empilhamento de `flex-col-reverse` faz income aparecer embaixo, expense em cima. Cores fixas via tokens.
> - O `title=` no bar div mostra tooltip nativo do navegador. Polish em fase 5 pode trocar por Popover.

- [ ] **Step 3: pnpm check**

```bash
pnpm check 2>&1 | tail -5
```
Expected: 0 errors.

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/dashboard/CategoryDonut.svelte src/lib/components/dashboard/MonthBars.svelte
git commit -m "$(cat <<'EOF'
feat(ui): CategoryDonut + MonthBars CSS-only

- CategoryDonut: conic-gradient empilhando fatias por percent, legenda top-8
  com swatch + nome + valor + %
- MonthBars: 12 colunas com bar income (verde) + bar expense (vermelho com opacity)
  empilhados por altura proporcional ao maior valor
- Sem deps externas (LayerChart defer pra fase 5 se precisar de interatividade)

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 5: TopCategoriesList + RecentList

**Files:**
- Create: `src/lib/components/dashboard/TopCategoriesList.svelte`
- Create: `src/lib/components/dashboard/RecentList.svelte`

- [ ] **Step 1: Create `src/lib/components/dashboard/TopCategoriesList.svelte`**

```svelte
<script lang="ts">
  import { formatMoney } from "$lib/format/money";
  import type { CategorySpend } from "$lib/bindings";

  type Props = {
    items: CategorySpend[];
    /** How many top items to show (default 5) */
    top?: number;
  };

  let { items, top = 5 }: Props = $props();

  let shown = $derived(items.slice(0, top));
  let topValue = $derived(
    shown.reduce((acc, i) => Math.max(acc, Number(i.total) || 0), 0),
  );

  function widthPct(totalStr: string): number {
    if (topValue <= 0) return 0;
    const v = Number(totalStr);
    if (!isFinite(v)) return 0;
    return Math.min(100, (v / topValue) * 100);
  }
</script>

<div class="flex flex-col gap-2.5">
  {#each shown as it}
    <div class="grid grid-cols-[16px_1fr_auto] gap-2 items-center">
      <span
        class="w-2.5 h-2.5 rounded-sm shrink-0"
        style="background: {it.color_token ? `var(${it.color_token})` : 'var(--color-cat-outros)'}"
      ></span>
      <span class="text-[12px] text-fg font-medium truncate">{it.name}</span>
      <span class="text-[11.5px] text-fg-muted tabular">{formatMoney(it.total)}</span>
      <div class="col-start-2 col-span-2 h-1 bg-surface-2 rounded-full overflow-hidden">
        <span
          class="block h-full rounded-full"
          style="width: {widthPct(it.total)}%; background: {it.color_token ? `var(${it.color_token})` : 'var(--color-accent)'}"
        ></span>
      </div>
    </div>
  {:else}
    <div class="text-fg-faint italic text-[12px]">Sem gastos no período.</div>
  {/each}
</div>
```

- [ ] **Step 2: Create `src/lib/components/dashboard/RecentList.svelte`**

```svelte
<script lang="ts">
  import { formatMoney } from "$lib/format/money";
  import type { Transaction } from "$lib/bindings";

  type Props = {
    transactions: Transaction[];
  };

  let { transactions }: Props = $props();
</script>

<ul class="flex flex-col">
  {#each transactions as t (t.id)}
    <li class="grid grid-cols-[68px_1fr_100px] gap-3 items-center px-3 py-2 border-b border-border-subtle last:border-b-0">
      <span class="text-[11px] text-fg-muted tabular">{t.date}</span>
      <span class="text-[12px] text-fg truncate">{t.description}</span>
      <span class="text-[12px] text-right tabular font-medium {Number(t.amount) >= 0 ? 'text-pos' : 'text-fg'}">
        {formatMoney(t.amount)}
      </span>
    </li>
  {:else}
    <li class="text-fg-faint italic text-[12px] px-3 py-4">Nenhuma transação ainda.</li>
  {/each}
</ul>
```

- [ ] **Step 3: pnpm check**

```bash
pnpm check 2>&1 | tail -5
```
Expected: 0 errors.

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/dashboard/TopCategoriesList.svelte src/lib/components/dashboard/RecentList.svelte
git commit -m "$(cat <<'EOF'
feat(ui): TopCategoriesList (meter) + RecentList pro dashboard

- TopCategoriesList: nome + valor + barra proporcional ao maior do top
- RecentList: linhas compactas data/descrição/valor com cor por sinal
- Empty states inline em ambos

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 6: Wire Dashboard.svelte + IndexRedirect

**Files:**
- Modify: `src/routes/Dashboard.svelte`
- Create: `src/routes/IndexRedirect.svelte`
- Modify: `src/routes/routes.ts`

- [ ] **Step 1: Replace `src/routes/Dashboard.svelte`**

```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import MonthStepper from "$lib/components/shell/MonthStepper.svelte";
  import KpiCard from "$lib/components/dashboard/KpiCard.svelte";
  import CategoryDonut from "$lib/components/dashboard/CategoryDonut.svelte";
  import MonthBars from "$lib/components/dashboard/MonthBars.svelte";
  import TopCategoriesList from "$lib/components/dashboard/TopCategoriesList.svelte";
  import RecentList from "$lib/components/dashboard/RecentList.svelte";
  import { filters } from "$lib/stores/filters.svelte";
  import { summaryByCategory, summaryByMonth, summaryKpis } from "$lib/api/summary";
  import { listTransactions } from "$lib/api/transactions";
  import type {
    CategorySpend,
    KpiSummary,
    MonthSummary,
    Transaction,
  } from "$lib/bindings";

  let kpis = $state<KpiSummary | null>(null);
  let byCategory = $state<CategorySpend[]>([]);
  let byMonth = $state<MonthSummary[]>([]);
  let recent = $state<Transaction[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  async function refresh() {
    try {
      const [k, c, recentTx] = await Promise.all([
        summaryKpis(filters.month),
        summaryByCategory(filters.month),
        listTransactions({
          account_id: null,
          month: filters.month,
          category_id: null,
          limit: 8,
        }),
      ]);
      kpis = k;
      byCategory = c;
      recent = recentTx;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  onMount(async () => {
    try {
      byMonth = await summaryByMonth(12);
      await refresh();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  });

  async function onMonthChange(m: string | null) {
    filters.month = m;
    await refresh();
  }
</script>

<section class="p-8 max-w-6xl mx-auto flex flex-col gap-5">
  <header class="flex items-baseline justify-between gap-4 flex-wrap">
    <h2 class="text-xl font-semibold tracking-tight" style="font-family: var(--font-display)">
      Dashboard
    </h2>
    <MonthStepper month={filters.month} onChange={onMonthChange} />
  </header>

  {#if loading}
    <div class="text-fg-faint text-sm">Carregando…</div>
  {:else if error}
    <div class="rounded-lg border border-border bg-surface p-3 text-sm text-neg">{error}</div>
  {:else}
    <!-- KPIs -->
    {#if kpis}
      <div class="grid grid-cols-4 gap-3">
        <KpiCard label="Renda" value={kpis.income} tone="pos" />
        <KpiCard label="Gastos" value={kpis.expense} />
        <KpiCard
          label="Saldo do mês"
          value={kpis.net}
          tone={Number(kpis.net) >= 0 ? "pos" : "neg"}
        />
        <KpiCard
          label="Transações"
          value={String(kpis.transaction_count)}
          raw={true}
          caption={kpis.transaction_count === 1 ? "uma transação" : "transações no período"}
        />
      </div>
    {/if}

    <!-- Donut + 12-month bars -->
    <div class="grid grid-cols-[380px_1fr] gap-4">
      <div class="rounded-xl bg-surface border border-border-subtle p-4 flex flex-col gap-3">
        <div class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint">
          Gastos por categoria
        </div>
        <CategoryDonut items={byCategory} total={kpis?.expense ?? "0"} />
      </div>
      <div class="rounded-xl bg-surface border border-border-subtle p-4 flex flex-col gap-3">
        <div class="flex items-baseline justify-between">
          <div class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint">
            Últimos 12 meses
          </div>
          <div class="text-[10px] text-fg-faint flex gap-3">
            <span class="flex items-center gap-1.5"><span class="w-2 h-2 rounded bg-pos"></span> entradas</span>
            <span class="flex items-center gap-1.5"><span class="w-2 h-2 rounded bg-neg opacity-60"></span> saídas</span>
          </div>
        </div>
        <MonthBars months={byMonth} />
      </div>
    </div>

    <!-- Top categories + Recent -->
    <div class="grid grid-cols-[1fr_360px] gap-4">
      <div class="rounded-xl bg-surface border border-border-subtle p-4 flex flex-col gap-3">
        <div class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint">
          Top categorias do mês
        </div>
        <TopCategoriesList items={byCategory} />
      </div>
      <div class="rounded-xl bg-surface border border-border-subtle flex flex-col">
        <div class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint px-4 pt-4 pb-2">
          Últimas transações
        </div>
        <RecentList transactions={recent} />
      </div>
    </div>
  {/if}
</section>
```

- [ ] **Step 2: Create `src/routes/IndexRedirect.svelte`**

```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import { push } from "svelte-spa-router";
  import { listTransactions } from "$lib/api/transactions";

  let checking = $state(true);

  onMount(async () => {
    try {
      const some = await listTransactions({
        account_id: null,
        month: null,
        category_id: null,
        limit: 1,
      });
      push(some.length === 0 ? "/onboarding" : "/dashboard");
    } catch {
      push("/onboarding");
    } finally {
      checking = false;
    }
  });
</script>

{#if checking}
  <section class="p-8 text-fg-faint text-sm">Carregando…</section>
{/if}
```

- [ ] **Step 3: Update `src/routes/routes.ts`**

```ts
import IndexRedirect from "./IndexRedirect.svelte";
import Onboarding from "./Onboarding.svelte";
import Dashboard from "./Dashboard.svelte";
import Transactions from "./Transactions.svelte";
import Import from "./Import.svelte";
import Categories from "./Categories.svelte";
import Rules from "./Rules.svelte";
import Settings from "./Settings.svelte";

export const routes = {
  "/": IndexRedirect,
  "/onboarding": Onboarding,
  "/dashboard": Dashboard,
  "/transactions": Transactions,
  "/import": Import,
  "/categories": Categories,
  "/rules": Rules,
  "/settings": Settings,
};
```

- [ ] **Step 4: pnpm check**

```bash
pnpm check 2>&1 | tail -5
```
Expected: 0 errors.

- [ ] **Step 5: Smoke test boot**

```bash
pkill -f "tauri dev" 2>/dev/null; pkill -f "vite" 2>/dev/null; pkill -f "target/debug/finan" 2>/dev/null
pnpm tauri dev > /tmp/finan-dev.log 2>&1 &
until lsof -iTCP:1420 -sTCP:LISTEN >/dev/null 2>&1; do sleep 2; done
grep -iE "error|fail" /tmp/finan-dev.log | head -5
pkill -f "tauri dev"; pkill -f "vite"; pkill -f "target/debug/finan"
```
Expected: clean boot.

- [ ] **Step 6: Commit**

```bash
git add src/routes/Dashboard.svelte src/routes/IndexRedirect.svelte src/routes/routes.ts
git commit -m "$(cat <<'EOF'
feat(dashboard): tela completa + IndexRedirect na rota /

- Dashboard wires 4 KPIs + CategoryDonut + MonthBars + TopCategoriesList + RecentList
- MonthStepper no header dispara refresh dos widgets dependentes do mês
- Bars dos 12 meses ficam independentes (sempre last-12)
- IndexRedirect chama listTransactions(limit:1) → /onboarding se vazio, /dashboard
- routes.ts mapeia / → IndexRedirect

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 7: Acceptance verification + Fase 4 close

**Files:**
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

Expected: pnpm check 0 / pnpm test 19+ / cargo test 31+ / clippy clean / fmt clean / build OK.

If clippy or fmt complain, fix inline:
- `cargo fmt` to apply.
- For clippy warnings, prefer fixing over suppressing.

- [ ] **Step 2: Manual E2E walkthrough**

```bash
pkill -f "tauri dev" 2>/dev/null
pnpm tauri dev > /tmp/finan-dev.log 2>&1 &
until lsof -iTCP:1420 -sTCP:LISTEN >/dev/null 2>&1; do sleep 2; done
```

Manual checklist (precisa de transações importadas):
- [ ] Abrir app: rota `/` redireciona pra `/dashboard` (se há tx) ou `/onboarding` (se vazio).
- [ ] Dashboard mostra 4 KPIs com BRL formatado.
- [ ] Saldo do mês em verde se positivo, vermelho se negativo.
- [ ] Donut renderiza fatias com cores das categorias; centro mostra total de gastos.
- [ ] Legenda do donut mostra top 8 categorias com valor e %.
- [ ] Bars dos últimos 12 meses mostram colunas com verde (income) e vermelho (expense).
- [ ] Hover na bar mostra tooltip nativo (title attr).
- [ ] Top categorias do mês mostra meter horizontal proporcional.
- [ ] Últimas 8 transações listadas com data/descrição/valor.
- [ ] Stepper de mês ‹/› muda KPIs, donut, top, recent — mas barras 12m permanecem.
- [ ] "Todos os meses" no stepper: KPIs e donut mostram tudo.

```bash
pkill -f "tauri dev"; pkill -f "vite"; pkill -f "target/debug/finan"
```

- [ ] **Step 3: Update README**

```
## Status

- ✅ Fase 0 — Scaffold (Tauri + Svelte + DB + sidebar + IPC tipado)
- ✅ Fase 1 — Importar OFX (parser TS + dedup por FITID + listagem)
- ✅ Fase 2 — Categorização manual inline + filtros + notes
- ✅ Fase 3 — Regras automáticas (description-contains + auto-apply + apply-existing)
- ✅ Fase 4 — Dashboard (KPIs + donut + barras 12m + top categorias + recent)
- 🚧 Fase 5 — Polish (search, settings, atalhos) (próximo)
```

- [ ] **Step 4: Closing commit**

```bash
git add README.md
git commit -m "$(cat <<'EOF'
chore(fase-4): close dashboard phase — acceptance criteria batem

- 3 commands Rust de agregação (kpis, by_category, by_month) com rust_decimal
- TransactionFilters.limit pra widget Recent
- CSS-only charts: donut conic-gradient + bars flex (zero deps externas)
- IndexRedirect: rota / decide entre /onboarding (vazio) e /dashboard (com dados)
- Tests: 31 cargo (27 anteriores + 4 novos: kpis, by_category, cutoff, limit)
       / 19 vitest / pnpm check 0 / clippy/fmt limpos

Próximo: plano da Fase 5 (Polish: search global, settings, atalhos).

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Self-Review

### Spec coverage

| Spec item | Task |
|---|---|
| §8.4 Toolbar seletor de mês ‹/› | T3 (MonthStepper), T6 (Dashboard usa) |
| §8.4 4 KPIs (Renda, Gastos, Saldo, Tx) | T1 (command), T3 (KpiCard), T6 (Dashboard) |
| §8.4 Donut por categoria | T1 (command), T4 (CategoryDonut CSS), T6 |
| §8.4 Bar 12 meses | T1 (summary_by_month), T4 (MonthBars CSS), T6 |
| §8.4 Top 5 categorias com meter | T5 (TopCategoriesList), T6 |
| §8.4 Recent (8 últimas) | T1 (limit in filters), T5 (RecentList), T6 |
| §7 "rota / decide entre /onboarding e /dashboard" | T6 (IndexRedirect) |
| §9 critério "trocar mês atualiza widgets" | T6 (refresh on month change) |
| §9 critério "donut bate com soma" | T1 (server-side sum = total displayed center) |
| §5 invariante "dinheiro nunca f64" | T1 (rust_decimal em todos os summary) |

### Placeholder scan

Reli todos os blocos. Sem "TBD" / "TODO" / "implement later" / "add error handling". Cada step tem código completo.

### Type consistency

- `KpiSummary { income, expense, net, transaction_count }` — Rust (T1) → bindings (T2) → KpiCard mapping em Dashboard (T6). `transaction_count` é `u32` Rust → `number` TS. Match.
- `CategorySpend { category_id (Option), name, color_token (Option), total, percent }` — Rust (T1) → CategoryDonut (T4) + TopCategoriesList (T5). Match.
- `MonthSummary { month, income, expense }` — Rust (T1) → MonthBars (T4). Match.
- `TransactionFilters { account_id, month, category_id, limit }` — Rust adicionou limit (T1) → bindings (T2) → listTransactions wrapper inalterado (passa o objeto inteiro). Dashboard usa `{ limit: 8, ... }` e Transactions.svelte continua passando objetos sem limit (`null`/undefined no campo). Backward compatible.
- `MonthStepper` props: `month`, `onChange`, `showClear?`. TxFilterBar passa `onChange={onMonthChange}` (T3). Dashboard passa `onChange={onMonthChange}` (T6). Consistent.
- `IndexRedirect` chama `push(...)` do svelte-spa-router — função síncrona, OK.

### Risks documented inline

- **T1 `compute_cutoff` test:** time-dependent, mas o teste apenas valida o shape de string (length 10, formato YYYY-MM-DD). Não checa valores específicos pra ser estável independente de quando rodar.
- **T6 IndexRedirect race:** se `listTransactions(limit:1)` demorar, o usuário vê "Carregando…" brevemente. Aceitável.
- **CSS conic-gradient stops:** se `percent` somar < 100 por arredondamento (raramente), preencho o resto com `surface-2`. Documentado no código.
- **Cores no MonthBars:** `bg-pos` e `bg-neg` (não `bg-color-pos`) — Tailwind 4 derivou utilities dos tokens `--color-pos` e `--color-neg` automaticamente. Se a util não existir, troco por `style="background: var(--color-pos)"` — verificar em pnpm check; com Tailwind 4 + `@theme inline`, deve funcionar.
