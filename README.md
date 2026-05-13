# finan-app

App de organização financeira pessoal pra macOS.

## Sobre o projeto

A ideia é ter um app simples e rápido pra acompanhar minhas finanças sem depender de serviço em nuvem nem pagar nada. Tudo roda local no meu Mac — os dados ficam num arquivo SQLite na minha própria máquina, sem login, sem conta, sem internet.

O fluxo é direto: exporto o extrato mensal do meu banco (formato OFX), arrasto pra dentro do app, e ele importa as transações pro banco local. A partir daí dá pra categorizar os gastos, filtrar por mês, e ver onde o dinheiro está indo.

### Princípios

- **100% local** — nenhum dado sai do meu Mac
- **100% gratuito** — sem assinaturas, sem APIs pagas
- **Leve** — binário pequeno, abre rápido, consome pouca memória
- **Clean** — interface minimalista, só o que importa

### O que o app faz (MVP)

- Importa extratos OFX de qualquer banco (drag-and-drop)
- Guarda as transações num SQLite local
- Categoriza gastos (manual e por regras automáticas)
- Mostra dashboard com gastos por categoria e por mês
- Filtra e busca transações

## Status

- ✅ Fase 0 — Scaffold (Tauri + Svelte + DB + sidebar + IPC tipado)
- ✅ Fase 1 — Importar OFX (parser TS + dedup por FITID + listagem)
- ✅ Fase 2 — Categorização manual inline + filtros + notes
- 🚧 Fase 3 — Regras automáticas (próximo)
- ⏳ Fase 4-5 — Dashboard, polish

## Stack

- **[Tauri 2](https://v2.tauri.app/)** — framework pra app desktop nativo
- **[Svelte 5](https://svelte.dev/)** + **[Vite](https://vite.dev/)** — frontend
- **[shadcn-svelte](https://www.shadcn-svelte.com/)** — componentes de UI
- **[SQLite](https://www.sqlite.org/)** (via `@tauri-apps/plugin-sql`) — banco de dados local
- **[Drizzle ORM](https://orm.drizzle.team/)** — type-safety nas queries
- **[LayerChart](https://layerchart.com/)** — gráficos
