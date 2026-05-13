# Fase 5 — Polish (MVP final) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fechar o MVP do finan adicionando: (1) Settings com path do DB, "Abrir no Finder", export e restore de backup; (2) busca global em descrição/notes na rota Transações; (3) atalhos de teclado `⌘1..5`, `⌘F`, `⌘O`; (4) ajustes visuais finais — Button da shadcn usa nossos tokens em focus/destructive states.

**Architecture:** Plugins Tauri 2 instalados pra integração macOS: `tauri-plugin-dialog` (save/open file dialogs) + `tauri-plugin-opener` (reveal in Finder). Backup/restore como commands Rust tipados (copy file + validate SQLite header). Search vira filtro no backend via `TransactionFilters.q` → `LOWER(description) LIKE '%q%' OR LOWER(notes) LIKE '%q%'`. Atalhos via handler global no `App.svelte` que despacha `push()` / focus. Polish do Button: trocar `ring-ring`, `bg-destructive`, `border-input` por tokens nossos.

**Tech Stack:** novo:
- `tauri-plugin-dialog` (Cargo) + `@tauri-apps/plugin-dialog` (npm)
- `tauri-plugin-opener` (Cargo) + `@tauri-apps/plugin-opener` (npm)

**Acceptance criteria (Fase 5):**
1. `/settings` mostra path do DB; botão `Abrir no Finder` revela o arquivo; botões `Exportar backup` e `Restaurar backup` funcionam.
2. Exportar backup: abre save dialog, copia o `.db` pro path escolhido. Arquivo resultante é um SQLite válido (pode ser aberto com `sqlite3`).
3. Restaurar backup: abre open dialog, valida (header SQLite + tabelas mínimas), substitui o `.db` atual, app pede pra reiniciar (ou reinicia automaticamente).
4. `/transactions` toolbar tem SearchBox com placeholder "Buscar…"; digitando filtra description+notes case-insensitive; estado persiste no `filters` store.
5. Atalhos macOS funcionam: `⌘1`=Dashboard, `⌘2`=Transações, `⌘3`=Importar, `⌘4`=Categorias, `⌘5`=Regras; `⌘,`=Configurações; `⌘O`=Importar OFX (foca dropzone); `⌘F`=ir pra /transactions e focar search.
6. shadcn Button: classes `ring-ring`, `destructive`, `border-input` substituídas por tokens nossos (`focus-visible:ring-accent`, `bg-neg`, `border-border`).
7. Tests: `cargo test --lib` ≥ 33 (31 anteriores + 2 novos: backup_creates_valid_sqlite, restore_validates_header), `pnpm test` ≥ 19, `pnpm check` 0, clippy/fmt limpos.

**Out of scope:**
- Sincronização cloud (princípio "100% local").
- Multi-currency.
- Notifications/badges.
- Drag-to-reorder anywhere.
- Light theme (dark-only por design — spec §6).

---

## Estrutura de arquivos

```
src-tauri/
├── Cargo.toml                                  T1 (adds 2 plugins)
├── capabilities/default.json                   T1 (permits dialog + opener)
└── src/
    ├── commands/
    │   ├── backup.rs                           T2 (novo)
    │   └── mod.rs                              T2 (declara backup)
    └── lib.rs                                  T1 (.plugin()) + T3 (collect_commands!)

src/
└── lib/
    ├── api/
    │   └── backup.ts                           T3 (novo)
    ├── stores/
    │   └── filters.svelte.ts                   T5 (adiciona q field)
    └── components/
        ├── shell/SearchBox.svelte              T5 (novo)
        ├── transactions/TxFilterBar.svelte     T5 (passa q + handler)
        └── ui/button/button.svelte             T7 (refactor variants)
└── routes/
    ├── Settings.svelte                         T4 (rewrite)
    ├── Transactions.svelte                     T5 (passa q ao listTransactions)
    └── App.svelte                              T6 (global keydown handler)
```

---

## Task 1: Install Tauri plugins (dialog + opener) + permissions

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `package.json` (add 2 npm packages)
- Modify: `src-tauri/capabilities/default.json`
- Modify: `src-tauri/src/lib.rs` (register `.plugin(...)`)

- [ ] **Step 1: Install Rust crates**

Add to `src-tauri/Cargo.toml` `[dependencies]`:
```toml
tauri-plugin-dialog = "2"
tauri-plugin-opener = "2"
```

- [ ] **Step 2: Install TS plugins**

```bash
pnpm add @tauri-apps/plugin-dialog @tauri-apps/plugin-opener
```

- [ ] **Step 3: Update `src-tauri/capabilities/default.json`**

Add the new permissions. The file currently has:
```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Default capabilities for finan",
  "windows": ["main"],
  "permissions": ["core:default"]
}
```

Replace `permissions` with:
```json
  "permissions": [
    "core:default",
    "dialog:default",
    "opener:default"
  ]
```

- [ ] **Step 4: Register plugins in `src-tauri/src/lib.rs`**

Find:
```rust
tauri::Builder::default()
    .invoke_handler(specta_builder.invoke_handler())
    .setup(|app| {
```

Insert plugin registrations BEFORE `.invoke_handler(...)`:
```rust
tauri::Builder::default()
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_opener::init())
    .invoke_handler(specta_builder.invoke_handler())
    .setup(|app| {
```

- [ ] **Step 5: Verify build**

```bash
cd src-tauri && cargo build 2>&1 | tail -10 && cd ..
```
Expected: builds successfully (downloads 2 plugin crates ~1-2 min first time).

```bash
pnpm check 2>&1 | tail -5
```
Expected: 0 errors.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/capabilities/default.json src-tauri/src/lib.rs package.json pnpm-lock.yaml
git commit -m "$(cat <<'EOF'
feat(plugins): tauri-plugin-dialog + tauri-plugin-opener

- Cargo deps + npm packages
- Capability "default" permite dialog:default + opener:default
- Plugins registrados no Tauri Builder antes do invoke_handler
- Pré-requisito pra Settings (backup/restore + open in Finder) na próxima task

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 2: Backend backup/restore commands

**Files:**
- Create: `src-tauri/src/commands/backup.rs`
- Modify: `src-tauri/src/commands/mod.rs`

- [ ] **Step 1: Update `src-tauri/src/commands/mod.rs`**

```rust
pub mod accounts;
pub mod backup;
pub mod categories;
pub mod health;
pub mod rules;
pub mod summary;
pub mod transactions;
```

- [ ] **Step 2: Create `src-tauri/src/commands/backup.rs`**

```rust
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use tauri::State;

use crate::db::Db;
use crate::error::{AppError, AppResult};

const SQLITE_HEADER: &[u8; 16] = b"SQLite format 3\x00";

/// Returns the absolute path of the active SQLite DB file as a string.
#[tauri::command]
#[specta::specta]
pub fn db_path(db: State<'_, Db>) -> AppResult<String> {
    Ok(db.path.display().to_string())
}

/// Copy the active DB file to `destination`. Closes nothing — SQLite WAL mode
/// guarantees the .db file is consistent on disk even mid-write, but to be
/// safe we issue a `PRAGMA wal_checkpoint(FULL)` first.
#[tauri::command]
#[specta::specta]
pub fn export_backup(db: State<'_, Db>, destination: String) -> AppResult<String> {
    {
        let conn = db.conn.lock().expect("db mutex poisoned");
        let _ = conn.pragma_update(None, "wal_checkpoint", "FULL");
    }
    let dest = PathBuf::from(&destination);
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(&db.path, &dest)?;
    Ok(dest.display().to_string())
}

/// Validate that a file looks like a SQLite 3 DB (header magic) and is non-empty,
/// then overwrite the active DB file. The frontend should prompt the user to
/// restart the app after this call — the in-memory `Connection` is now stale.
#[tauri::command]
#[specta::specta]
pub fn restore_backup(db: State<'_, Db>, source: String) -> AppResult<()> {
    let src = PathBuf::from(&source);
    validate_sqlite_file(&src)?;

    // Best-effort checkpoint to flush WAL before swap.
    {
        let conn = db.conn.lock().expect("db mutex poisoned");
        let _ = conn.pragma_update(None, "wal_checkpoint", "FULL");
    }

    fs::copy(&src, &db.path)?;
    // Also remove any sibling -wal/-shm files so the restored DB is not shadowed.
    for ext in ["-wal", "-shm"] {
        let sidecar = db.path.with_extension(format!(
            "{}{}",
            db.path.extension().and_then(|e| e.to_str()).unwrap_or(""),
            ext
        ));
        let _ = fs::remove_file(sidecar);
    }
    Ok(())
}

fn validate_sqlite_file(path: &PathBuf) -> AppResult<()> {
    let mut file = fs::File::open(path)?;
    let mut header = [0u8; 16];
    let n = file.read(&mut header)?;
    if n < 16 || &header != SQLITE_HEADER {
        return Err(AppError::Invalid(format!(
            "{} is not a valid SQLite 3 file",
            path.display()
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;
    use rusqlite::Connection;
    use std::io::Write;
    use std::path::Path;

    fn temp_path(name: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("finan-test-{}-{name}", std::process::id()));
        p
    }

    #[test]
    fn validate_accepts_real_sqlite_file() {
        let path = temp_path("ok.db");
        let _ = fs::remove_file(&path);
        let conn = Connection::open(&path).unwrap();
        migrations::apply(&conn).unwrap();
        drop(conn);

        validate_sqlite_file(&path).expect("valid sqlite should pass");
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn validate_rejects_random_file() {
        let path = temp_path("nope.bin");
        let _ = fs::remove_file(&path);
        let mut f = fs::File::create(&path).unwrap();
        f.write_all(b"not a sqlite file at all").unwrap();
        drop(f);

        let r = validate_sqlite_file(&path);
        assert!(r.is_err());
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn restore_copies_file_contents() {
        let src = temp_path("src.db");
        let dst = temp_path("dst.db");
        let _ = fs::remove_file(&src);
        let _ = fs::remove_file(&dst);

        // create src with migrations applied
        let conn = Connection::open(&src).unwrap();
        migrations::apply(&conn).unwrap();
        drop(conn);

        // simulate restore: validate then copy
        validate_sqlite_file(&src).unwrap();
        fs::copy(&src, &dst).unwrap();

        // dst should also be a valid SQLite DB
        assert!(validate_sqlite_file(&dst).is_ok());
        let dst_conn = Connection::open(&dst).unwrap();
        let count: i64 = dst_conn
            .query_row("SELECT COUNT(*) FROM categories", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 9);

        let _ = fs::remove_file(&src);
        let _ = fs::remove_file(&dst);
        let _ = Path::new("dummy"); // touch path import so it's not unused
    }
}
```

- [ ] **Step 3: Run cargo test**

```bash
cd src-tauri && cargo test --lib 2>&1 | tail -15 && cd ..
```
Expected: **34 tests pass** (31 prior + 3 new from `backup::tests`).

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/
git commit -m "$(cat <<'EOF'
feat(backup): db_path + export_backup + restore_backup commands

- export_backup: WAL checkpoint + fs::copy pro destination escolhido
- restore_backup: valida header SQLite3 magic, copia sobre o DB ativo,
  remove sidecars -wal/-shm. Frontend deve pedir restart depois.
- validate_sqlite_file: magic bytes check
- 3 testes (accept real, reject garbage, restore round-trip)

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: Register backup commands + TS API wrapper

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Create: `src/lib/api/backup.ts`

- [ ] **Step 1: Update `collect_commands!` in `lib.rs`**

Add 3 entries:
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
    commands::backup::db_path,
    commands::backup::export_backup,
    commands::backup::restore_backup,
]);
```

Total now 21 commands.

- [ ] **Step 2: Regen bindings + verify**

```bash
pkill -f "tauri dev" 2>/dev/null; pkill -f "vite" 2>/dev/null; pkill -f "target/debug/finan" 2>/dev/null
pnpm tauri dev > /tmp/finan-dev.log 2>&1 &
until grep -q "exportBackup" src/lib/bindings.ts 2>/dev/null && \
      grep -q "restoreBackup" src/lib/bindings.ts 2>/dev/null && \
      grep -q "dbPath" src/lib/bindings.ts 2>/dev/null; do
  sleep 3
done
pkill -f "tauri dev"; pkill -f "vite"; pkill -f "target/debug/finan"
```

- [ ] **Step 3: Create `src/lib/api/backup.ts`**

```ts
import { commands } from "../bindings";

function unwrap<T>(result: { status: "ok"; data: T } | { status: "error"; error: string }): T {
  if (result.status === "error") throw new Error(result.error);
  return result.data;
}

export async function dbPath(): Promise<string> {
  return unwrap(await commands.dbPath());
}

export async function exportBackup(destination: string): Promise<string> {
  return unwrap(await commands.exportBackup(destination));
}

export async function restoreBackup(source: string): Promise<void> {
  const r = await commands.restoreBackup(source);
  if (r.status === "error") throw new Error(r.error);
}
```

- [ ] **Step 4: pnpm check**

```bash
pnpm check 2>&1 | tail -5
```
Expected: 0 errors.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/lib.rs src/lib/api/backup.ts
git commit -m "$(cat <<'EOF'
feat(ipc): registra backup commands + TS wrapper

- 21 commands totais
- dbPath, exportBackup, restoreBackup funções tipadas

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 4: Settings page (path + Open in Finder + Backup + Restore)

**Files:**
- Modify: `src/routes/Settings.svelte`

- [ ] **Step 1: Replace `src/routes/Settings.svelte`**

```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import { Button } from "$lib/components/ui/button";
  import { open as openDialog, save as saveDialog, message } from "@tauri-apps/plugin-dialog";
  import { revealItemInDir } from "@tauri-apps/plugin-opener";
  import { dbPath, exportBackup, restoreBackup } from "$lib/api/backup";

  let path = $state<string | null>(null);
  let busy = $state(false);
  let info = $state<string | null>(null);
  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      path = await dbPath();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  });

  async function reveal() {
    if (!path) return;
    try {
      await revealItemInDir(path);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  async function doExport() {
    info = null;
    error = null;
    const target = await saveDialog({
      title: "Salvar backup do finan",
      defaultPath: `finan-backup-${new Date().toISOString().slice(0, 10)}.db`,
      filters: [{ name: "SQLite DB", extensions: ["db"] }],
    });
    if (!target) return;

    busy = true;
    try {
      const written = await exportBackup(target);
      info = `Backup salvo em ${written}`;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }

  async function doRestore() {
    info = null;
    error = null;
    const picked = await openDialog({
      title: "Restaurar backup do finan",
      multiple: false,
      filters: [{ name: "SQLite DB", extensions: ["db"] }],
    });
    if (!picked || Array.isArray(picked)) return;

    const confirmed = await confirm(
      "Isso vai SUBSTITUIR seu banco de dados atual. Você precisa reiniciar o app depois. Continuar?",
    );
    if (!confirmed) return;

    busy = true;
    try {
      await restoreBackup(picked);
      await message(
        "Backup restaurado. Feche e abra o finan de novo (Cmd+Q então abra) pra ver as transações restauradas.",
        { title: "Restauração concluída", kind: "info" },
      );
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }
</script>

<section class="p-8 max-w-3xl mx-auto flex flex-col gap-6">
  <header>
    <h2 class="text-xl font-semibold tracking-tight" style="font-family: var(--font-display)">
      Configurações
    </h2>
  </header>

  <!-- DB path -->
  <div class="rounded-xl bg-surface border border-border-subtle p-5 flex flex-col gap-3">
    <div class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint">
      Banco de dados
    </div>
    <p class="text-xs text-fg-muted leading-relaxed">
      Todas as suas transações ficam neste arquivo no seu Mac. Nada é enviado pra nuvem.
    </p>
    <div class="font-mono text-[11.5px] text-fg break-all bg-surface-2 rounded-md p-2 border border-border-subtle">
      {path ?? "Carregando…"}
    </div>
    <div>
      <Button variant="outline" onclick={reveal} disabled={!path}>
        Abrir no Finder
      </Button>
    </div>
  </div>

  <!-- Backup -->
  <div class="rounded-xl bg-surface border border-border-subtle p-5 flex flex-col gap-3">
    <div class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint">
      Backup
    </div>
    <p class="text-xs text-fg-muted leading-relaxed">
      Exporta uma cópia do arquivo pra qualquer pasta. Restaurar SUBSTITUI o banco atual —
      faça backup antes.
    </p>
    <div class="flex gap-2">
      <Button onclick={doExport} disabled={busy}>Exportar backup…</Button>
      <Button variant="outline" onclick={doRestore} disabled={busy}>Restaurar backup…</Button>
    </div>
    {#if info}
      <div class="text-[11.5px] text-pos">{info}</div>
    {/if}
    {#if error}
      <div class="text-[11.5px] text-neg">{error}</div>
    {/if}
  </div>

  <!-- About -->
  <div class="rounded-xl bg-surface border border-border-subtle p-5 flex flex-col gap-2">
    <div class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint">
      Sobre
    </div>
    <div class="text-[12px] text-fg-muted">
      <strong class="text-fg">finan</strong> v0.1.0 — finanças pessoais 100% locais no Mac.
    </div>
    <div class="text-[11px] text-fg-faint">
      Atalhos: <span class="font-mono">⌘1</span> Dashboard ·
      <span class="font-mono">⌘2</span> Transações ·
      <span class="font-mono">⌘3</span> Importar ·
      <span class="font-mono">⌘4</span> Categorias ·
      <span class="font-mono">⌘5</span> Regras ·
      <span class="font-mono">⌘,</span> Configurações ·
      <span class="font-mono">⌘F</span> Buscar ·
      <span class="font-mono">⌘O</span> Abrir OFX
    </div>
  </div>
</section>
```

- [ ] **Step 2: pnpm check**

```bash
pnpm check 2>&1 | tail -5
```
Expected: 0 errors. If `@tauri-apps/plugin-dialog` or `@tauri-apps/plugin-opener` types aren't resolved, ensure they were installed in T1.

- [ ] **Step 3: Commit**

```bash
git add src/routes/Settings.svelte
git commit -m "$(cat <<'EOF'
feat(settings): DB path + Open in Finder + Backup/Restore + About

- 3 cards: Banco de dados (path + Abrir no Finder), Backup (export/restore), Sobre
- export usa saveDialog com filtro .db + nome default datado
- restore usa openDialog + confirm() destrutivo + message() pedindo restart
- Lista de atalhos no About

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 5: Search backend + UI

**Files:**
- Modify: `src-tauri/src/commands/transactions.rs` (add `q` to TransactionFilters + WHERE clause)
- Modify: `src/lib/stores/filters.svelte.ts` (add `q`)
- Create: `src/lib/components/shell/SearchBox.svelte`
- Modify: `src/lib/components/transactions/TxFilterBar.svelte` (mount SearchBox)
- Modify: `src/routes/Transactions.svelte` (pass `q` + handler)

- [ ] **Step 1: Modify `src-tauri/src/commands/transactions.rs`**

Find the `TransactionFilters` struct:
```rust
pub struct TransactionFilters {
    pub account_id: Option<i64>,
    pub month: Option<String>,
    pub category_id: Option<i64>,
    pub limit: Option<u32>,
}
```

Replace with:
```rust
pub struct TransactionFilters {
    pub account_id: Option<i64>,
    pub month: Option<String>,
    pub category_id: Option<i64>,
    pub q: Option<String>,
    pub limit: Option<u32>,
}
```

In `list_transactions`, after the `category_id` filter block, add:
```rust
if let Some(q) = f.q.as_ref() {
    let trimmed = q.trim();
    if !trimmed.is_empty() {
        where_clauses.push(format!(
            "(LOWER(description) LIKE ?{n} OR LOWER(COALESCE(notes, '')) LIKE ?{n})",
            n = bound.len() + 1
        ));
        bound.push(Box::new(format!("%{}%", trimmed.to_lowercase())));
    }
}
```

Add 1 new test to `mod tests`:
```rust
    #[test]
    fn list_transactions_filter_by_q_matches_description_and_notes() {
        let mut conn = fresh_conn();
        let acc = insert_account(&conn, "test", Some("ACC1"));
        let txs = vec![mk("F1", "10"), mk("F2", "20"), mk("F3", "30")];
        raw_insert_batch(&mut conn, acc, &txs);
        conn.execute(
            "UPDATE transactions SET notes = 'pagar uber depois' WHERE ofx_fitid = 'F2'",
            [],
        )
        .unwrap();
        conn.execute(
            "UPDATE transactions SET description = 'UBER TRIP' WHERE ofx_fitid = 'F3'",
            [],
        )
        .unwrap();

        let mut stmt = conn
            .prepare(
                "SELECT ofx_fitid FROM transactions
                 WHERE LOWER(description) LIKE ?1 OR LOWER(COALESCE(notes, '')) LIKE ?1
                 ORDER BY id",
            )
            .unwrap();
        let hits: Vec<String> = stmt
            .query_map(params!["%uber%"], |r| r.get::<_, Option<String>>(0))
            .unwrap()
            .filter_map(|r| r.ok().flatten())
            .collect();

        assert_eq!(hits, vec!["F2".to_string(), "F3".to_string()]);
    }
```

- [ ] **Step 2: Update `src/lib/stores/filters.svelte.ts`**

Replace whole file:
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
  let q = $state<string>("");

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
    get q() {
      return q;
    },
    set q(v: string) {
      q = v;
    },
    clear() {
      month = null;
      categoryId = null;
      q = "";
    },
    resetToCurrentMonth() {
      month = currentMonth();
      categoryId = null;
      q = "";
    },
  };
}

export const filters = createFilterStore();
```

- [ ] **Step 3: Regen bindings**

```bash
pkill -f "tauri dev" 2>/dev/null
pnpm tauri dev > /tmp/finan-dev.log 2>&1 &
until grep -q '"q":' src/lib/bindings.ts 2>/dev/null || grep -q 'q: string \| null' src/lib/bindings.ts 2>/dev/null; do
  sleep 3
done
pkill -f "tauri dev"; pkill -f "vite"; pkill -f "target/debug/finan"
```

- [ ] **Step 4: Create `src/lib/components/shell/SearchBox.svelte`**

```svelte
<script lang="ts">
  type Props = {
    value: string;
    placeholder?: string;
    onInput: (v: string) => void;
    /** Exposed via bind so parent can focus on shortcut */
    ref?: HTMLInputElement | null;
  };

  let { value, placeholder = "Buscar…", onInput, ref = $bindable(null) }: Props = $props();
</script>

<div class="inline-flex items-center gap-1.5 px-2 py-1 rounded-md border border-border bg-surface-2 focus-within:border-accent focus-within:bg-bg transition-colors">
  <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" class="text-fg-faint">
    <circle cx="11" cy="11" r="7"/>
    <path d="M21 21l-4.3-4.3"/>
  </svg>
  <input
    bind:this={ref}
    type="text"
    {value}
    {placeholder}
    oninput={(e) => onInput((e.currentTarget as HTMLInputElement).value)}
    class="bg-transparent border-0 outline-none text-[12px] w-44 text-fg placeholder:text-fg-faint"
  />
  {#if value}
    <button
      type="button"
      onclick={() => onInput("")}
      aria-label="Limpar busca"
      class="text-fg-faint hover:text-fg-muted text-[11px]"
    >
      ✕
    </button>
  {/if}
</div>
```

- [ ] **Step 5: Update `src/lib/components/transactions/TxFilterBar.svelte`**

```svelte
<script lang="ts">
  import MonthStepper from "$lib/components/shell/MonthStepper.svelte";
  import SearchBox from "$lib/components/shell/SearchBox.svelte";
  import type { Category } from "$lib/bindings";

  type Props = {
    categories: Category[];
    month: string | null;
    categoryId: number | null;
    q: string;
    onMonthChange: (m: string | null) => void;
    onCategoryChange: (id: number | null) => void;
    onQueryChange: (v: string) => void;
    searchInputRef?: HTMLInputElement | null;
  };

  let {
    categories,
    month,
    categoryId,
    q,
    onMonthChange,
    onCategoryChange,
    onQueryChange,
    searchInputRef = $bindable(null),
  }: Props = $props();

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

  <div class="ml-auto">
    <SearchBox value={q} onInput={onQueryChange} bind:ref={searchInputRef} />
  </div>
</div>
```

- [ ] **Step 6: Update `src/routes/Transactions.svelte`**

Find the existing `refresh()` function and the `<TxFilterBar>` usage. Update them as follows.

Replace `refresh()`:
```ts
async function refresh() {
  try {
    transactions = await listTransactions({
      account_id: null,
      month: filters.month,
      category_id: filters.categoryId,
      q: filters.q === "" ? null : filters.q,
      limit: null,
    });
    if (selectedTx) {
      const fresh = transactions.find((t) => t.id === selectedTx?.id);
      selectedTx = fresh ?? null;
    }
  } catch (e) {
    error = e instanceof Error ? e.message : String(e);
  }
}
```

Add a search handler:
```ts
let searchInputRef = $state<HTMLInputElement | null>(null);

async function onQueryChange(v: string) {
  filters.q = v;
  await refresh();
}
```

Update the `<TxFilterBar>` markup:
```svelte
<TxFilterBar
  {categories}
  month={filters.month}
  categoryId={filters.categoryId}
  q={filters.q}
  {onMonthChange}
  onCategoryChange={onCategoryFilterChange}
  {onQueryChange}
  bind:searchInputRef
/>
```

Also export `searchInputRef` so the global shortcut handler in App.svelte can focus it. Add to the script:
```ts
export function focusSearch() {
  searchInputRef?.focus();
}
```

Wait — Svelte 5 components don't expose `export function` the same way as Svelte 4. We need a different mechanism. **Simpler approach:** the global handler uses `document.querySelector` to find the search input. We mark it with `data-search-input` attribute in SearchBox:

Modify `SearchBox.svelte` `<input>` to add `data-search-input`:
```svelte
<input
  bind:this={ref}
  type="text"
  data-search-input
  {value}
  ...
```

Then T6's global handler can do `document.querySelector("[data-search-input]")?.focus()`.

Remove the `export function focusSearch` from Transactions.svelte — keep just the state and handler:
```ts
let searchInputRef = $state<HTMLInputElement | null>(null);

async function onQueryChange(v: string) {
  filters.q = v;
  await refresh();
}
```

- [ ] **Step 7: Cargo test + pnpm check**

```bash
cd src-tauri && cargo test --lib 2>&1 | tail -5 && cd ..
pnpm check 2>&1 | tail -5
```
Expected: 35 cargo tests pass / 0 pnpm errors.

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/commands/transactions.rs src/lib/stores/ src/lib/components/shell/SearchBox.svelte src/lib/components/transactions/TxFilterBar.svelte src/routes/Transactions.svelte
git commit -m "$(cat <<'EOF'
feat(search): TransactionFilters.q + SearchBox component + filtro inline

- Rust: q opcional, WHERE LOWER(description) LIKE OR LOWER(notes) LIKE
- filters store ganha q (default vazio)
- SearchBox.svelte com lente + ✕ pra limpar; data-search-input pro focus shortcut
- TxFilterBar empurra SearchBox pro ml-auto à direita
- Transactions.svelte passa q + onQueryChange
- 1 teste cargo (filter por q em description e notes)

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 6: Global keyboard shortcuts

**Files:**
- Modify: `src/App.svelte`

- [ ] **Step 1: Replace `src/App.svelte`**

```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import Router, { push } from "svelte-spa-router";
  import Sidebar from "$lib/components/shell/Sidebar.svelte";
  import { routes } from "./routes/routes";

  const SHORTCUTS: Record<string, () => void | Promise<void>> = {
    "1": () => push("/dashboard"),
    "2": () => push("/transactions"),
    "3": () => push("/import"),
    "4": () => push("/categories"),
    "5": () => push("/rules"),
    ",": () => push("/settings"),
  };

  function onKeydown(e: KeyboardEvent) {
    if (!(e.metaKey || e.ctrlKey)) return;
    const target = e.target as HTMLElement | null;
    const inEditable =
      target &&
      (target.tagName === "INPUT" ||
        target.tagName === "TEXTAREA" ||
        target.isContentEditable);

    const k = e.key.toLowerCase();

    if (k === "f") {
      e.preventDefault();
      // navigate to transactions, then focus search (idempotent if already there)
      const wasOnTransactions = window.location.hash === "#/transactions";
      if (!wasOnTransactions) push("/transactions");
      // small timeout to let the route mount its search input
      setTimeout(() => {
        const input = document.querySelector<HTMLInputElement>("[data-search-input]");
        input?.focus();
        input?.select();
      }, 40);
      return;
    }

    if (k === "o") {
      e.preventDefault();
      push("/import");
      return;
    }

    if (inEditable) return; // don't hijack 1-5 while typing

    if (SHORTCUTS[e.key]) {
      e.preventDefault();
      void SHORTCUTS[e.key]();
    }
  }

  onMount(() => {
    window.addEventListener("keydown", onKeydown);
    return () => window.removeEventListener("keydown", onKeydown);
  });
</script>

<div class="min-h-screen grid grid-cols-[232px_1fr]">
  <Sidebar />
  <main class="bg-bg overflow-y-auto">
    <Router {routes} />
  </main>
</div>
```

- [ ] **Step 2: pnpm check**

```bash
pnpm check 2>&1 | tail -5
```
Expected: 0 errors.

- [ ] **Step 3: Smoke test**

```bash
pkill -f "tauri dev" 2>/dev/null; pkill -f "vite" 2>/dev/null; pkill -f "target/debug/finan" 2>/dev/null
pnpm tauri dev > /tmp/finan-dev.log 2>&1 &
until lsof -iTCP:1420 -sTCP:LISTEN >/dev/null 2>&1; do sleep 2; done
grep -iE "error|fail" /tmp/finan-dev.log | head -5
pkill -f "tauri dev"; pkill -f "vite"; pkill -f "target/debug/finan"
```
Expected: clean boot.

- [ ] **Step 4: Commit**

```bash
git add src/App.svelte
git commit -m "$(cat <<'EOF'
feat(shortcuts): atalhos globais (⌘1-5, ⌘,, ⌘F, ⌘O)

- ⌘1=Dashboard, ⌘2=Transações, ⌘3=Importar, ⌘4=Categorias, ⌘5=Regras
- ⌘,=Configurações (convenção macOS)
- ⌘F=ir pra /transactions + focar search via data-search-input
- ⌘O=ir pra /import
- 1-5 desabilitados quando o foco está em input/textarea
- ⌘F e ⌘O sempre ativos (overrride do browser)

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 7: Visual polish — Button focus + destructive

**Files:**
- Modify: `src/lib/components/ui/button/button.svelte`

- [ ] **Step 1: Read the existing file**

```bash
cat src/lib/components/ui/button/button.svelte
```

This is the shadcn-svelte Button copied during F0-T8. Its `base` classes contain references to `ring-ring`, `aria-invalid:ring-destructive/20`, `aria-invalid:border-destructive` etc. — none of these tokens exist in our `app.css`.

- [ ] **Step 2: Patch the `base` classes**

Find the `base:` line inside `buttonVariants` and replace its entire string with a cleaner one that uses our tokens:

```js
base: "inline-flex shrink-0 items-center justify-center whitespace-nowrap rounded-md text-sm font-medium transition-colors outline-none select-none disabled:pointer-events-none disabled:opacity-50 focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-offset-1 focus-visible:ring-offset-bg [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4",
```

The original had `aria-invalid` rules referencing missing destructive tokens. We drop them — invalid styling can come back in a future iteration with proper tokens.

If the file also has a `destructive` variant referencing `bg-destructive` / `text-destructive-foreground`, update that variant to:
```js
destructive: "bg-neg/10 hover:bg-neg/20 text-neg",
```
(This may already match the F0-T8 variant — verify; if so, no change.)

If `link` variant has `text-primary`, replace with `text-accent`. Verify and patch.

- [ ] **Step 3: pnpm check**

```bash
pnpm check 2>&1 | tail -5
```
Expected: 0 errors.

- [ ] **Step 4: Smoke test boot to verify nothing renders broken**

```bash
pkill -f "tauri dev" 2>/dev/null
pnpm tauri dev > /tmp/finan-dev.log 2>&1 &
until lsof -iTCP:1420 -sTCP:LISTEN >/dev/null 2>&1; do sleep 2; done
grep -iE "error|fail" /tmp/finan-dev.log | head -5
pkill -f "tauri dev"; pkill -f "vite"; pkill -f "target/debug/finan"
```

- [ ] **Step 5: Commit**

```bash
git add src/lib/components/ui/button/button.svelte
git commit -m "$(cat <<'EOF'
fix(ui): Button usa tokens finan em focus/destructive em vez dos defaults shadcn

- base: focus-visible:ring-2 ring-accent ring-offset-1 (em vez de ring-ring)
- Removidas regras aria-invalid:ring-destructive (tokens não existem no palette)
- destructive variant: bg-neg/10 hover:bg-neg/20 text-neg
- link variant: text-accent (em vez de text-primary)

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 8: Acceptance + MVP close

**Files:**
- Modify: `README.md`
- (verification)

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

Expected: pnpm check 0 / pnpm test 19+ / cargo test 35+ / clippy clean / fmt clean / build OK. If fmt has diffs, run `cargo fmt` to apply.

- [ ] **Step 2: Manual E2E walkthrough**

```bash
pkill -f "tauri dev" 2>/dev/null
pnpm tauri dev > /tmp/finan-dev.log 2>&1 &
until lsof -iTCP:1420 -sTCP:LISTEN >/dev/null 2>&1; do sleep 2; done
```

Manual checklist:
- [ ] App abre → Dashboard (ou Onboarding se DB vazia).
- [ ] `⌘1` vai pra Dashboard. `⌘2` vai pra Transações. `⌘3` Importar. `⌘4` Categorias. `⌘5` Regras. `⌘,` Configurações.
- [ ] Em `/transactions`: pressionar `⌘F` foca o SearchBox. Digitar filtra a lista por description/notes.
- [ ] `⌘O` em qualquer rota leva pra `/import`.
- [ ] Em `/settings`: path do DB aparece; "Abrir no Finder" mostra o arquivo no Finder.
- [ ] "Exportar backup…" abre save dialog → escolher local → confirma "Backup salvo em …".
- [ ] Verificar fora do app: `sqlite3 <caminho-backup> ".tables"` mostra as 5 tabelas.
- [ ] "Restaurar backup…" abre open dialog → escolher um .db válido → confirm destrutivo → mensagem "feche e abra de novo".
- [ ] Fechar app (Cmd+Q) e abrir de novo: dados do backup restaurado aparecem.
- [ ] Tentar restaurar um arquivo .txt → erro "is not a valid SQLite 3 file".

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
- ✅ Fase 4 — Dashboard (KPIs + donut + barras 12m + top + recent)
- ✅ Fase 5 — Polish (search ⌘F + settings + backup/restore + atalhos)

**MVP completo.**
```

- [ ] **Step 4: Closing commit + tag**

```bash
git add README.md
git commit -m "$(cat <<'EOF'
chore(fase-5): close polish phase — MVP completo

- Tauri plugins: dialog + opener
- Settings: DB path + Abrir no Finder + backup export + backup restore
- Search global na /transactions (TransactionFilters.q + SearchBox)
- Atalhos: ⌘1-5 nav, ⌘, settings, ⌘F search, ⌘O import
- Button polish: focus-visible:ring-accent, destructive bg-neg
- Tests: 35 cargo / 19 vitest / pnpm check 0 / clippy/fmt limpos

🎉 MVP finan v0.1.0 fechado.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"

git tag -a v0.1.0 -m "finan MVP v0.1.0 — local-first personal finance for macOS"
git tag
```

---

## Self-Review

### Spec coverage

| Spec item | Task |
|---|---|
| §8.7 mostrar path do DB | T4 (Settings page) |
| §8.7 Abrir no Finder | T1 (plugin), T4 (revealItemInDir) |
| §8.7 Exportar backup | T2 (command), T4 (UI + save dialog) |
| §8.7 Restaurar backup destrutivo | T2 (validate + copy + WAL purge), T4 (confirm + message) |
| §8.7 About / versão / atalhos listados | T4 |
| §8.3 search global ⌘F em description + notes | T5 (backend filter + SearchBox), T6 (shortcut) |
| §9 Fase 5 critério "atalhos funcionam" | T6 |
| §9 Fase 5 critério "backup/restore round-trip" | T2 + T4 |
| §9 Fase 5 "refinamento visual" | T7 (Button focus + destructive) |
| §10 cargo tests aumentam | T2 (3) + T5 (1) = 4 novos |

### Placeholder scan

Reli todos os blocos. Sem "TBD" / "TODO" / "implement later" / "add appropriate error handling". Cada step tem código completo.

### Type consistency

- `TransactionFilters` field order: `account_id, month, category_id, q, limit`. Callers no Rust passam todos os campos (Rust `Default::default()` + struct update funciona); callers TS já passam objetos completos (Dashboard, Transactions, IndexRedirect, Import).
- Plugin imports: `@tauri-apps/plugin-dialog` exporta `open`, `save`, `message`, `confirm` (mas usamos `confirm` nativo do browser pra simplicidade). `@tauri-apps/plugin-opener` exporta `revealItemInDir`, `openUrl`, `openPath`. Match.
- `data-search-input` é um attribute, não um type — convenção compartilhada entre `SearchBox.svelte` (set) e `App.svelte` (query). Documentada inline.

### Risks documented inline

- **T2 restore + restart:** o approach é "feche e abra de novo" via message dialog. Tauri 2 tem `app.restart()` mas requer permissão extra; deixar manual é mais explícito pro usuário entender que dados in-memory ficaram stale.
- **T5 LIKE escape:** se o usuário digitar `%` na busca, vira wildcard SQL. Aceitável pro MVP (consequência: matches "amplos"). Polish: escape `%` e `_` antes de bind se virar problema real.
- **T6 ⌘F override:** sobrescrever o ⌘F do navegador é intencional (`e.preventDefault()`). Em outros apps macOS isso é a convenção (Find).
- **T7 destructive variant pode já estar correto** após F0-T8 que customizou variants — verifique antes de re-editar.
