import { commands } from "../bindings";
import type { Category, NewCategory } from "../bindings";

function unwrap<T>(result: { status: "ok"; data: T } | { status: "error"; error: string }): T {
  if (result.status === "error") throw new Error(result.error);
  return result.data;
}

export async function listCategories(): Promise<Category[]> {
  return unwrap(await commands.listCategories());
}

export async function createCategory(input: NewCategory): Promise<Category> {
  return unwrap(await commands.createCategory(input));
}
