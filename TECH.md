# Documentação Técnica — finan-app

App desktop de finanças pessoais pra macOS. 100% local, 100% gratuito, leve, minimalista. Este documento descreve as decisões técnicas, a stack escolhida e as ressalvas de cada componente.

## Sumário

- [Visão geral da arquitetura](#visão-geral-da-arquitetura)
- [Stack e justificativas](#stack-e-justificativas)
  - [1. Tauri 2 — framework do app](#1-tauri-2--framework-do-app)
  - [2. Svelte 5 + Vite — frontend](#2-svelte-5--vite--frontend)
  - [3. shadcn-svelte — componentes UI](#3-shadcn-svelte--componentes-ui)
  - [4. Camada de dados — SQLite + Rust](#4-camada-de-dados--sqlite--rust)
  - [5. LayerChart — gráficos](#5-layerchart--gráficos)
  - [6. Parsing de OFX](#6-parsing-de-ofx)
- [Fluxo de dados](#fluxo-de-dados)
- [Tratamento de valores monetários](#tratamento-de-valores-monetários)
- [Distribuição no macOS](#distribuição-no-macos)
- [Resumo de decisões](#resumo-de-decisões)

---

## Visão geral da arquitetura

```
┌─────────────────────────────────────────────────────────────┐
│                         macOS (WKWebView)                   │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  Frontend (Svelte 5 + Vite)                           │  │
│  │  - UI: shadcn-svelte                                  │  │
│  │  - Charts: LayerChart                                 │  │
│  │  - OFX parsing: ofx-data-extractor (TS)               │  │
│  └─────────────────────────┬─────────────────────────────┘  │
│                            │ Tauri IPC (commands tipados)   │
│                            │ tipos gerados via tauri-specta │
│  ┌─────────────────────────▼─────────────────────────────┐  │
│  │  Backend (Rust)                                       │  │
│  │  - Commands: list/insert/categorize transactions      │  │
│  │  - rusqlite (SQLite bundled)                          │  │
│  │  - rust_decimal (valores monetários)                  │  │
│  └─────────────────────────┬─────────────────────────────┘  │
│                            │                                │
│  ┌─────────────────────────▼─────────────────────────────┐  │
│  │  SQLite local (~/Library/Application Support/finan/)  │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

**Princípios que orientam as decisões:**
- 100% local — nenhum dado sai do Mac
- 100% gratuito — sem assinaturas, sem APIs pagas
- Leve — binário pequeno, abre rápido, baixo consumo de RAM
- Minimalista — só o essencial na UI e no código

---

## Stack e justificativas

### 1. Tauri 2 — framework do app

**Versão:** 2.x (estável desde out/2024).

**Por quê:**
- No macOS, Tauri usa o **WKWebView nativo** (não embute Chromium). Resultado típico: binário 2–10 MB, ~30–50 MB de RAM idle, abertura em <500 ms.
- Lógica sensível (acesso a SQLite, sistema de arquivos) fica no Rust, isolada do WebView por uma fronteira de IPC explícita.
- Bate com os princípios "leve" e "100% local" sem trabalho extra.

**Ressalvas:**
- **Notarization da Apple tem ficado lenta** (filas presas por 24–72h+ relatadas desde jan/2026). Não é bug do Tauri, é da Apple. Use `notarytool` (`altool` foi descontinuado).
- **Conta Apple Developer paga (US$ 99/ano)** é necessária pra distribuir sem aviso de "app não verificado". Tensiona com o princípio "100% gratuito" se houver intenção de compartilhar o app — pra uso pessoal local, dá pra dispensar (ver [Distribuição](#distribuição-no-macos)).

**Alternativas descartadas:** Electron (bundle 5–10x maior, RAM 2–3x maior); Wails (Go, ecossistema menor pra desktop).

---

### 2. Svelte 5 + Vite — frontend

**Versão:** Svelte 5 (estável desde out/2024), Vite 5+.

**Por quê:**
- Compilador-first → bundles minúsculos, ideal pra UI minimalista.
- **Runes** (`$state`, `$derived`, `$effect`) funcionam fora de `.svelte` (em `.svelte.ts`), o que permite isolar lógica de domínio (categorização, agregação por mês) da camada de view.
- Sem SSR no caso (é desktop), então **Vite puro sem SvelteKit** — menos cerimônia.

**Convenções a adotar:**
- `$state` pra estado mutável; `$state.raw` quando o valor é grande/imutável (listas grandes de transações).
- Stores legadas só onde fizer sentido (compatibilidade com lib externa).
- Lógica de domínio em `.svelte.ts`; componentes `.svelte` só renderizam.

**Ressalvas:** Runes têm curva de aprendizado se você vem de Svelte 3/4. Documente convenções desde cedo no `CONTRIBUTING` (quando criar) ou neste arquivo.

---

### 3. shadcn-svelte — componentes UI

**Versão:** 1.x (mar/2026), com Tailwind 4 e Svelte 5.

**Por quê:**
- Modelo **copy-paste**: você é dono do código dos componentes, **zero peso de biblioteca em runtime**.
- Fácil customizar pra estética minimalista (paleta neutra, espaçamento generoso, foco em tipografia).
- Tem **integração oficial com LayerChart** via componente `<Chart>`.

**Disciplina necessária:**
- Use a CLI (`pnpm dlx shadcn-svelte add <componente>`).
- Revise diffs ao atualizar — componentes copiados podem ter sido customizados localmente.
- Capriche em **components de form** (máscara monetária, date pickers BR, validação) — a base é boa mas a customização pra finanças é com você.

---

### 4. Camada de dados — SQLite + Rust

> Aqui está a decisão mais importante e a única divergência do README original.

#### Decisão

**Manter SQLite. Não usar `@tauri-apps/plugin-sql` + Drizzle ORM.** Em vez disso:

- **`rusqlite` (Rust)** com feature `bundled` (SQLite compilado dentro do binário, zero dependência de sistema).
- **Tauri commands tipados** expostos do Rust pro frontend (`list_transactions`, `insert_transactions`, `categorize`, etc.).
- **`tauri-specta`** (ou `ts-rs`) pra gerar tipos TS a partir dos structs Rust — type-safety end-to-end.
- **`rust_decimal`** pra todos os valores monetários (ver [Tratamento de valores monetários](#tratamento-de-valores-monetários)).

#### Por que NÃO Drizzle + plugin-sql

- Drizzle **não tem driver nativo** pra `@tauri-apps/plugin-sql`. A integração que circula em 2026 usa o `sqlite-proxy` driver: serializa SQL → IPC → plugin Rust → resposta. Funciona, mas:
  - Migrations do Drizzle dependem de filesystem Node — exige gambiarra com `import.meta.glob` do Vite pra empacotar `.sql`.
  - Overhead de IPC em cada query.
  - Você está colando pedaços sem suporte oficial de nenhum dos dois times.
- Pra um app onde SQLite é único, queries são poucas e estáveis (~10–15 no MVP inteiro), ORM é overkill.
- Dinheiro precisa de `Decimal`. Drizzle/plugin-sql vão te empurrar pra number/string e fazer conversões silenciosas — exatamente o tipo de bug que destrói confiança num app de finanças.

#### Plano B (se rusqlite parecer pesado)

Manter `@tauri-apps/plugin-sql`, **largar o Drizzle**, usar o cliente direto com helpers TS escritos à mão. Menos type-safety, zero ginástica de proxy. **Não é recomendado** mas é aceitável.

#### Esquema inicial (rascunho)

```sql
CREATE TABLE accounts (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  bank TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE categories (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  color TEXT,
  parent_id INTEGER REFERENCES categories(id)
);

CREATE TABLE transactions (
  id INTEGER PRIMARY KEY,
  account_id INTEGER NOT NULL REFERENCES accounts(id),
  date TEXT NOT NULL,                    -- ISO 8601
  amount TEXT NOT NULL,                  -- string serializada de Decimal
  description TEXT NOT NULL,
  category_id INTEGER REFERENCES categories(id),
  ofx_fitid TEXT,                        -- pra evitar duplicatas no reimport
  imported_at TEXT NOT NULL DEFAULT (datetime('now')),
  UNIQUE(account_id, ofx_fitid)
);

CREATE TABLE rules (
  id INTEGER PRIMARY KEY,
  pattern TEXT NOT NULL,                 -- regex/contains
  category_id INTEGER NOT NULL REFERENCES categories(id),
  priority INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_transactions_date ON transactions(date);
CREATE INDEX idx_transactions_category ON transactions(category_id);
```

Valores monetários ficam como `TEXT` no SQLite (serialização do `Decimal`), nunca `REAL`.

---

### 5. LayerChart — gráficos

**Versão:** ativamente desenvolvido em 2026 (pre-release de 01/mai/2026 trouxe Canvas/perf).

**Por quê:**
- Svelte-nativo (sem wrapper), composable, integrado oficialmente com shadcn-svelte.
- Cobre todos os casos do MVP: donut/bar (gastos por categoria), area/line (série temporal).

**Ressalva:** Curva maior que Chart.js — você compõe escalas/eixos/séries em vez de passar um config object. Pra gráficos triviais isso pode parecer trabalho extra.

**Plano B:** Chart.js + svelte-chartjs. +70 KB gzipped, mas config-first e ecossistema enorme. Trocar é fácil — gráficos são isolados em componentes.

---

### 6. Parsing de OFX

**Não existe biblioteca OFX madura em Rust** (crates.io não tem nada de referência em 2026). **Parsing fica no TypeScript**, dentro do WebView.

**Biblioteca recomendada:** [`ofx-data-extractor`](https://github.com/Fabiopf02/ofx-data-extractor) — TS-first, modos strict/lenient, validação, roda em browser.

**Fluxo:**
1. Usuário arrasta `.ofx` na UI.
2. TS lê o arquivo (`FileReader`).
3. `ofx-data-extractor` parseia → array de transações tipadas.
4. Frontend chama `invoke('insert_transactions', { txs })`.
5. Rust valida, converte `amount` pra `Decimal`, persiste no SQLite usando `ofx_fitid` como chave de deduplicação.

**Atenção crítica — OFX brasileiro:**
- Itaú, Bradesco, Nubank etc. têm variações: encoding ISO-8859-1 (não UTF-8), tags fora do padrão, FITID inconsistente.
- **Teste com extratos reais dos seus bancos antes de assumir que qualquer parser funciona.**
- Provavelmente vai precisar de uma camada de normalização por banco (detectar banco pelo header e aplicar transformações específicas).

---

## Fluxo de dados

**Import:**
```
arquivo .ofx (drag-and-drop)
    │
    ▼
Frontend (TS) — parseia com ofx-data-extractor
    │
    ▼
invoke('insert_transactions', { account_id, txs })
    │
    ▼
Rust — valida, converte para Decimal, aplica regras de categorização
    │
    ▼
SQLite — INSERT com UNIQUE(account_id, ofx_fitid) pra dedupe
```

**Consulta (dashboard):**
```
UI requer "gastos por categoria no mês X"
    │
    ▼
invoke('summary_by_category', { month: '2026-05' })
    │
    ▼
Rust executa query agregada no SQLite
    │
    ▼
Retorna Vec<CategorySummary> tipado (gerado via tauri-specta)
    │
    ▼
LayerChart renderiza o donut/bar
```

---

## Tratamento de valores monetários

**Regra inegociável:** dinheiro nunca é `f32`/`f64`/`number`. Sempre `Decimal`.

- **Rust:** `rust_decimal::Decimal`. Serializa pra string ao persistir/transmitir.
- **TypeScript:** receba como `string` no contrato do command. Pra exibir, formate com `Intl.NumberFormat('pt-BR', { style: 'currency', currency: 'BRL' })`. Pra somar no frontend (raramente necessário — preferível somar no Rust), use `decimal.js` ou similar.
- **SQLite:** coluna `TEXT`, não `REAL`. SQLite não tem tipo decimal nativo.
- **Operações sensíveis (somas, médias, conversão de moeda)** acontecem no Rust, nunca no JS.

---

## Distribuição no macOS

**Pra uso pessoal (sem distribuir):**
- Build local com `pnpm tauri build`.
- App fica em `src-tauri/target/release/bundle/macos/`.
- Como não é assinado, no primeiro `open` o macOS bloqueia. Solução: `xattr -d com.apple.quarantine /Applications/finan.app` (ou Ctrl+clique → Abrir).
- **Custo: zero.** Bate com o princípio "100% gratuito".

**Pra distribuir (futuro, opcional):**
- Apple Developer Account: US$ 99/ano.
- Code signing com Developer ID Application certificate.
- Notarization via `notarytool` (NÃO `altool`, descontinuado).
- Stapling do ticket no `.app` antes de empacotar `.dmg`.
- Em jan/2026, filas de notarization estavam lentas (24–72h+). Não bloqueia desenvolvimento; só atrasa releases.

---

## Resumo de decisões

| Camada | Escolha | Status |
|---|---|---|
| App framework | **Tauri 2** | Mantida |
| Frontend | **Svelte 5 + Vite** (sem SvelteKit) | Mantida |
| UI components | **shadcn-svelte** | Mantida |
| Banco | **SQLite** | Mantida |
| Acesso ao banco | **rusqlite + Tauri commands tipados (via tauri-specta)** | **Mudou** (era Drizzle + plugin-sql) |
| Valores monetários | **rust_decimal** (Rust) + string no contrato IPC | Adicionada |
| Charts | **LayerChart** (plano B: Chart.js) | Mantida |
| OFX parsing | **ofx-data-extractor** no frontend | Adicionada |
| Build/distribuição | Build local sem signing (uso pessoal). Apple Developer opcional pra distribuir. | Definida |

## Fontes consultadas

- [Tauri 2.0 Stable Release](https://v2.tauri.app/blog/tauri-20/)
- [Tauri Core Ecosystem Releases](https://v2.tauri.app/release/)
- [Tauri vs Electron 2026 — PkgPulse](https://www.pkgpulse.com/blog/electron-vs-tauri-2026)
- [macOS Code Signing — Tauri docs](https://v2.tauri.app/distribute/sign/macos/)
- [Notarization stuck — Tauri discussion #8630](https://github.com/orgs/tauri-apps/discussions/8630)
- [Notarization stuck Jan 2026 — Tauri issue #14579](https://github.com/tauri-apps/tauri/issues/14579)
- [@tauri-apps/plugin-sql docs](https://v2.tauri.app/plugin/sql/)
- [Drizzle + SQLite in Tauri (Huakun)](https://huakun.tech/blogs/drizzle-+-sqlite-in-Tauri-App)
- [Building a Local-First Tauri App with Drizzle, Encryption, Turso](https://dev.to/huakun/building-a-local-first-tauri-app-with-drizzle-orm-encryption-and-turso-sync-31pn)
- [Rust ORMs in 2026 — Diesel vs SQLx vs SeaORM vs Rusqlite](https://aarambhdevhub.medium.com/rust-orms-in-2026-diesel-vs-sqlx-vs-seaorm-vs-rusqlite-which-one-should-you-actually-use-706d0fe912f3)
- [Introducing Runes — Svelte blog](https://svelte.dev/blog/runes)
- [shadcn-svelte — Svelte 5 migration guide](https://www.shadcn-svelte.com/docs/migration/svelte-5)
- [LayerChart](https://www.layerchart.com/)
- [ofx-data-extractor](https://github.com/Fabiopf02/ofx-data-extractor)
