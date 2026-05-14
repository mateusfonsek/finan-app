import { commands } from "../bindings";
import type {
  Category,
  CategoryWithCount,
  NewCategory,
  UpdateCategory,
} from "../bindings";

function unwrap<T>(result: { status: "ok"; data: T } | { status: "error"; error: string }): T {
  if (result.status === "error") throw new Error(result.error);
  return result.data;
}

export async function listCategories(): Promise<Category[]> {
  return unwrap(await commands.listCategories());
}

export async function listCategoriesWithCount(): Promise<CategoryWithCount[]> {
  return unwrap(await commands.listCategoriesWithCount());
}

export async function createCategory(input: NewCategory): Promise<Category> {
  return unwrap(await commands.createCategory(input));
}

export async function updateCategory(
  categoryId: number,
  input: UpdateCategory,
): Promise<Category> {
  return unwrap(await commands.updateCategory(categoryId, input));
}

export async function deleteCategory(categoryId: number): Promise<void> {
  const r = await commands.deleteCategory(categoryId);
  if (r.status === "error") throw new Error(r.error);
}
