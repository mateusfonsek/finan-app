import { commands } from "../bindings";

function unwrap<T>(result: { status: "ok"; data: T } | { status: "error"; error: string }): T {
  if (result.status === "error") throw new Error(result.error);
  return result.data;
}

/** Records that a bill occurrence is settled. `transactionId` is optional on
 *  purpose: a bill paid in cash, or by someone else, has no transaction and
 *  still counts as paid. */
export async function settleBill(
  ruleId: number,
  dueMonth: string,
  transactionId: number | null,
): Promise<void> {
  unwrap(await commands.settleBill(ruleId, dueMonth, transactionId));
}

export async function unsettleBill(ruleId: number, dueMonth: string): Promise<void> {
  unwrap(await commands.unsettleBill(ruleId, dueMonth));
}
