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
3. **Primeira abertura** — o app **não é assinado** com conta paga da Apple. Como ele foi **baixado da web**, na primeira vez o macOS bloqueia com um aviso do tipo *"A Apple não pôde verificar se o 'finan app' está livre de malware…"* (só com os botões **Mover para o Lixo** e **OK** — clique em **OK**, não no lixo).

   **Jeito garantido (1 comando no Terminal):**

   ```sh
   xattr -dr com.apple.quarantine "/Applications/finan app.app"
   ```

   Depois é só abrir o app normalmente.

   **Alternativa sem terminal:** abra **Ajustes do Sistema → Privacidade e Segurança**, role até a seção **Segurança** e clique em **"Abrir Mesmo Assim"** ao lado do aviso sobre o "finan app". ⚠️ Esse botão só aparece **logo após** você tentar abrir o app (dê duplo-clique nele primeiro); se não aparecer, tente abrir o app de novo e volte aqui em seguida — ou use o comando acima.

> Por que isso? O navegador marca arquivos baixados com uma flag de "quarentena"; sem notarização da Apple, o Gatekeeper exige essa confirmação manual na primeira abertura. **Se você buildar o app você mesmo** (instruções abaixo), isso não se aplica — apps gerados localmente não recebem a quarentena.

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
