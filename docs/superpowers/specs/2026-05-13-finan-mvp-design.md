# finan — MVP Design Spec

**Data:** 2026-05-13
**Status:** Aprovado (brainstorming) → próximo passo: writing-plans

## 1. Contexto e objetivos

App desktop pra macOS de finanças pessoais. 100% local (SQLite no Mac), 100% gratuito, leve, minimalista. Fluxo nuclear: importar extrato OFX → categorizar → ver dashboard.

**Stack confirmada (ver [TECH.md](../../../TECH.md)):**
- Tauri 2 (framework desktop)
- Svelte 5 + Vite (frontend, sem SvelteKit)
- Tailwind 4 + shadcn-svelte (UI)
- rusqlite (`bundled`) + Tauri commands tipados via `tauri-specta`
- `rust_decimal` pra dinheiro
- LayerChart (gráficos)
- `ofx-data-extractor` no frontend (TS) pra parsing OFX

**Critérios de sucesso do MVP:**
- Importar um extrato OFX brasileiro real e ver as transações persistidas sem duplicação em reimport.
- Categorizar transações manualmente, com persistência.
- Dashboard mostra renda/gastos/saldo do mês + donut por categoria + barras 12m + últimas tx.
- Binário macOS bundled (`pnpm tauri build`) abre em <1s e ocupa <50 MB de RAM idle.
- Zero comunicação de rede em runtime (auditável).

## 2. Decisões aprovadas

| Tema | Decisão |
|---|---|
| Slicing do MVP | **Thin-slice vertical end-to-end** (fases 0–5, ver §7). |
| Categorização | **Manual no MVP.** Regras "description contains" entram na fase 3. |
| Multi-conta | **Data model multi-conta desde o início.** UI esconde seletor enquanto houver só uma. |
| Edição de transações | **Só categoria e notes editáveis.** Date, amount, description são read-only após import. |
| Budgets | **Defer pro pós-MVP.** Dashboard mostra "top categorias do mês" sem orçamento. |
| Estilo | **Tailwind 4 + tokens CSS** extraídos do palette de `examples/assets/app.css`. Wireframes de `examples/` NÃO são reutilizados — só inspiração de palette e estrutura visual. |

## 3. Arquitetura

```
┌─────────────────────────────────────────────────────────────┐
│                    macOS (WKWebView)                        │
│  ┌───────────────────────────────────────────────────────┐  │
│  │ Frontend (Svelte 5 + Vite + Tailwind 4)               │  │
│  │  · routes/     telas (onboarding, dashboard, …)       │  │
│  │  · lib/api/    wrappers tipados de invoke()           │  │
│  │  · lib/ofx/    parser TS (ofx-data-extractor + norm.) │  │
│  │  · lib/stores/ estado (.svelte.ts, runes)             │  │
│  │  · lib/components/ui  shadcn-svelte (copy-paste)      │  │
│  └─────────────────────────┬─────────────────────────────┘  │
│                            │ Tauri IPC                       │
│                            │ tipos gerados por tauri-specta  │
│  ┌─────────────────────────▼─────────────────────────────┐  │
│  │ Backend (Rust)                                        │  │
│  │  · commands/   accounts, transactions, categories,    │  │
│  │                summary (agregações)                   │  │
│  │  · domain/     structs com rust_decimal               │  │
│  │  · db/         conexão rusqlite + migrations          │  │
│  │  · error.rs    AppError (thiserror)                   │  │
│  └─────────────────────────┬─────────────────────────────┘  │
│  ┌─────────────────────────▼─────────────────────────────┐  │
│  │ SQLite: ~/Library/Application Support/finan/finan.db  │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

**Fronteiras invioláveis:**
- UI **nunca** toca SQL. Toda persistência vai por command Rust tipado.
- Toda operação monetária acontece no Rust com `rust_decimal::Decimal`. TS recebe `string` no contrato IPC.
- Parser OFX é o único módulo TS que toca "dados externos crus"; valida e normaliza antes de mandar pro Rust.

## 4. Estrutura de pastas

```
finan-app/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   ├── db/
│   │   │   ├── mod.rs              conexão + setup
│   │   │   ├── migrations.rs       migrations versionadas embed!
│   │   │   └── queries.rs          queries tipadas
│   │   ├── domain/
│   │   │   ├── account.rs
│   │   │   ├── transaction.rs      Transaction { amount: Decimal, … }
│   │   │   ├── category.rs
│   │   │   └── rule.rs             (fase 3)
│   │   ├── commands/
│   │   │   ├── accounts.rs         list, create
│   │   │   ├── transactions.rs     list (filter), insert_batch, update_category, update_notes
│   │   │   ├── categories.rs       list, create, update, delete
│   │   │   ├── rules.rs            (fase 3)
│   │   │   └── summary.rs          by_category(month), by_month(12), kpis(month), recent(n)
│   │   ├── error.rs                AppError + impl Serialize p/ IPC
│   │   └── lib.rs                  exports + tauri-specta config
│   ├── migrations/
│   │   ├── 0001_init.sql
│   │   └── 0002_rules.sql          (fase 3)
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/
│   ├── lib/
│   │   ├── api/                    accounts.ts, transactions.ts, … (wrappers de invoke)
│   │   ├── bindings.ts             gerado por tauri-specta (gitignored)
│   │   ├── ofx/
│   │   │   ├── parse.ts            wrapper de ofx-data-extractor
│   │   │   ├── normalize.ts        encoding ISO-8859-1, banco-específicos
│   │   │   └── types.ts            ParsedTransaction, ParsedAccount
│   │   ├── components/
│   │   │   ├── ui/                 shadcn-svelte (copy-paste)
│   │   │   ├── shell/              Sidebar.svelte, Toolbar.svelte
│   │   │   ├── transactions/       TxTable.svelte, CategoryPicker.svelte
│   │   │   ├── dashboard/          Kpi.svelte, CategoryDonut.svelte, MonthBars.svelte
│   │   │   └── import/             DropZone.svelte, ImportPreview.svelte
│   │   ├── stores/                 month.svelte.ts, account.svelte.ts
│   │   ├── styles/
│   │   │   └── tokens.css          @theme com palette do mockup
│   │   └── utils/                  format-money.ts, format-date.ts
│   ├── routes/
│   │   ├── routes.ts               mapa de rotas pro svelte-spa-router
│   │   ├── Onboarding.svelte
│   │   ├── Dashboard.svelte
│   │   ├── Transactions.svelte
│   │   ├── Import.svelte
│   │   ├── Categories.svelte
│   │   ├── Rules.svelte            (fase 3)
│   │   └── Settings.svelte
│   ├── App.svelte                  shell (sidebar + main + <Router/>)
│   ├── main.ts                     entry point Vite
│   └── app.html
├── docs/superpowers/specs/
├── README.md
├── TECH.md
├── CLAUDE.md
├── package.json
├── vite.config.ts
└── tailwind.config.ts              (Tailwind 4 usa @theme inline em tokens.css)
```

> **Roteamento:** Vite + Svelte puro (sem SvelteKit). Router escolhido: [`svelte-spa-router`](https://github.com/ItalyPaleAle/svelte-spa-router) (hash-based, simples, sem build extra). Path-based seria possível com `@roxi/routify` mas é overkill — desktop não precisa de URL real.

## 5. Esquema de dados

### Migration 0001 (fase 0)

```sql
CREATE TABLE accounts (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  bank TEXT,                              -- 'itau', 'nubank', 'bradesco', etc.
  ofx_acctid TEXT,                        -- ACCTID do OFX
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE categories (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  color_token TEXT,                       -- '--color-cat-mercado' etc.
  kind TEXT NOT NULL CHECK(kind IN ('expense','income','transfer')),
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE transactions (
  id INTEGER PRIMARY KEY,
  account_id INTEGER NOT NULL REFERENCES accounts(id),
  date TEXT NOT NULL,                     -- ISO 8601 (YYYY-MM-DD)
  amount TEXT NOT NULL,                   -- rust_decimal::Decimal serializado
  description TEXT NOT NULL,
  category_id INTEGER REFERENCES categories(id),
  notes TEXT,
  ofx_fitid TEXT,
  imported_at TEXT NOT NULL DEFAULT (datetime('now')),
  UNIQUE(account_id, ofx_fitid)           -- chave de dedupe no reimport
);

CREATE INDEX idx_tx_date ON transactions(date);
CREATE INDEX idx_tx_category ON transactions(category_id);
CREATE INDEX idx_tx_account ON transactions(account_id);

-- categorias seed
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

### Migration 0002 (fase 3)

```sql
CREATE TABLE rules (
  id INTEGER PRIMARY KEY,
  pattern TEXT NOT NULL,                  -- substring case-insensitive
  category_id INTEGER NOT NULL REFERENCES categories(id),
  priority INTEGER NOT NULL DEFAULT 0,    -- maior = aplicado primeiro
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_rules_priority ON rules(priority DESC);
```

### Valores monetários — invariantes

- Rust: `rust_decimal::Decimal`.
- SQLite: `TEXT` (string de `Decimal`), **nunca `REAL`**.
- IPC: serializado como `string` pra TS.
- TS: nunca soma/multiplica. Formata com `Intl.NumberFormat('pt-BR', { style: 'currency', currency: 'BRL' })`. Operações vão pro Rust.

## 6. Design system

`src/lib/styles/tokens.css` extrai a palette do mockup pro `@theme` do Tailwind 4. Veja TECH.md §3 e o snippet abaixo.

**Tokens-chave (do `examples/assets/app.css`):**

| Categoria | Tokens |
|---|---|
| Surfaces | `--color-bg`, `--color-surface`, `--color-surface-2`, `--color-surface-3`, `--color-hover` |
| Borders | `--color-border`, `--color-border-subtle` |
| Text | `--color-fg`, `--color-fg-muted`, `--color-fg-subtle`, `--color-fg-faint` |
| Accent (moss) | `--color-accent`, `--color-accent-hi`, `--color-accent-on` |
| Categorical | `--color-cat-{mercado,transporte,restaurante,casa,saude,lazer,assinatura,renda,outros}` |
| Semantic | `--color-pos`, `--color-neg` |
| Radii | `--radius-sm/md/lg/xl` |
| Fonts | `--font-display`, `--font-body`, `--font-mono` (SF Pro / SF Mono no macOS) |

**Regras de uso:**
- Dark-first; `color-scheme: dark` aplicado no `html`.
- Números monetários sempre com `font-variant-numeric: tabular-nums` (helper `.tabular`).
- Toolbar com `backdrop-filter: saturate(180%) blur(20px)` em `--color-bg-translucent` (estilo macOS).
- Letter-spacing negativo no display (-0.015 a -0.03em) pra dar peso visual.
- Tipografia base 13px (compacta, macOS-like), display até 32px (onboarding).

**shadcn-svelte:** componentes copiados mapeiam suas cores semânticas (`bg-background`, `text-foreground`, `border-border`) pros tokens acima via aliases no `@theme`. Não usar paleta default do shadcn.

## 7. Navegação

```
Sidebar (232px, fundo --color-surface)
├── Brand · "finan · 100% local"
├── Visão geral
│   ├── Dashboard           (default ao abrir, se houver dados)
│   └── Transações          (count = total no DB)
├── Importar
│   └── Importar OFX
├── Organizar
│   ├── Categorias          (count = total)
│   └── Regras              (fase 3)
├── ─── spacer ───
├── Configurações
└── Account chip (path do DB)
```

**Estado inicial (DB vazia):** rota `/` redireciona pra `/onboarding`. Após primeiro import bem-sucedido, vira `/dashboard`.

## 8. Fluxos principais

### 8.1 Onboarding (primeiro uso, DB vazia)

1. Hero centralizado: brand mark + título "Suas finanças, no seu Mac." + subtítulo.
2. Grid de 4 princípios (100% local · Sem login · Leve · OFX padrão).
3. Drop zone gigante: arrastar OFX OU `Escolher arquivo…` OU link "Ver com dados de exemplo" (fase 5, opcional).
4. Footer: caminho do DB.
5. Ao soltar arquivo válido → vai pra `/import?file=<path>`.

### 8.2 Import OFX

1. **Detecção:** file-card mostra nome do arquivo, banco detectado (badge verde), número de transações, valor total in/out, período coberto.
2. **Preview:** tabela com checkbox por linha + dropdown de categoria (vazia no MVP; com sugestão `is-rule` na fase 3). Header da preview tem search e filter "todas / sem categoria".
3. **Right pane:** summary card (entradas, saídas, líquido, novas vs duplicadas).
4. **Footer-bar:** botão `Importar N transações` (primary), `Cancelar` (ghost).
5. **Pós-import:** redireciona pra `/transactions?since=<imported_at>` mostrando só as recém-importadas.
6. **Dedupe:** transações com `(account_id, ofx_fitid)` já existentes são marcadas como "duplicada" e desmarcadas por padrão.

### 8.3 Transações

- Tabela: Date · Description · Category · Amount.
- Filtros (chips no toolbar): mês (default = atual), categoria, conta (escondido se 1 conta).
- Search global (`⌘F`): match em description e notes.
- Categoria editável inline (`CategoryPicker.svelte` — popover com lista + criar nova).
- Notes: editável via right pane (clicar na linha abre detail).
- Ordenação por qualquer coluna.

### 8.4 Dashboard

**Toolbar:** seletor de mês (`← Mai/2026 →`) + range custom (post-MVP).

**Layout:**
```
┌────────────────────────────────────────────────┐
│ [KPI Renda] [KPI Gastos] [KPI Saldo] [KPI Tx]  │
├──────────────────────┬─────────────────────────┤
│ Donut: gastos        │ Barras: 12 meses        │
│ por categoria        │ (income vs expense)     │
│ + legenda            │                         │
├──────────────────────┼─────────────────────────┤
│ Top 5 categorias     │ Últimas 8 transações    │
│ (meter por % do mês) │                         │
└──────────────────────┴─────────────────────────┘
```

**Render:** LayerChart pro donut e barras. Sparkline nos KPIs com `<div>`s simples (CSS only, sem lib). Se LayerChart travar, fallback CSS conic-gradient pro donut (já demonstrado em `examples/dashboard.html`).

### 8.5 Categorias

- Lista com swatch de cor + nome + count de transações.
- Criar (modal): nome + picker de cor (paleta categorical) + kind.
- Editar / deletar (com confirmação se houver transações vinculadas).

### 8.6 Regras (fase 3)

- Lista com pattern + categoria + priority.
- Criar (modal): pattern (substring) + categoria + priority.
- "Aplicar a transações existentes" (botão): roda todas as regras nas transações **sem categoria** (`category_id IS NULL`). Nunca sobrescreve categorização manual. Quando múltiplas regras casam, vence a de maior `priority`; empate vai pela mais recente.

### 8.7 Configurações

- Mostrar path do DB.
- Botão `Abrir no Finder` (revela o arquivo).
- Botão `Exportar backup` (copia o `.db` pra local escolhido).
- Botão `Importar backup` (substitui DB; aviso destrutivo).
- About: versão, link pro README.

## 9. Fases de build

| Fase | Entregável | Critério de aceitação | Duração estimada |
|---|---|---|---|
| **0 — Scaffold** | Tauri 2 + Svelte 5 + Vite + Tailwind 4 + shadcn-svelte instalados. rusqlite + migrations 0001 rodando. Sidebar + 3 rotas vazias (`/onboarding`, `/dashboard`, `/import`). `tauri-specta` gerando bindings. | `pnpm tauri dev` abre janela, sidebar funciona, DB cria arquivo em `~/Library/Application Support/finan/`. | 1 dia |
| **1 — Import** | Onboarding + drop OFX + parsing + persistência. Lista crua de transações sem categoria. | Importar extrato real do Itaú/Nubank, ver linhas. Reimport não duplica. | 1–2 dias |
| **2 — Categorização** | Inline category picker + filtro por mês + filtro por categoria. Right pane com notes. | Categorizar 30 transações em <2 min. Filtros mantêm estado em store. | 1 dia |
| **3 — Regras** | Migration 0002. CRUD de regras. Aplicação automática no import + comando "aplicar em existentes". | Criar regra "uber → Transporte" e ver categorias auto-atribuídas no próximo import. | 1 dia |
| **4 — Dashboard** | KPIs, donut, barras, top categorias, recent. Seletor de mês. | Trocar de mês atualiza todos os widgets. Donut bate com soma do mês. | 1–2 dias |
| **5 — Polish** | Search global, settings (path, export/backup), atalhos (`⌘O`, `⌘F`, `⌘1..5`), refinamento visual. | Atalhos funcionam, backup/restore round-trip. | 1 dia |

**Total estimado:** 6–9 dias.

## 10. Estratégia de testes

| Camada | Ferramenta | O que testar |
|---|---|---|
| Parser OFX (TS) | Vitest | Itaú, Nubank, Bradesco; encoding ISO-8859-1; ausência de FITID; dedupe key correta. Fixtures anonimizadas em `src/lib/ofx/__fixtures__/`. |
| Money math (Rust) | `cargo test` | Soma, agregação por categoria, arredondamento, parse de string. |
| Categorization rules (Rust, fase 3) | `cargo test` | Priority order, case-insensitivity, múltiplos matches. |
| Commands (Rust) | `cargo test` com SQLite `:memory:` | CRUD básico, dedupe no insert_batch, integridade referencial, filtros de listagem. |
| Migrations (Rust) | `cargo test` | Aplicar do zero, idempotência. |
| E2E | **Skip no MVP.** | Adicionar Playwright só se aparecer bug recorrente de fluxo. |

Lint/typecheck obrigatórios antes de commit: `cargo clippy -- -D warnings` + `cargo fmt --check` + `pnpm tsc --noEmit` + `pnpm svelte-check`.

## 11. Princípios de código

Alinhado com [CLAUDE.md](../../../CLAUDE.md):

1. **Sem abstrações pré-MVP.** ORM-like só se aparecer 3+ queries similares. Cada query é função tipada concreta.
2. **Sem error handling pra casos impossíveis.** Erros de boundary (file IO, SQL, parse) usam `thiserror::Error` no Rust e `Result<T, AppError>` no contrato dos commands.
3. **Sem comentários explicando o quê.** Só o porquê quando não-óbvio (e quando aparecer, vira parágrafo de uma linha no máximo).
4. **Componentes Svelte com responsabilidade única.** Lógica em `.svelte.ts`; `.svelte` só renderiza.
5. **Dinheiro nunca toca f64.** Garantido por tipos no Rust e contrato `string` no IPC.
6. **Nada de comunicação de rede em produção.** `tauri.conf.json` define CSP de produção sem origens externas (`connect-src 'self'`). Em dev, Vite HMR é permitido localmente. Em runtime do app empacotado, qualquer fetch externo deve falhar — auditável abrindo Web Inspector.

## 12. Out of scope (pós-MVP)

- Budgets / metas.
- Multi-moeda.
- Sync entre Macs (mesmo via iCloud).
- Mobile / web.
- AI / sugestões automáticas além de regras.
- Import CSV.
- Relatórios PDF.
- Tags livres (além de category).
- Detecção automática de transferências entre contas.

## 13. Riscos conhecidos

| Risco | Mitigação |
|---|---|
| OFX brasileiro tem dialetos por banco | Camada de normalização em `lib/ofx/normalize.ts`; testar com extratos reais de Itaú, Nubank, Bradesco na fase 1. |
| `tauri-specta` quebrar com Tauri 2.x | Plano B: tipos TS escritos à mão (custo: ~1h de retrabalho na fase 0). |
| LayerChart instabilidade (pre-release de mai/2026) | Plano B documentado: Chart.js + svelte-chartjs. Donut tem fallback CSS conic-gradient. |
| Performance ao importar OFX de muitos meses | Insert batch no Rust com transaction. Não esperado virar gargalo em <10k linhas. |
| macOS bloquear app não-assinado | Documentar `xattr -d com.apple.quarantine` no README. Notarization é pós-MVP. |

## 14. Próximos passos

1. Spec aprovada pelo usuário.
2. Invocar `superpowers:writing-plans` pra gerar plano de implementação fase 0 (scaffold), com checkpoints por etapa.
3. Implementação por fase, com revisão entre fases.

---

**Referências:**
- [README.md](../../../README.md) — princípios e MVP.
- [TECH.md](../../../TECH.md) — validação detalhada da stack.
- [CLAUDE.md](../../../CLAUDE.md) — princípios de execução pra IA assistente.
- `examples/assets/app.css` — fonte canônica da palette (referência, não reutilizado).
- `examples/{onboarding,dashboard,import}.html` — referência de layout (referência, não reutilizado).
