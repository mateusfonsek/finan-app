import { commands } from "../bindings";
import type { Account, NewAccount } from "../bindings";

function unwrap<T>(result: { status: "ok"; data: T } | { status: "error"; error: string }): T {
  if (result.status === "error") throw new Error(result.error);
  return result.data;
}

export async function listAccounts(): Promise<Account[]> {
  return unwrap(await commands.listAccounts());
}

export async function createOrGetAccount(input: NewAccount): Promise<Account> {
  return unwrap(await commands.createOrGetAccount(input));
}
