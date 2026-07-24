import { locale } from "$lib/i18n/locale.svelte";

const cache = new Map<string, Intl.NumberFormat>();

function formatter(): Intl.NumberFormat {
  const key = `${locale.currencyLocale}:${locale.currencyCode}`;
  let fmt = cache.get(key);
  if (!fmt) {
    fmt = new Intl.NumberFormat(locale.currencyLocale, {
      style: "currency",
      currency: locale.currencyCode,
    });
    cache.set(key, fmt);
  }
  return fmt;
}

export function formatMoney(amount: string): string {
  const n = Number(amount);
  if (!Number.isFinite(n)) {
    throw new Error(`invalid amount: ${amount}`);
  }
  return formatter().format(n);
}
