<div align="center">

<sub><a href="README.pt-BR.md">🇧🇷 Leia em português</a></sub>

<img src="src-tauri/icons/icon.png" width="116" alt="finan app" />

# finan app

**Your money. Your data too.**

100% local personal finance for your Mac — no cloud, no account, no tracking.

<br>

<div>
<a href="https://github.com/MateusFonseK/finan-app/releases/latest"><img src="https://img.shields.io/badge/⬇%20Download-.dmg-3fa34d?logo=apple&logoColor=white" alt="Download para macOS" /></a>
<img src="https://img.shields.io/badge/macOS-Apple%20Silicon%20%2B%20Intel-111111?logo=apple&logoColor=white" alt="macOS" />
<img src="https://img.shields.io/badge/tamanho-~13%20MB-3fa34d" alt="Tamanho ~13 MB" />
<img src="https://img.shields.io/badge/licença-MIT-3fa34d" alt="Licença MIT" />
<img src="https://img.shields.io/badge/feito%20com-Tauri%20%2B%20Svelte-ff3e00" alt="Tauri + Svelte" />
</div>

<br>

<a href="#-installation">Installation</a> ·
<a href="#-features">Features</a> ·
<a href="#-privacy">Privacy</a> ·
<a href="#%EF%B8%8F-building-from-source">Build</a>

</div>

<br>

**finan app** organizes your personal finances without your data ever leaving your computer. You import your bank's `.ofx` statement, it categorizes with rules, suggests patterns and shows everything on a clear dashboard. Everything lives in a single file on your Mac and never goes anywhere else.

And it's **light**: ~13 MB that download in an instant, open fast and barely take up space on your Mac.

<br>

![Dashboard](docs/screenshots/dashboard.png)

<table>
  <tr>
    <td width="33%" valign="top"><img src="docs/screenshots/transacoes.png" alt="Transactions" /><p align="center"><sub><b>Transactions</b> — categorize and create rules</sub></p></td>
    <td width="33%" valign="top"><img src="docs/screenshots/importar.png" alt="Import OFX" /><p align="center"><sub><b>Import</b> — drop the OFX and review</sub></p></td>
    <td width="33%" valign="top"><img src="docs/screenshots/calendario.png" alt="Calendar" /><p align="center"><sub><b>Calendar</b> — due dates and payments</sub></p></td>
  </tr>
</table>

## ✨ Features

- 📥 **Import OFX** — drag in the statement (or open it with finan app) and review before saving; deduplicates transactions and detects reversals automatically.
- 🏷️ **Categories and rules** — categorize manually or create rules (matching a piece of the description) that apply themselves.
- 💡 **Automatic suggestions** — the app detects recurring uncategorized spending and suggests ready-made rules.
- 📊 **Dashboard** — the month's income, spending and balance, spending by category, income sources (with recurring ones flagged), investments and the trend over the last 12 months.
- 📅 **Calendar** — due dates and payments derived from your rules.
- 💾 **Backup** — export and restore your database at any time.
- 🌍 **Multi-language** — the UI, the default categories and the auto-classification rules all live in [locale packs](locales/README.md). Adding a language is copying a folder and translating JSON. Ships with Portuguese (Brazil) and English (US).
- 🪶 **Light and fast** — ~13 MB to download, ~18 MB installed. Opens in a blink and barely weighs on your Mac.
- 🖥️ **Native to macOS** — native menu, keyboard shortcuts, light/dark theme, universal (Apple Silicon + Intel).

## 📥 Installation

1. Download the latest `.dmg` from the **[Releases](https://github.com/MateusFonseK/finan-app/releases/latest)** tab.
2. Open the `.dmg` and drag **finan app** into the **Applications** folder.
3. **First launch** — the app is **not signed** with a paid Apple account. Because it was downloaded from the web, the first time macOS blocks it with a warning like *"Apple could not verify 'finan app' is free of malware…"* (buttons **Move to Trash** and **OK** — click **OK**, not the trash).

   **Guaranteed way** — run in Terminal:
   ```sh
   xattr -dr com.apple.quarantine "/Applications/finan app.app"
   ```
   Then just open it normally.

   **Alternative without the terminal:** **System Settings → Privacy & Security** → **Security** section → **"Open Anyway"**. (The button only appears right after you try to open the app.)

> The browser marks downloaded files with a "quarantine" flag; without Apple notarization, Gatekeeper asks for this manual confirmation the first time. The code is open — you can audit it and/or build it yourself.

## 🔒 Privacy

There is no account, login, telemetry or ads. Everything lives in `~/Library/Application Support/app.finan/finan.db`, on your Mac.

The **only** network request the app can ever make happens **during import**, and only when two things are both true: the active [locale pack](locales/README.md) declares a tax-id format and a lookup provider, **and** you've turned the lookup on in Settings — it's opt-in and off by default. Today that's the **pt-BR** pack, which queries [BrasilAPI](https://brasilapi.com.br) to resolve a company name from the **CNPJ** found in a transaction description and suggest a category. Only the **CNPJ digits** (public information) ever leave your Mac — never amounts, descriptions or personal data. The **en-US** pack ships with no tax-id format and no provider at all, so with it active the app makes **no network request, ever**.

## 🛠️ Building from source

Prerequisites: [Rust](https://rustup.rs), [Node 22+](https://nodejs.org) and [pnpm](https://pnpm.io).

```sh
pnpm install

# development
pnpm tauri dev

# universal build (Apple Silicon + Intel)
rustup target add aarch64-apple-darwin x86_64-apple-darwin
pnpm tauri build --target universal-apple-darwin
```

The `.app` and the `.dmg` come out at `src-tauri/target/universal-apple-darwin/release/bundle/`.

To contribute (commit format and how releases work), see [CONTRIBUTING.md](CONTRIBUTING.md).

## 🧱 Stack

[Tauri 2](https://tauri.app) (Rust) · [Svelte 5](https://svelte.dev) · SQLite (rusqlite). No backend, no telemetry.

## 📄 License

[MIT](LICENSE) © 2026 Mateus Fonseca

<div align="center">
<br>
<sub>Made with care. 🌱</sub>
</div>
