import { commands } from "../bindings";
import type {
  CategorySpend,
  IncomeSource,
  InvestmentSummary,
  KpiSummary,
  MonthSummary,
  TransferSummary,
} from "../bindings";

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

export async function investmentSummary(month: string | null = null): Promise<InvestmentSummary> {
  return unwrap(await commands.investmentSummary(month));
}

export async function transferSummary(month: string | null = null): Promise<TransferSummary> {
  return unwrap(await commands.transferSummary(month));
}

export async function incomeSources(month: string | null = null): Promise<IncomeSource[]> {
  return unwrap(await commands.incomeSources(month));
}
