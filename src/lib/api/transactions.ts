import { commands } from "../bindings";
import type {
  InsertResult,
  NewTransaction,
  Transaction,
  TransactionFilters,
} from "../bindings";

function unwrap<T>(result: { status: "ok"; data: T } | { status: "error"; error: string }): T {
  if (result.status === "error") throw new Error(result.error);
  return result.data;
}

export async function listTransactions(
  filters: TransactionFilters | null = null,
): Promise<Transaction[]> {
  return unwrap(await commands.listTransactions(filters));
}

export async function insertTransactions(
  accountId: number,
  txs: NewTransaction[],
): Promise<InsertResult> {
  return unwrap(await commands.insertTransactions(accountId, txs));
}

export async function checkExistingFitids(
  accountId: number,
  fitids: string[],
): Promise<string[]> {
  return unwrap(await commands.checkExistingFitids(accountId, fitids));
}

export async function updateTransactionCategory(
  transactionId: number,
  categoryId: number | null,
): Promise<void> {
  unwrap(await commands.updateTransactionCategory(transactionId, categoryId));
}

export async function updateTransactionNotes(
  transactionId: number,
  notes: string | null,
): Promise<void> {
  unwrap(await commands.updateTransactionNotes(transactionId, notes));
}
