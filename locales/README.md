# Locale packs

Each folder here is a **locale pack**: everything that is specific to a country /
language lives in JSON, so you can add a new language **without touching any Rust or
Svelte code**. The app auto-discovers every subfolder — no registry to edit.

```
locales/
  pt-BR/            ← locale code (language-REGION), e.g. pt-BR, en-US, pt-PT
    manifest.json   ← currency, date locale, tax-id (regex + lookup provider)
    categories.json ← default categories (with a stable, language-independent key)
    rules.json      ← auto-classification: tax-id → category map, seed rules, description normalization
    strings.json    ← all UI text + native menu labels
```

## How to add your language

1. **Copy** `pt-BR/` to a new folder named with your BCP-47 code, e.g. `en-US/`.
   Use **language-REGION**, not just language — the classification rules are
   country-specific (a Brazilian CNPJ ≠ a UK company number).
2. **`manifest.json`** — set `code`, `name`, `flag`, `currency` (ISO code +
   `Intl` locale), `dateLocale`, and `taxId`:
   - `regex`: how a business tax id appears in a bank statement description
     (Brazil: CNPJ `NN.NNN.NNN/NNNN-NN`). Leave `regex` empty to disable tax-id
     enrichment for your locale.
   - `provider`: which company-lookup service to call. Today only `brasilapi`
     is implemented; any other value simply skips the online lookup (the app
     still works, it just won't auto-name companies).
3. **`categories.json`** — translate `name` for each category. **Keep the `key`
   values identical** to `pt-BR` (`market`, `restaurant`, …). The keys are what
   the rules reference, so classification keeps working in any language.
4. **`rules.json`**:
   - `cnae_map`: tax-classification-code prefix → category `key`. This is
     Brazil-CNAE-specific; adapt to your country's scheme (or leave empty).
   - `seed_rules`: merchant substring → category `key`. Add the merchants common
     in your country (`UBER`, `NETFLIX`, local delivery apps, …).
   - `normalization`: how your banks phrase statement lines (Pix, debit, boleto…).
     Each entry maps a description prefix to a readable label. Leave the list
     empty if you don't need it — descriptions fall back to their raw text.
   - `reversals`: how your banks phrase a reversal/refund pair. Each phase picks
     a pairing `strategy` — `exact_remainder`, `counterparty_signature` or
     `quoted_merchant` — and supplies the prefix, the window in days and the two
     roles (`reversal`/`reversed`, `refund`/`refunded`). Array order is
     execution order, and an earlier phase wins a contested transaction. Leave
     `phases` empty to disable reversal detection for your locale.
5. **`strings.json`** — translate every value. Keep the **keys** unchanged.
   `{v}`, `{name}` etc. are interpolation placeholders — keep them.
6. Rebuild the app. Your language appears in **Settings → Idioma** automatically.

## Notes

- The **stable `key`** on a category is the contract between data and display.
  Never rename a key; only translate its `name`.
- Switching language re-localizes the UI immediately; the **native macOS menu**
  updates on the next app launch.
- Existing categories in a user's database are their own data and are **not**
  renamed when the language changes — only fresh databases are seeded from the
  active pack.
- **Graceful degradation is the design, not a gap.** Nothing forces you to fill
  in every field. The **`en-US`** pack ships with empty `taxId.regex`,
  `taxId.provider`, `cnae_map` and `reversals.phases`, because US statements
  carry no EIN, there is no free public company-lookup equivalent to BrasilAPI,
  and US reversal phrasing couldn't be verified. The app degrades cleanly: no
  tax-id enrichment, no CNAE-based suggestions, no reversal detection — the
  rest of the pack (categories, seed rules, strings) works exactly as with any
  other locale. Ship what you can verify; leave the rest empty.

## Checking your pack

```sh
cargo test --manifest-path src-tauri/Cargo.toml contract::
```

Validates every folder here against `pt-BR`: identical category keys, colour
tokens that exist in `src/app.css`, rules pointing at declared categories, an
identical set of string keys, and matching `{placeholders}`. Every one of these
fails silently at runtime, which is why the test exists — run it before opening
a PR.
