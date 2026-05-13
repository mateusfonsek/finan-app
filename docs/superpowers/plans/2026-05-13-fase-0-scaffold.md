# Fase 0 — Scaffold Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Levantar o esqueleto completo do app finan: Tauri 2 + Svelte 5 + Vite + Tailwind 4 + shadcn-svelte no frontend; rusqlite com migration `0001_init` + um command tipado (`health_check`) no backend. Ao fim desta fase, `pnpm tauri dev` abre janela com sidebar funcional, navegação entre 6 rotas vazias, e SQLite criado no disco com schema aplicado.

**Architecture:** Frontend Svelte 5 puro (sem SvelteKit) compilado pelo Vite, roteamento hash-based via `svelte-spa-router`. Backend Rust isolado em `src-tauri/`, com módulos `db/`, `commands/`, `domain/`, `error.rs`. Tipos compartilhados gerados por `tauri-specta` em build time. SQLite acessado via `rusqlite` com feature `bundled` (compila a lib dentro do binário). Caminho do DB resolvido via `tauri::path::app_data_dir()`.

**Tech Stack:**
- Tauri 2.x · Svelte 5.x · Vite 5.x · TypeScript 5.x
- Tailwind 4 (`@tailwindcss/vite`) · shadcn-svelte · svelte-spa-router
- rusqlite 0.32 (`bundled`) · rust_decimal 1.36 · serde 1 · thiserror 2
- tauri-specta 2 · specta-typescript 0.0.7
- Vitest (TS tests) · `cargo test` (Rust tests com SQLite `:memory:`)

**Acceptance criteria desta fase:**
1. `pnpm tauri dev` abre uma janela macOS com sidebar funcional.
2. Sidebar navega entre 6 rotas vazias (Onboarding, Dashboard, Transactions, Import, Categories, Settings).
3. SQLite criado em `~/Library/Application Support/<bundleId>/finan.db` com tabelas `accounts`, `categories`, `transactions` + seeds de categorias.
4. Command Rust `health_check` retorna struct tipado consumido pelo frontend (mostra version + db_path no Dashboard).
5. `cargo test` passa (migration + health_check). `pnpm test` passa (sample). `pnpm svelte-check` zero erros.
6. Bindings TS gerados em `src/lib/bindings.ts` (gitignored).

**Out of scope (próximos planos):**
- Fase 1: Import OFX (parser + persistência).
- Fase 2: Categorização manual + filtros.
- Fase 3: Rules engine.
- Fase 4: Dashboard com KPIs/charts.
- Fase 5: Polish (search, settings, atalhos).

---

## Estrutura de arquivos a criar

```
finan-app/
├── package.json                        T1
├── pnpm-workspace.yaml                 T1 (vazio mas evita warning)
├── tsconfig.json                       T1
├── tsconfig.node.json                  T1
├── vite.config.ts                      T1
├── svelte.config.js                    T1
├── index.html                          T1
├── src/
│   ├── main.ts                         T1
│   ├── App.svelte                      T1 → reescrito em T4
│   ├── app.css                         T2 (Tailwind + @theme tokens)
│   ├── vite-env.d.ts                   T1
│   ├── lib/
│   │   ├── api/health.ts               T7
│   │   ├── components/
│   │   │   ├── shell/Sidebar.svelte    T4
│   │   │   └── ui/button.svelte        T8 (shadcn)
│   │   └── utils/format-money.ts       T9
│   │   └── utils/format-money.test.ts  T9
│   ├── routes/
│   │   ├── routes.ts                   T3
│   │   ├── Onboarding.svelte           T3
│   │   ├── Dashboard.svelte            T3 → atualizado em T7
│   │   ├── Transactions.svelte         T3
│   │   ├── Import.svelte               T3
│   │   ├── Categories.svelte           T3
│   │   └── Settings.svelte             T3
│   └── bindings.ts                     T6 (generated, gitignored)
└── src-tauri/
    ├── Cargo.toml                      T1 → atualizado em T5, T6
    ├── build.rs                        T1
    ├── tauri.conf.json                 T1
    ├── capabilities/default.json       T1
    ├── icons/                          T1 (placeholder do template)
    ├── migrations/
    │   └── 0001_init.sql               T5
    └── src/
        ├── main.rs                     T1
        ├── lib.rs                      T1 → reescrito em T6
        ├── error.rs                    T5
        ├── db/
        │   ├── mod.rs                  T5
        │   └── migrations.rs           T5
        └── commands/
            ├── mod.rs                  T6
            └── health.rs               T6
```

---

## Task 1: Bootstrap Tauri 2 + Svelte 5 + Vite manualmente

**Files:**
- Create: `package.json`
- Create: `pnpm-workspace.yaml`
- Create: `tsconfig.json`, `tsconfig.node.json`
- Create: `vite.config.ts`
- Create: `svelte.config.js`
- Create: `index.html`
- Create: `src/main.ts`, `src/App.svelte`, `src/vite-env.d.ts`
- Create: `src-tauri/Cargo.toml`, `src-tauri/build.rs`
- Create: `src-tauri/src/main.rs`, `src-tauri/src/lib.rs`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/capabilities/default.json`
- Create: `src-tauri/icons/icon.png` (placeholder)

**Por que manual em vez de `pnpm create tauri-app`?** O wizard cria projeto em diretório vazio. Já temos `docs/`, `examples/`, `README.md`, `.git/` — o wizard não suporta isso. Manual é mais previsível.

- [ ] **Step 1: Verificar Node + pnpm + Rust**

Run:
```bash
node --version && pnpm --version && rustc --version && cargo --version
```

Expected: Node 20+, pnpm 9+, Rust 1.77+, cargo presente. Se faltar algo, instalar antes de prosseguir.

- [ ] **Step 2: Criar `package.json`**

```json
{
  "name": "finan",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview",
    "check": "svelte-check --tsconfig ./tsconfig.json",
    "tauri": "tauri"
  },
  "devDependencies": {
    "@sveltejs/vite-plugin-svelte": "^4.0.0",
    "@tauri-apps/cli": "^2.0.0",
    "svelte": "^5.0.0",
    "svelte-check": "^4.0.0",
    "tslib": "^2.6.0",
    "typescript": "^5.5.0",
    "vite": "^5.4.0"
  },
  "dependencies": {
    "@tauri-apps/api": "^2.0.0"
  }
}
```

- [ ] **Step 3: Criar `tsconfig.json`**

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "useDefineForClassFields": true,
    "module": "ESNext",
    "resolveJsonModule": true,
    "allowJs": true,
    "checkJs": true,
    "isolatedModules": true,
    "moduleResolution": "bundler",
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "skipLibCheck": true,
    "sourceMap": true,
    "esModuleInterop": true,
    "forceConsistentCasingInFileNames": true,
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noImplicitReturns": true,
    "noFallthroughCasesInSwitch": true,
    "allowSyntheticDefaultImports": true,
    "paths": {
      "$lib": ["./src/lib"],
      "$lib/*": ["./src/lib/*"]
    }
  },
  "include": ["src/**/*.ts", "src/**/*.svelte", "src/**/*.d.ts"],
  "references": [{ "path": "./tsconfig.node.json" }]
}
```

- [ ] **Step 4: Criar `tsconfig.node.json`**

```json
{
  "compilerOptions": {
    "composite": true,
    "skipLibCheck": true,
    "module": "ESNext",
    "moduleResolution": "bundler",
    "allowSyntheticDefaultImports": true,
    "strict": true
  },
  "include": ["vite.config.ts"]
}
```

- [ ] **Step 5: Criar `vite.config.ts`**

```ts
import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { fileURLToPath, URL } from "node:url";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [svelte()],
  resolve: {
    alias: {
      $lib: fileURLToPath(new URL("./src/lib", import.meta.url)),
    },
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? { protocol: "ws", host, port: 1421 }
      : undefined,
    watch: { ignored: ["**/src-tauri/**"] },
  },
});
```

- [ ] **Step 6: Criar `svelte.config.js`**

```js
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

export default {
  preprocess: vitePreprocess(),
  compilerOptions: {
    runes: true,
  },
};
```

- [ ] **Step 7: Criar `index.html`**

```html
<!doctype html>
<html lang="pt-BR">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>finan</title>
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.ts"></script>
  </body>
</html>
```

- [ ] **Step 8: Criar `src/main.ts`**

```ts
import { mount } from "svelte";
import App from "./App.svelte";

const app = mount(App, { target: document.getElementById("app")! });

export default app;
```

- [ ] **Step 9: Criar `src/App.svelte` (placeholder mínimo)**

```svelte
<script lang="ts">
  let message = $state("finan — scaffold OK");
</script>

<main>
  <h1>{message}</h1>
</main>
```

- [ ] **Step 10: Criar `src/vite-env.d.ts`**

```ts
/// <reference types="svelte" />
/// <reference types="vite/client" />
```

- [ ] **Step 11: Criar `pnpm-workspace.yaml` (vazio, só pra silenciar warning de Tauri)**

```yaml
packages: []
```

- [ ] **Step 12: Criar `src-tauri/Cargo.toml`**

```toml
[package]
name = "finan"
version = "0.1.0"
description = "finan — app local de finanças pessoais"
edition = "2021"
rust-version = "1.77"

[lib]
name = "finan_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = [] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[profile.release]
panic = "abort"
codegen-units = 1
lto = true
opt-level = "s"
strip = true
```

- [ ] **Step 13: Criar `src-tauri/build.rs`**

```rust
fn main() {
    tauri_build::build()
}
```

- [ ] **Step 14: Criar `src-tauri/src/main.rs`**

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    finan_lib::run()
}
```

- [ ] **Step 15: Criar `src-tauri/src/lib.rs`**

```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|_app| Ok(()))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 16: Criar `src-tauri/tauri.conf.json`**

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "finan",
  "version": "0.1.0",
  "identifier": "app.finan",
  "build": {
    "beforeDevCommand": "pnpm dev",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "pnpm build",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "title": "finan",
        "width": 1180,
        "height": 760,
        "minWidth": 880,
        "minHeight": 600,
        "resizable": true,
        "fullscreen": false,
        "decorations": true,
        "titleBarStyle": "Visible"
      }
    ],
    "security": {
      "csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self' ipc: http://ipc.localhost"
    }
  },
  "bundle": {
    "active": true,
    "targets": ["app", "dmg"],
    "icon": ["icons/icon.png"],
    "macOS": {
      "minimumSystemVersion": "12.0"
    }
  }
}
```

- [ ] **Step 17: Criar `src-tauri/capabilities/default.json`**

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Default capabilities for finan",
  "windows": ["main"],
  "permissions": [
    "core:default"
  ]
}
```

- [ ] **Step 18: Criar `src-tauri/icons/icon.png` (placeholder)**

Run:
```bash
mkdir -p src-tauri/icons
# Gera um PNG 512x512 sólido como placeholder. Substituir antes de release.
python3 -c "
import struct, zlib
def png_solid(w, h, rgba):
    sig = b'\x89PNG\r\n\x1a\n'
    def chunk(t, d):
        return struct.pack('>I', len(d)) + t + d + struct.pack('>I', zlib.crc32(t + d) & 0xffffffff)
    ihdr = struct.pack('>IIBBBBB', w, h, 8, 6, 0, 0, 0)
    raw = b''.join(b'\\x00' + bytes(rgba) * w for _ in range(h))
    idat = zlib.compress(raw, 9)
    return sig + chunk(b'IHDR', ihdr) + chunk(b'IDAT', idat) + chunk(b'IEND', b'')
open('src-tauri/icons/icon.png','wb').write(png_solid(512,512,(34,87,46,255)))
"
ls -la src-tauri/icons/
```

Expected: arquivo `icon.png` de ~1-2KB criado.

- [ ] **Step 19: Instalar dependências TS**

Run:
```bash
pnpm install
```

Expected: `node_modules/` criado, sem erros.

- [ ] **Step 20: Build Rust pela primeira vez (download deps)**

Run:
```bash
cd src-tauri && cargo build && cd ..
```

Expected: compila com sucesso (pode levar 3-5 min na primeira vez). Sem erros.

- [ ] **Step 21: Smoke test — `pnpm tauri dev` abre janela**

Run:
```bash
pnpm tauri dev
```

Expected: janela macOS abre com título "finan", mostra "finan — scaffold OK". Fechar com Cmd+Q.

- [ ] **Step 22: Commit**

```bash
git add .
git commit -m "$(cat <<'EOF'
feat(scaffold): Tauri 2 + Svelte 5 + Vite skeleton

- package.json, tsconfig, vite.config, svelte.config
- App.svelte placeholder, main.ts entry
- src-tauri com Cargo.toml, lib.rs, main.rs, tauri.conf.json
- icon placeholder, capability default
- pnpm tauri dev abre janela com mensagem de scaffold OK

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 2: Tailwind 4 + tokens do design system

**Files:**
- Modify: `package.json` (add deps)
- Modify: `vite.config.ts` (add tailwindcss plugin)
- Create: `src/app.css`
- Modify: `src/main.ts` (import app.css)
- Modify: `src/App.svelte` (use Tailwind class to verify)

- [ ] **Step 1: Instalar Tailwind 4**

Run:
```bash
pnpm add -D tailwindcss @tailwindcss/vite
```

- [ ] **Step 2: Atualizar `vite.config.ts` pra incluir tailwind plugin**

Edit `vite.config.ts`:

```ts
import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";
import { fileURLToPath, URL } from "node:url";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [svelte(), tailwindcss()],
  resolve: {
    alias: {
      $lib: fileURLToPath(new URL("./src/lib", import.meta.url)),
    },
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? { protocol: "ws", host, port: 1421 }
      : undefined,
    watch: { ignored: ["**/src-tauri/**"] },
  },
});
```

- [ ] **Step 3: Criar `src/app.css` com Tailwind + tokens do palette**

```css
@import "tailwindcss";

@theme inline {
  /* surfaces */
  --color-bg: oklch(19% 0.005 240);
  --color-bg-translucent: oklch(19% 0.005 240 / 0.72);
  --color-surface: oklch(22% 0.006 240);
  --color-surface-2: oklch(25% 0.006 240);
  --color-surface-3: oklch(28% 0.007 240);
  --color-hover: oklch(30% 0.008 240);

  /* borders */
  --color-border: oklch(32% 0.009 240);
  --color-border-subtle: oklch(27% 0.008 240);

  /* text */
  --color-fg: oklch(96% 0.005 240);
  --color-fg-muted: oklch(72% 0.008 240);
  --color-fg-subtle: oklch(55% 0.01 240);
  --color-fg-faint: oklch(42% 0.01 240);

  /* accent (moss green) */
  --color-accent: oklch(66% 0.115 145);
  --color-accent-hi: oklch(74% 0.13 145);
  --color-accent-on: oklch(15% 0.04 145);
  --color-accent-soft: oklch(66% 0.115 145 / 0.16);
  --color-accent-ring: oklch(66% 0.115 145 / 0.32);

  /* categorical (chips, charts) */
  --color-cat-mercado: oklch(66% 0.115 145);
  --color-cat-transporte: oklch(64% 0.10 230);
  --color-cat-restaurante: oklch(70% 0.12 65);
  --color-cat-casa: oklch(64% 0.10 295);
  --color-cat-saude: oklch(66% 0.10 0);
  --color-cat-lazer: oklch(68% 0.10 195);
  --color-cat-assinatura: oklch(60% 0.08 320);
  --color-cat-renda: oklch(70% 0.13 145);
  --color-cat-outros: oklch(60% 0.005 240);

  /* semantic */
  --color-pos: oklch(70% 0.13 145);
  --color-neg: oklch(68% 0.13 25);

  /* radii */
  --radius-sm: 4px;
  --radius-md: 6px;
  --radius-lg: 8px;
  --radius-xl: 12px;
  --radius-2xl: 16px;

  /* fonts */
  --font-display: -apple-system, BlinkMacSystemFont, "SF Pro Display", "Inter", system-ui, sans-serif;
  --font-body: -apple-system, BlinkMacSystemFont, "SF Pro Text", "Inter", system-ui, sans-serif;
  --font-mono: ui-monospace, "SF Mono", "JetBrains Mono", Menlo, monospace;
}

html {
  color-scheme: dark;
}

body {
  margin: 0;
  background: var(--color-bg);
  color: var(--color-fg);
  font-family: var(--font-body);
  font-size: 13px;
  line-height: 1.5;
  letter-spacing: -0.01em;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  font-feature-settings: "calt", "ss01", "cv11";
}

::selection {
  background: var(--color-accent-soft);
  color: var(--color-fg);
}

.tabular {
  font-variant-numeric: tabular-nums;
}
```

- [ ] **Step 4: Importar `app.css` em `src/main.ts`**

Edit `src/main.ts`:

```ts
import "./app.css";
import { mount } from "svelte";
import App from "./App.svelte";

const app = mount(App, { target: document.getElementById("app")! });

export default app;
```

- [ ] **Step 5: Verificar Tailwind funciona em `App.svelte`**

Edit `src/App.svelte`:

```svelte
<script lang="ts">
  let message = $state("finan — Tailwind OK");
</script>

<main class="p-8">
  <h1 class="text-2xl font-semibold text-accent">{message}</h1>
  <p class="mt-2 text-fg-muted">Se você está vendo verde-musgo, os tokens carregaram.</p>
</main>
```

- [ ] **Step 6: Rodar dev e verificar visualmente**

Run:
```bash
pnpm tauri dev
```

Expected: janela abre. Título em verde-musgo (`text-accent`). Fundo escuro. Texto cinza claro abaixo. Fechar com Cmd+Q.

- [ ] **Step 7: Type-check**

Run:
```bash
pnpm check
```

Expected: zero erros.

- [ ] **Step 8: Commit**

```bash
git add .
git commit -m "$(cat <<'EOF'
feat(ui): Tailwind 4 + design tokens do palette finan

- @tailwindcss/vite plugin
- src/app.css com @theme inline (surfaces, fg, accent, categorical, semantic, radii, fonts)
- dark color-scheme + body defaults
- helper .tabular pra números monetários

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: Roteamento com svelte-spa-router

**Files:**
- Modify: `package.json` (add svelte-spa-router)
- Create: `src/routes/routes.ts`
- Create: `src/routes/{Onboarding,Dashboard,Transactions,Import,Categories,Settings}.svelte`
- Modify: `src/App.svelte` (mount Router)

- [ ] **Step 1: Instalar svelte-spa-router**

Run:
```bash
pnpm add svelte-spa-router
```

- [ ] **Step 2: Criar `src/routes/Onboarding.svelte`**

```svelte
<script lang="ts"></script>

<section class="p-8">
  <h2 class="text-xl font-semibold">Onboarding</h2>
  <p class="text-fg-muted mt-1">Tela inicial — primeiro uso.</p>
</section>
```

- [ ] **Step 3: Criar `src/routes/Dashboard.svelte`**

```svelte
<script lang="ts"></script>

<section class="p-8">
  <h2 class="text-xl font-semibold">Dashboard</h2>
  <p class="text-fg-muted mt-1">KPIs, donut, barras, recent.</p>
</section>
```

- [ ] **Step 4: Criar `src/routes/Transactions.svelte`**

```svelte
<script lang="ts"></script>

<section class="p-8">
  <h2 class="text-xl font-semibold">Transações</h2>
  <p class="text-fg-muted mt-1">Tabela com filtros e categorização inline.</p>
</section>
```

- [ ] **Step 5: Criar `src/routes/Import.svelte`**

```svelte
<script lang="ts"></script>

<section class="p-8">
  <h2 class="text-xl font-semibold">Importar OFX</h2>
  <p class="text-fg-muted mt-1">Drag-and-drop do extrato.</p>
</section>
```

- [ ] **Step 6: Criar `src/routes/Categories.svelte`**

```svelte
<script lang="ts"></script>

<section class="p-8">
  <h2 class="text-xl font-semibold">Categorias</h2>
  <p class="text-fg-muted mt-1">Gerenciar categorias de despesa/renda.</p>
</section>
```

- [ ] **Step 7: Criar `src/routes/Settings.svelte`**

```svelte
<script lang="ts"></script>

<section class="p-8">
  <h2 class="text-xl font-semibold">Configurações</h2>
  <p class="text-fg-muted mt-1">Path do DB, backup, atalhos.</p>
</section>
```

- [ ] **Step 8: Criar `src/routes/routes.ts`**

```ts
import Onboarding from "./Onboarding.svelte";
import Dashboard from "./Dashboard.svelte";
import Transactions from "./Transactions.svelte";
import Import from "./Import.svelte";
import Categories from "./Categories.svelte";
import Settings from "./Settings.svelte";

export const routes = {
  "/": Dashboard,
  "/onboarding": Onboarding,
  "/dashboard": Dashboard,
  "/transactions": Transactions,
  "/import": Import,
  "/categories": Categories,
  "/settings": Settings,
};
```

- [ ] **Step 9: Reescrever `src/App.svelte` com `<Router/>`**

```svelte
<script lang="ts">
  import Router, { link } from "svelte-spa-router";
  import { routes } from "./routes/routes";
</script>

<div class="min-h-screen grid grid-cols-[232px_1fr]">
  <aside class="bg-surface border-r border-border-subtle p-3">
    <nav class="flex flex-col gap-1">
      <a use:link href="/dashboard" class="px-2 py-1 rounded-md hover:bg-hover">Dashboard</a>
      <a use:link href="/transactions" class="px-2 py-1 rounded-md hover:bg-hover">Transações</a>
      <a use:link href="/import" class="px-2 py-1 rounded-md hover:bg-hover">Importar OFX</a>
      <a use:link href="/categories" class="px-2 py-1 rounded-md hover:bg-hover">Categorias</a>
      <a use:link href="/onboarding" class="px-2 py-1 rounded-md hover:bg-hover">Onboarding</a>
      <a use:link href="/settings" class="px-2 py-1 rounded-md hover:bg-hover">Configurações</a>
    </nav>
  </aside>
  <main class="bg-bg">
    <Router {routes} />
  </main>
</div>
```

> **Nota:** A sidebar nesta task é provisória (links texto puros). O componente `Sidebar.svelte` polido entra em Task 4.

- [ ] **Step 10: Type-check**

Run:
```bash
pnpm check
```

Expected: zero erros.

- [ ] **Step 11: Smoke test — navegar entre rotas**

Run:
```bash
pnpm tauri dev
```

Manual: clicar em cada link da sidebar. URL hash muda (`#/dashboard`, `#/transactions`, ...). Conteúdo da `<main>` troca conforme a rota.

Expected: 6 rotas navegáveis. Fechar com Cmd+Q.

- [ ] **Step 12: Commit**

```bash
git add .
git commit -m "$(cat <<'EOF'
feat(routing): svelte-spa-router + 6 rotas vazias

- routes/routes.ts mapeia paths -> componentes
- Onboarding, Dashboard, Transactions, Import, Categories, Settings (stubs)
- App.svelte com layout sidebar+main e <Router/>

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 4: Sidebar component (polido)

**Files:**
- Create: `src/lib/components/shell/Sidebar.svelte`
- Modify: `src/App.svelte` (usar Sidebar.svelte)

- [ ] **Step 1: Criar `src/lib/components/shell/Sidebar.svelte`**

```svelte
<script lang="ts">
  import { link, location } from "svelte-spa-router";

  type NavItem = { href: string; label: string; section?: string };

  const navItems: NavItem[] = [
    { section: "Visão geral", href: "/dashboard", label: "Dashboard" },
    { section: "Visão geral", href: "/transactions", label: "Transações" },
    { section: "Importar", href: "/import", label: "Importar OFX" },
    { section: "Organizar", href: "/categories", label: "Categorias" },
  ];

  const sections = ["Visão geral", "Importar", "Organizar"];

  function isActive(href: string, current: string): boolean {
    if (href === "/dashboard" && (current === "/" || current === "/dashboard")) return true;
    return current === href;
  }
</script>

<aside class="bg-surface border-r border-border-subtle flex flex-col py-3 px-2.5 select-none">
  <div class="flex items-center gap-2 px-2 pb-3.5">
    <div class="w-[22px] h-[22px] rounded-md grid place-items-center"
         style="background: linear-gradient(180deg, var(--color-accent-hi), var(--color-accent));">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="w-3 h-3" style="color: var(--color-accent-on);">
        <path d="M11 20A7 7 0 0 1 4 13c0-7 7-9 16-9 0 9-2 16-9 16Z"/>
        <path d="M4 13c4-1 9-1 13-5"/>
      </svg>
    </div>
    <div>
      <div class="text-[13.5px] font-semibold tracking-tight" style="font-family: var(--font-display)">finan</div>
      <div class="text-[10px] text-fg-faint mt-px">100% local</div>
    </div>
  </div>

  {#each sections as section}
    <div class="mt-2.5 flex flex-col gap-px">
      <div class="text-[10.5px] font-semibold uppercase tracking-wider text-fg-faint px-2 pt-2 pb-1">
        {section}
      </div>
      {#each navItems.filter((i) => i.section === section) as item}
        {@const active = isActive(item.href, $location)}
        <a use:link
           href={item.href}
           class="flex items-center gap-2 px-2 py-1.5 rounded-md text-[12.5px] font-medium transition-colors {active ? 'bg-accent-soft text-fg' : 'text-fg-muted hover:bg-hover hover:text-fg'}">
          {item.label}
        </a>
      {/each}
    </div>
  {/each}

  <div class="flex-1"></div>

  <a use:link href="/settings"
     class="flex items-center gap-2 px-2 py-1.5 rounded-md text-[12.5px] font-medium text-fg-muted hover:bg-hover hover:text-fg transition-colors">
    Configurações
  </a>
</aside>
```

- [ ] **Step 2: Atualizar `src/App.svelte` pra usar Sidebar**

```svelte
<script lang="ts">
  import Router from "svelte-spa-router";
  import Sidebar from "$lib/components/shell/Sidebar.svelte";
  import { routes } from "./routes/routes";
</script>

<div class="min-h-screen grid grid-cols-[232px_1fr]">
  <Sidebar />
  <main class="bg-bg overflow-y-auto">
    <Router {routes} />
  </main>
</div>
```

- [ ] **Step 3: Type-check**

Run:
```bash
pnpm check
```

Expected: zero erros.

- [ ] **Step 4: Verificar visualmente**

Run:
```bash
pnpm tauri dev
```

Expected: sidebar com brand mark verde, seções "Visão geral / Importar / Organizar", item ativo destacado em fundo verde-suave, hover muda fundo. "Configurações" no rodapé. Fechar com Cmd+Q.

- [ ] **Step 5: Commit**

```bash
git add .
git commit -m "$(cat <<'EOF'
feat(ui): Sidebar polido com brand mark, seções e estado ativo

- Sidebar.svelte com nav agrupada (Visão geral, Importar, Organizar)
- active state via $location do svelte-spa-router
- brand mark com gradient moss green
- Configurações sticky no rodapé

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 5: Rust deps + DB connection + migration 0001

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Create: `src-tauri/migrations/0001_init.sql`
- Create: `src-tauri/src/error.rs`
- Create: `src-tauri/src/db/mod.rs`
- Create: `src-tauri/src/db/migrations.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Adicionar dependências ao `Cargo.toml`**

Edit `src-tauri/Cargo.toml`, replace `[dependencies]` section:

```toml
[dependencies]
tauri = { version = "2", features = [] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.32", features = ["bundled"] }
rust_decimal = { version = "1.36", features = ["serde-with-str"] }
thiserror = "2"
chrono = { version = "0.4", features = ["serde"] }
```

- [ ] **Step 2: Criar `src-tauri/migrations/0001_init.sql`**

```sql
CREATE TABLE accounts (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  bank TEXT,
  ofx_acctid TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE categories (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  color_token TEXT,
  kind TEXT NOT NULL CHECK(kind IN ('expense','income','transfer')),
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE transactions (
  id INTEGER PRIMARY KEY,
  account_id INTEGER NOT NULL REFERENCES accounts(id),
  date TEXT NOT NULL,
  amount TEXT NOT NULL,
  description TEXT NOT NULL,
  category_id INTEGER REFERENCES categories(id),
  notes TEXT,
  ofx_fitid TEXT,
  imported_at TEXT NOT NULL DEFAULT (datetime('now')),
  UNIQUE(account_id, ofx_fitid)
);

CREATE INDEX idx_tx_date ON transactions(date);
CREATE INDEX idx_tx_category ON transactions(category_id);
CREATE INDEX idx_tx_account ON transactions(account_id);

INSERT INTO categories (name, color_token, kind) VALUES
  ('Mercado',     '--color-cat-mercado',     'expense'),
  ('Restaurante', '--color-cat-restaurante', 'expense'),
  ('Transporte',  '--color-cat-transporte',  'expense'),
  ('Casa',        '--color-cat-casa',        'expense'),
  ('Saúde',       '--color-cat-saude',       'expense'),
  ('Lazer',       '--color-cat-lazer',       'expense'),
  ('Assinatura',  '--color-cat-assinatura',  'expense'),
  ('Renda',       '--color-cat-renda',       'income'),
  ('Outros',      '--color-cat-outros',      'expense');
```

- [ ] **Step 3: Criar `src-tauri/src/error.rs`**

```rust
use serde::{Serialize, Serializer};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("path resolution failed: {0}")]
    Path(String),

    #[error("invalid data: {0}")]
    Invalid(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

pub type AppResult<T> = Result<T, AppError>;
```

- [ ] **Step 4: Criar `src-tauri/src/db/migrations.rs`**

```rust
use rusqlite::Connection;

use crate::error::AppResult;

const MIGRATIONS: &[(&str, &str)] = &[
    ("0001_init", include_str!("../../migrations/0001_init.sql")),
];

pub fn apply(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _migrations (
            name TEXT PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );",
    )?;

    for (name, sql) in MIGRATIONS {
        let already: bool = conn
            .query_row(
                "SELECT 1 FROM _migrations WHERE name = ?1",
                [name],
                |_row| Ok(true),
            )
            .unwrap_or(false);

        if !already {
            conn.execute_batch(sql)?;
            conn.execute("INSERT INTO _migrations (name) VALUES (?1)", [name])?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn table_exists(conn: &Connection, name: &str) -> bool {
        conn.query_row(
            "SELECT name FROM sqlite_master WHERE type='table' AND name=?1",
            [name],
            |row| row.get::<_, String>(0),
        )
        .is_ok()
    }

    #[test]
    fn applies_init_migration() {
        let conn = Connection::open_in_memory().unwrap();
        apply(&conn).unwrap();

        assert!(table_exists(&conn, "accounts"));
        assert!(table_exists(&conn, "categories"));
        assert!(table_exists(&conn, "transactions"));
        assert!(table_exists(&conn, "_migrations"));
    }

    #[test]
    fn seeds_default_categories() {
        let conn = Connection::open_in_memory().unwrap();
        apply(&conn).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM categories", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 9);

        let renda_kind: String = conn
            .query_row(
                "SELECT kind FROM categories WHERE name='Renda'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(renda_kind, "income");
    }

    #[test]
    fn migration_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        apply(&conn).unwrap();
        apply(&conn).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM categories", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 9, "re-running migrations should not duplicate seeds");
    }
}
```

- [ ] **Step 5: Criar `src-tauri/src/db/mod.rs`**

```rust
use std::path::PathBuf;
use std::sync::Mutex;

use rusqlite::Connection;
use tauri::{AppHandle, Manager};

use crate::error::{AppError, AppResult};

pub mod migrations;

pub struct Db {
    pub conn: Mutex<Connection>,
    pub path: PathBuf,
}

pub fn init(app: &AppHandle) -> AppResult<Db> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Path(e.to_string()))?;

    std::fs::create_dir_all(&dir)?;
    let path = dir.join("finan.db");

    let conn = Connection::open(&path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;

    migrations::apply(&conn)?;

    Ok(Db {
        conn: Mutex::new(conn),
        path,
    })
}
```

- [ ] **Step 6: Atualizar `src-tauri/src/lib.rs`**

```rust
mod db;
mod error;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let database = db::init(app.handle())
                .expect("failed to initialize database");
            eprintln!("[finan] db at {}", database.path.display());
            app.manage(database);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 7: Rodar testes Rust**

Run:
```bash
cd src-tauri && cargo test --lib && cd ..
```

Expected: 3 testes passam (`applies_init_migration`, `seeds_default_categories`, `migration_is_idempotent`).

- [ ] **Step 8: Smoke test — DB criado no disco**

Run:
```bash
# Garante que não existe ainda
rm -f ~/Library/Application\ Support/app.finan/finan.db
pnpm tauri dev
```

Esperar a janela abrir. Em outro terminal:

```bash
ls -la ~/Library/Application\ Support/app.finan/
sqlite3 ~/Library/Application\ Support/app.finan/finan.db ".tables"
sqlite3 ~/Library/Application\ Support/app.finan/finan.db "SELECT name FROM categories;"
```

Expected:
- `finan.db` existe
- `.tables` lista: `_migrations accounts categories transactions`
- categories tem 9 nomes (Mercado, Restaurante, …, Outros)

Fechar app com Cmd+Q.

- [ ] **Step 9: Commit**

```bash
git add .
git commit -m "$(cat <<'EOF'
feat(db): rusqlite + migration 0001 com seeds de categorias

- rusqlite 0.32 (bundled), rust_decimal, thiserror, chrono
- migrations/0001_init.sql: accounts, categories, transactions + indexes
- 9 categorias seed com color_token dos CSS vars
- db/migrations.rs com tracking table _migrations e idempotência
- db/mod.rs abre conexão em ~/Library/Application Support/app.finan/finan.db
- WAL journal mode + foreign_keys ON
- 3 cargo tests passando

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 6: tauri-specta + comando `health_check`

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Create: `src-tauri/src/commands/mod.rs`
- Create: `src-tauri/src/commands/health.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `.gitignore` (já tem `src/lib/bindings.ts` — confirmar)

- [ ] **Step 1: Adicionar tauri-specta ao `Cargo.toml`**

Edit `src-tauri/Cargo.toml`, append to `[dependencies]`:

```toml
specta = "2.0.0-rc.20"
specta-typescript = "0.0.7"
tauri-specta = { version = "2.0.0-rc.20", features = ["derive", "typescript"] }
```

- [ ] **Step 2: Criar `src-tauri/src/commands/mod.rs`**

```rust
pub mod health;
```

- [ ] **Step 3: Criar `src-tauri/src/commands/health.rs`**

```rust
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

use crate::db::Db;
use crate::error::AppResult;

#[derive(Debug, Serialize, Deserialize, Type)]
pub struct HealthInfo {
    pub version: String,
    pub db_path: String,
    pub category_count: u32,
}

#[tauri::command]
#[specta::specta]
pub fn health_check(db: State<'_, Db>) -> AppResult<HealthInfo> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    let category_count: u32 = conn.query_row(
        "SELECT COUNT(*) FROM categories",
        [],
        |row| row.get(0),
    )?;

    Ok(HealthInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        db_path: db.path.display().to_string(),
        category_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;
    use rusqlite::Connection;

    #[test]
    fn health_info_struct_serializes() {
        let info = HealthInfo {
            version: "0.1.0".to_string(),
            db_path: "/tmp/finan.db".to_string(),
            category_count: 9,
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("\"version\":\"0.1.0\""));
        assert!(json.contains("\"category_count\":9"));
    }

    #[test]
    fn category_count_matches_seed() {
        let conn = Connection::open_in_memory().unwrap();
        migrations::apply(&conn).unwrap();
        let count: u32 = conn
            .query_row("SELECT COUNT(*) FROM categories", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 9);
    }
}
```

- [ ] **Step 4: Atualizar `src-tauri/src/lib.rs`**

```rust
mod commands;
mod db;
mod error;

use tauri::Manager;
use tauri_specta::{collect_commands, Builder};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let specta_builder = Builder::<tauri::Wry>::new()
        .commands(collect_commands![commands::health::health_check]);

    #[cfg(debug_assertions)]
    specta_builder
        .export(
            specta_typescript::Typescript::default(),
            "../src/lib/bindings.ts",
        )
        .expect("failed to export TS bindings");

    tauri::Builder::default()
        .invoke_handler(specta_builder.invoke_handler())
        .setup(|app| {
            let database = db::init(app.handle())
                .expect("failed to initialize database");
            eprintln!("[finan] db at {}", database.path.display());
            app.manage(database);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 5: Confirmar que `.gitignore` cobre `src/lib/bindings.ts`**

Run:
```bash
grep "bindings.ts" .gitignore
```

Expected: `src/lib/bindings.ts` aparece. Se não, adicionar.

- [ ] **Step 6: Build Rust + rodar testes**

Run:
```bash
cd src-tauri && cargo build && cargo test --lib && cd ..
```

Expected: build OK + 5 testes passam (3 de migrations + 2 de health).

- [ ] **Step 7: Verificar bindings.ts gerado**

Run:
```bash
pnpm tauri dev &
sleep 25
ls -la src/lib/bindings.ts
head -40 src/lib/bindings.ts
```

Expected: arquivo existe e contém `healthCheck` + tipo `HealthInfo` com campos `version`, `db_path`, `category_count`.

Matar o dev server:
```bash
pkill -f "tauri dev" || true
pkill -f "vite" || true
```

- [ ] **Step 8: Commit**

```bash
git add .
git commit -m "$(cat <<'EOF'
feat(ipc): tauri-specta + comando health_check tipado

- specta, specta-typescript, tauri-specta no Cargo.toml
- commands/health.rs com HealthInfo (version, db_path, category_count)
- specta::Type derivado pra geração automática de TS
- lib.rs exporta bindings em src/lib/bindings.ts (gitignored, debug only)
- 2 testes adicionais (struct serialization + category count)

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 7: Frontend consome `health_check`

**Files:**
- Create: `src/lib/api/health.ts`
- Modify: `src/routes/Dashboard.svelte`

- [ ] **Step 1: Criar `src/lib/api/health.ts`**

```ts
import { commands, type HealthInfo } from "../bindings";

export async function healthCheck(): Promise<HealthInfo> {
  const result = await commands.healthCheck();
  if (result.status === "error") {
    throw new Error(result.error);
  }
  return result.data;
}
```

> **Nota:** tauri-specta envolve commands em `{ status: "ok", data } | { status: "error", error }`. O wrapper acima desempacota.

- [ ] **Step 2: Atualizar `src/routes/Dashboard.svelte`**

```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import { healthCheck } from "$lib/api/health";
  import type { HealthInfo } from "$lib/../bindings";

  let info = $state<HealthInfo | null>(null);
  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      info = await healthCheck();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  });
</script>

<section class="p-8">
  <h2 class="text-xl font-semibold">Dashboard</h2>
  <p class="text-fg-muted mt-1">KPIs, donut, barras, recent.</p>

  <div class="mt-6 p-4 rounded-lg bg-surface border border-border-subtle max-w-xl">
    <div class="text-[10.5px] font-semibold uppercase tracking-wider text-fg-faint mb-2">
      Health check
    </div>
    {#if error}
      <div class="text-neg text-sm">Erro: {error}</div>
    {:else if info}
      <div class="grid grid-cols-[120px_1fr] gap-y-1 text-[12px]">
        <span class="text-fg-muted">Version</span><span class="tabular">{info.version}</span>
        <span class="text-fg-muted">DB path</span><span class="font-mono text-[11px] break-all">{info.db_path}</span>
        <span class="text-fg-muted">Categories</span><span class="tabular">{info.category_count}</span>
      </div>
    {:else}
      <div class="text-fg-faint text-sm">Carregando…</div>
    {/if}
  </div>
</section>
```

- [ ] **Step 3: Type-check**

Run:
```bash
pnpm check
```

Expected: zero erros. Se `bindings.ts` ainda não existir, rodar `pnpm tauri dev` brevemente (~20s) pra gerá-lo e tentar de novo.

- [ ] **Step 4: Verificar visualmente**

Run:
```bash
pnpm tauri dev
```

Expected: ir pra rota Dashboard (default). Painel "Health check" mostra version=0.1.0, db_path=`/Users/.../app.finan/finan.db`, categories=9. Fechar Cmd+Q.

- [ ] **Step 5: Commit**

```bash
git add .
git commit -m "$(cat <<'EOF'
feat(frontend): consumir health_check via bindings tipados

- lib/api/health.ts wrapper de commands.healthCheck() desempacotando Result
- Dashboard.svelte renderiza version, db_path, category_count
- Demonstra fronteira IPC end-to-end tipada (Rust struct -> TS type)

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 8: shadcn-svelte init + Button + uso em Onboarding

**Files:**
- Create: `components.json`
- Modify: `package.json` (deps adicionadas pelo shadcn)
- Create: `src/lib/components/ui/button/` (pasta com componentes shadcn)
- Modify: `src/routes/Onboarding.svelte`

- [ ] **Step 1: Inicializar shadcn-svelte**

Run:
```bash
pnpm dlx shadcn-svelte@latest init
```

**Responder aos prompts:**
- Which style? → **Default**
- Which base color? → **Slate** (será sobreposto pelos nossos tokens; escolha qualquer)
- Where is your global CSS? → `src/app.css`
- Configure import alias for components? → `$lib/components`
- Configure import alias for ui? → `$lib/components/ui`
- Configure import alias for utils? → `$lib/utils`
- Configure import alias for hooks? → `$lib/hooks`

Expected: cria `components.json`, `src/lib/utils.ts`, adiciona deps (`bits-ui`, `clsx`, `tailwind-merge`, `tailwind-variants`, `lucide-svelte`).

- [ ] **Step 2: Adicionar Button component**

Run:
```bash
pnpm dlx shadcn-svelte@latest add button
```

Expected: cria `src/lib/components/ui/button/` com `button.svelte` e `index.ts`.

- [ ] **Step 3: Atualizar Onboarding pra usar Button**

Edit `src/routes/Onboarding.svelte`:

```svelte
<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { push } from "svelte-spa-router";

  function goToImport() {
    push("/import");
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

  <div class="flex justify-center gap-2">
    <Button onclick={goToImport}>Importar meu primeiro OFX</Button>
    <Button variant="outline" onclick={() => push("/dashboard")}>Ver dashboard</Button>
  </div>
</section>
```

- [ ] **Step 4: Type-check**

Run:
```bash
pnpm check
```

Expected: zero erros.

- [ ] **Step 5: Verificar visualmente**

Run:
```bash
pnpm tauri dev
```

Manual: navegar pra `#/onboarding`. Verificar 2 botões. Clicar `Importar meu primeiro OFX` leva pra `/import`. Fechar Cmd+Q.

Expected: botões renderizam com estilo shadcn padrão (cores podem não bater com o palette ainda — refinamento de estilos shadcn fica pra Fase 5 polish).

- [ ] **Step 6: Commit**

```bash
git add .
git commit -m "$(cat <<'EOF'
feat(ui): shadcn-svelte init + Button + Onboarding skeleton

- components.json, src/lib/utils.ts (cn helper)
- Button component (bits-ui + tailwind-variants)
- Onboarding.svelte com hero e dois botões navegáveis
- Refinamento dos estilos shadcn pro palette finan: fase 5

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 9: Vitest + sample test (TS infra)

**Files:**
- Modify: `package.json` (deps + script)
- Create: `vitest.config.ts`
- Create: `src/lib/format/money.ts`
- Create: `src/lib/format/money.test.ts`

- [ ] **Step 1: Instalar Vitest**

Run:
```bash
pnpm add -D vitest @vitest/ui jsdom
```

- [ ] **Step 2: Criar `vitest.config.ts`**

```ts
import { defineConfig } from "vitest/config";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { fileURLToPath, URL } from "node:url";

export default defineConfig({
  plugins: [svelte({ hot: false })],
  resolve: {
    alias: {
      $lib: fileURLToPath(new URL("./src/lib", import.meta.url)),
    },
  },
  test: {
    environment: "jsdom",
    globals: true,
    include: ["src/**/*.{test,spec}.ts"],
  },
});
```

- [ ] **Step 3: Adicionar script `test` ao `package.json`**

Edit `package.json`, replace `scripts` section:

```json
"scripts": {
  "dev": "vite",
  "build": "vite build",
  "preview": "vite preview",
  "check": "svelte-check --tsconfig ./tsconfig.json",
  "test": "vitest run",
  "test:watch": "vitest",
  "tauri": "tauri"
}
```

- [ ] **Step 4: Criar `src/lib/format/money.ts`**

```ts
const FORMATTER = new Intl.NumberFormat("pt-BR", {
  style: "currency",
  currency: "BRL",
});

export function formatMoney(amount: string): string {
  const n = Number(amount);
  if (!Number.isFinite(n)) {
    throw new Error(`invalid amount: ${amount}`);
  }
  return FORMATTER.format(n);
}
```

> **Nota:** `src/lib/utils.ts` é criado pela shadcn-svelte init em T8 com a helper `cn()`. Mantemos formatadores numa pasta separada (`format/`) pra evitar conflito de path entre arquivo (`utils.ts`) e diretório homônimo.

- [ ] **Step 5: Criar `src/lib/format/money.test.ts` (TDD: escrever teste primeiro, depois confirmar passa)**

```ts
import { describe, it, expect } from "vitest";
import { formatMoney } from "./money";

describe("formatMoney", () => {
  it("formats positive amount as BRL with two decimals", () => {
    expect(formatMoney("123.45")).toMatch(/R\$\s*123,45/);
  });

  it("formats negative amount with minus sign", () => {
    expect(formatMoney("-50.00")).toMatch(/-R\$\s*50,00|R\$\s*-50,00/);
  });

  it("formats zero", () => {
    expect(formatMoney("0")).toMatch(/R\$\s*0,00/);
  });

  it("throws on non-numeric input", () => {
    expect(() => formatMoney("abc")).toThrow(/invalid amount/);
  });
});
```

- [ ] **Step 6: Rodar testes**

Run:
```bash
pnpm test
```

Expected: 4 testes passam em `format-money.test.ts`.

- [ ] **Step 7: Commit**

```bash
git add .
git commit -m "$(cat <<'EOF'
test(ts): Vitest infra + sample test em format-money

- vitest, @vitest/ui, jsdom
- vitest.config.ts com alias $lib
- lib/format/money.ts: wrapper Intl.NumberFormat pra BRL
- lib/format/money.test.ts: 4 casos (positivo, negativo, zero, inválido)
- script pnpm test rodando 4/4 passes

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 10: Acceptance verification + handoff pra Fase 1

**Files:**
- (nenhum criado/modificado — só verificação)
- (opcional) Modify: `README.md` adicionando seção "Status"

- [ ] **Step 1: Type-check completo**

Run:
```bash
pnpm check
```

Expected: zero erros, zero warnings.

- [ ] **Step 2: Rust tests + lint**

Run:
```bash
cd src-tauri && cargo test --lib && cargo clippy --all-targets -- -D warnings && cargo fmt --check && cd ..
```

Expected: 5 testes passam, clippy limpo, fmt OK.

- [ ] **Step 3: TS tests**

Run:
```bash
pnpm test
```

Expected: 4/4 passes.

- [ ] **Step 4: Build de produção (sanity check)**

Run:
```bash
pnpm build
```

Expected: `dist/` gerado sem erros. Vite reporta tamanho do bundle (esperado < 200KB gzipped pra esse escopo).

- [ ] **Step 5: Dev launch + walkthrough manual**

Run:
```bash
pnpm tauri dev
```

**Checklist visual (cada item deve passar antes de fechar):**

- [ ] Janela abre em <2s.
- [ ] Sidebar à esquerda com brand mark verde, seções "Visão geral / Importar / Organizar".
- [ ] Default route = `/dashboard`, painel "Health check" mostra:
  - version = `0.1.0`
  - db_path termina em `app.finan/finan.db`
  - categories = `9`
- [ ] Clicar `Transações` → tela com header "Transações" + texto stub.
- [ ] Clicar `Importar OFX` → tela "Importar OFX" + texto stub.
- [ ] Clicar `Categorias` → tela "Categorias" + texto stub.
- [ ] Clicar `Configurações` → tela "Configurações" + texto stub.
- [ ] Item ativo da sidebar muda visualmente (fundo verde-suave).
- [ ] Navegar manualmente pra `#/onboarding` → vê hero + 2 botões shadcn.
- [ ] Botão "Importar meu primeiro OFX" no onboarding navega pra `/import`.

Fechar com Cmd+Q.

- [ ] **Step 6: Confirmar DB com schema completo**

Run:
```bash
sqlite3 ~/Library/Application\ Support/app.finan/finan.db "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name;"
sqlite3 ~/Library/Application\ Support/app.finan/finan.db "SELECT name, kind FROM categories ORDER BY id;"
```

Expected:
```
_migrations
accounts
categories
transactions
```
e 9 categorias listadas com kinds `expense`/`income`.

- [ ] **Step 7: (Opcional) Atualizar README com status**

Edit `README.md`, append section before "Stack":

```markdown
## Status

- ✅ Fase 0 — Scaffold (Tauri + Svelte + DB + sidebar)
- 🚧 Fase 1 — Importar OFX (próximo)
- ⏳ Fase 2-5 — Categorização, regras, dashboard, polish
```

- [ ] **Step 8: Commit de fechamento**

```bash
git add .
git commit -m "$(cat <<'EOF'
chore: fase 0 scaffold concluída — acceptance criteria batem

Critérios verificados:
- pnpm tauri dev abre janela com sidebar funcional
- 6 rotas navegáveis via svelte-spa-router
- SQLite criado em ~/Library/Application Support/app.finan/finan.db
  com tabelas accounts, categories, transactions e 9 categorias seed
- Comando health_check retorna struct tipado consumido no Dashboard
- cargo test: 5 passes / pnpm test: 4 passes / pnpm check: 0 erros

Próximo: plano da Fase 1 (Importar OFX).

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

- [ ] **Step 9: (Opcional) Tag**

Run:
```bash
git tag -a v0.1.0-scaffold -m "Fase 0 scaffold complete"
git tag
```

Expected: tag `v0.1.0-scaffold` aparece no `git tag`.

---

## Próximos planos (referência)

Cada fase abaixo vira um plano separado em `docs/superpowers/plans/`, gerado quando a fase anterior fechar:

| Plano | Foco |
|---|---|
| `2026-MM-DD-fase-1-import-ofx.md` | Parser OFX (TS) + comando `insert_transactions` + dedup por FITID + lista crua de tx |
| `2026-MM-DD-fase-2-categorization.md` | Categorias UI + inline picker + filtro por mês/categoria + notes |
| `2026-MM-DD-fase-3-rules.md` | Migration 0002 + CRUD de regras + aplicação no import + comando aplicar-existentes |
| `2026-MM-DD-fase-4-dashboard.md` | LayerChart + commands de summary + KPIs + donut + barras + recent |
| `2026-MM-DD-fase-5-polish.md` | Search global + Settings (path, backup/restore) + atalhos + refinamento visual |

Cada plano referencia a spec mestre `docs/superpowers/specs/2026-05-13-finan-mvp-design.md`.

---

## Self-Review — coverage da spec

Checklist verificado contra `docs/superpowers/specs/2026-05-13-finan-mvp-design.md`:

| Item da spec | Onde no plano |
|---|---|
| Tauri 2 + Svelte 5 + Vite scaffold | T1 |
| Tailwind 4 + tokens do palette (§6 spec) | T2 |
| svelte-spa-router (§4 spec) | T3 |
| Sidebar com seções da nav (§7 spec) | T4 |
| rusqlite bundled + migration 0001 (§5 spec) | T5 |
| Tabelas accounts/categories/transactions + 9 seeds (§5 spec) | T5 (migration + tests) |
| WAL + foreign_keys | T5 |
| tauri-specta gerando bindings (§3 spec) | T6 |
| Comando tipado roundtrip (princípio "UI nunca toca SQL") | T6+T7 |
| shadcn-svelte (§3.6 spec) | T8 |
| Vitest infra (§10 spec) | T9 |
| Cargo test infra (§10 spec) | T5, T6 |
| DB em `~/Library/Application Support/.../finan.db` (§3 spec) | T5 (verificado em T10) |
| CSP sem rede externa (§11.6 spec) | T1 (`tauri.conf.json`) |
| AppError com thiserror (§11.2 spec) | T5 |

Gaps intencionais (fora do escopo de Fase 0, cobertos em planos futuros):
- Parser OFX → Fase 1
- Rules engine → Fase 3
- LayerChart → Fase 4
- Settings/backup → Fase 5
- E2E tests → "skip no MVP" conforme spec §10
