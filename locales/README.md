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
