import { invoke } from "@tauri-apps/api/core";

// -------------------------------------------------------------------------
// Auto-discovery of locale packs.
//
// Every folder under /locales/<code>/ is picked up at build time via glob —
// adding a new language is "drop a folder + translate JSON", no code changes.
// -------------------------------------------------------------------------

export type Manifest = {
  code: string;
  name: string;
  flag: string;
  currency: { code: string; locale: string };
  dateLocale: string;
  taxId: { name: string; regex: string; provider: string };
};

type Strings = Record<string, unknown>;
type Pack = { code: string; manifest: Manifest; strings: Strings };

const manifestMods = import.meta.glob("/locales/*/manifest.json", { eager: true });
const stringsMods = import.meta.glob("/locales/*/strings.json", { eager: true });

function codeFromPath(path: string): string {
  // "/locales/pt-BR/manifest.json" -> "pt-BR"
  return path.split("/")[2];
}

function pick<T>(mod: unknown): T {
  return ((mod as { default?: T }).default ?? (mod as T)) as T;
}

const packs: Record<string, Pack> = {};
for (const [path, mod] of Object.entries(manifestMods)) {
  const code = codeFromPath(path);
  packs[code] = { code, manifest: pick<Manifest>(mod), strings: packs[code]?.strings ?? {} };
}
for (const [path, mod] of Object.entries(stringsMods)) {
  const code = codeFromPath(path);
  if (packs[code]) packs[code].strings = pick<Strings>(mod);
}

const DEFAULT_LOCALE = "pt-BR";
const STORAGE_KEY = "locale";

function storageGet(): string | null {
  try {
    if (typeof localStorage !== "undefined" && typeof localStorage.getItem === "function") {
      return localStorage.getItem(STORAGE_KEY);
    }
  } catch {
    // ignore — no/blocked storage
  }
  return null;
}

function storageSet(code: string): void {
  try {
    if (typeof localStorage !== "undefined" && typeof localStorage.setItem === "function") {
      localStorage.setItem(STORAGE_KEY, code);
    }
  } catch {
    // ignore — no/blocked storage
  }
}

function pickInitial(): string {
  const saved = storageGet();
  if (saved && packs[saved]) return saved;
  if (packs[DEFAULT_LOCALE]) return DEFAULT_LOCALE;
  return Object.keys(packs)[0] ?? DEFAULT_LOCALE;
}

function lookup(obj: unknown, path: string): unknown {
  return path.split(".").reduce<unknown>((o, k) => {
    if (o && typeof o === "object") return (o as Record<string, unknown>)[k];
    return undefined;
  }, obj);
}

function createLocale() {
  let code = $state(pickInitial());

  const stringsOf = (c: string): Strings => packs[c]?.strings ?? {};

  /** Translate a dot-path key with optional `{var}` interpolation.
   *  Falls back to the default locale, then to the key itself. */
  function t(key: string, params?: Record<string, string | number>): string {
    let val = lookup(stringsOf(code), key);
    if (typeof val !== "string" && code !== DEFAULT_LOCALE) {
      val = lookup(stringsOf(DEFAULT_LOCALE), key);
    }
    if (typeof val !== "string") return key;
    if (params) {
      for (const [k, v] of Object.entries(params)) {
        val = (val as string).replaceAll(`{${k}}`, String(v));
      }
    }
    return val as string;
  }

  return {
    get code() {
      return code;
    },
    get manifest(): Manifest | undefined {
      return packs[code]?.manifest;
    },
    get currencyLocale(): string {
      return packs[code]?.manifest.currency.locale ?? DEFAULT_LOCALE;
    },
    get currencyCode(): string {
      return packs[code]?.manifest.currency.code ?? "BRL";
    },
    get dateLocale(): string {
      return packs[code]?.manifest.dateLocale ?? DEFAULT_LOCALE;
    },
    get months(): string[] {
      return (lookup(stringsOf(code), "months") as string[]) ?? [];
    },
    get monthsShort(): string[] {
      return (lookup(stringsOf(code), "months_short") as string[]) ?? [];
    },
    get weekdaysShort(): string[] {
      return (lookup(stringsOf(code), "weekdays_short") as string[]) ?? [];
    },
    /** Raw (non-string) value at a dot-path — for arrays/objects like spec lists.
     *  Falls back to the default locale. */
    raw<T>(key: string): T | undefined {
      let val = lookup(stringsOf(code), key);
      if (val === undefined && code !== DEFAULT_LOCALE) {
        val = lookup(stringsOf(DEFAULT_LOCALE), key);
      }
      return val as T | undefined;
    },
    /** All available locales, for a language picker. */
    list(): Manifest[] {
      return Object.values(packs).map((p) => p.manifest);
    },
    t,
    /** Change the active locale: updates the UI now and persists (so the native
     *  menu picks it up on next launch). */
    async set(next: string): Promise<void> {
      if (!packs[next]) return;
      code = next;
      storageSet(next);
      try {
        await invoke("set_active_locale", { code: next });
      } catch {
        // backend not available (e.g. web preview) — localStorage still holds it
      }
    },
    /** Sync from the backend's persisted choice on boot. */
    async init(): Promise<void> {
      try {
        const active = await invoke<string>("get_active_locale");
        if (active && packs[active]) {
          code = active;
          storageSet(active);
        }
      } catch {
        // ignore — fall back to localStorage/default already chosen
      }
    },
  };
}

export const locale = createLocale();

/** Convenience: bare `t()` for terse imports. */
export const t = locale.t;
