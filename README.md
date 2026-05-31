# finan app

**O dinheiro é seu. Os dados também.**

Um app de finanças pessoais **100% local** pro macOS. Importe extratos OFX do seu banco, categorize com regras e acompanhe tudo num só lugar — sem nuvem, sem conta, sem rastreamento. Seus dados ficam num único arquivo no seu Mac e nunca saem dele.

> Só macOS (Apple Silicon e Intel). Software livre, sem garantias.

<!-- Screenshots: adicione imagens em docs/screenshots e referencie aqui.
![Dashboard](docs/screenshots/dashboard.png) -->

## Funcionalidades

- **Importar OFX** — arraste o extrato (ou abra com o finan app) e revise antes de salvar; deduplica transações e detecta estornos.
- **Categorias e regras** — categorize manualmente ou crie regras (por trecho da descrição) que se aplicam sozinhas; sugestões automáticas pra recorrências.
- **Dashboard** — KPIs do mês, gastos por categoria, fontes de renda (com marcação de recorrentes), investimentos e tendência dos últimos 12 meses.
- **Calendário** — vencimentos e pagamentos derivados das suas regras.
- **Backup** — exporte/restaure o banco a qualquer momento.

## Instalação

1. Baixe o `.dmg` mais recente na aba **[Releases](../../releases)**.
2. Abra o `.dmg` e arraste o **finan app** pra pasta **Aplicativos**.
3. **Primeira abertura** — o app **não é assinado** (não tenho conta no Apple Developer Program). Como ele foi **baixado da web**, o macOS pode recusar abri-lo ("danificado" ou "não foi possível verificar o desenvolvedor"). Se isso acontecer, rode uma vez no Terminal:

   ```sh
   xattr -dr com.apple.quarantine "/Applications/finan app.app"
   ```

   Depois é só abrir normalmente. (Alternativa: tente abrir, vá em **Ajustes do Sistema → Privacidade e Segurança** e clique em **"Abrir mesmo assim"**.)

> Por que isso? O navegador marca arquivos baixados com uma flag de "quarentena"; sem notarização da Apple, o Gatekeeper bloqueia. O comando acima remove a flag. **Se você buildar o app você mesmo** (instruções abaixo), isso não se aplica — apps gerados localmente não recebem a quarentena.

## Buildar do código

Pré-requisitos: [Rust](https://rustup.rs), [Node 20+](https://nodejs.org) e [pnpm](https://pnpm.io).

```sh
pnpm install

# rodar em desenvolvimento
pnpm tauri dev

# build local (arquitetura da sua máquina)
pnpm tauri build

# build universal (Apple Silicon + Intel) — recomendado pra distribuir
rustup target add aarch64-apple-darwin x86_64-apple-darwin
pnpm tauri build --target universal-apple-darwin
```

O `.app` e o `.dmg` saem em `src-tauri/target/**/release/bundle/`.

## Stack

[Tauri 2](https://tauri.app) (Rust) · [Svelte 5](https://svelte.dev) · SQLite (rusqlite). Sem backend, sem telemetria.

## Privacidade

Nenhum dado é enviado pra servidores. A única requisição de rede é, opcionalmente, à [BrasilAPI](https://brasilapi.com.br) para resolver o nome de um CNPJ ao categorizar — feita sob demanda e sem enviar seus dados financeiros.

## Licença

[MIT](LICENSE) © 2026 Mateus Fonseca
