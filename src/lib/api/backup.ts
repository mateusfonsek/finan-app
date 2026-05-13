import { commands } from "../bindings";

function unwrap<T>(result: { status: "ok"; data: T } | { status: "error"; error: string }): T {
  if (result.status === "error") throw new Error(result.error);
  return result.data;
}

export async function dbPath(): Promise<string> {
  return unwrap(await commands.dbPath());
}

export async function exportBackup(destination: string): Promise<string> {
  return unwrap(await commands.exportBackup(destination));
}

export async function restoreBackup(source: string): Promise<void> {
  const r = await commands.restoreBackup(source);
  if (r.status === "error") throw new Error(r.error);
}
