const FORMATTER = new Intl.NumberFormat("pt-BR", {
  style: "currency",
  currency: "BRL",
});

export function formatMoney(amount: string): string {
  const n = Number(amount);
  if (!Number.isFinite(n)) {
    throw new Error(`invalid amount: ${amount}`);
  }
  return FORMATTER.format(n);
}
