import { commands } from "../bindings";
import type { CategorySpend, KpiSummary, MonthSummary } from "../bindings";

function unwrap<T>(result: { status: "ok"; data: T } | { status: "error"; error: string }): T {
  if (result.status === "error") throw new Error(result.error);
  return result.data;
}

export async function summaryKpis(month: string | null = null): Promise<KpiSummary> {
  return unwrap(await commands.summaryKpis(month));
}

export async function summaryByCategory(month: string | null = null): Promise<CategorySpend[]> {
  return unwrap(await commands.summaryByCategory(month));
}

export async function summaryByMonth(monthsBack: number): Promise<MonthSummary[]> {
  return unwrap(await commands.summaryByMonth(monthsBack));
}
