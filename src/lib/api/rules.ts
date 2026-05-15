import { commands } from "../bindings";
import type { CalendarEvent, NewRule, Rule, UpdateRule } from "../bindings";

function unwrap<T>(result: { status: "ok"; data: T } | { status: "error"; error: string }): T {
  if (result.status === "error") throw new Error(result.error);
  return result.data;
}

export async function listRules(): Promise<Rule[]> {
  return unwrap(await commands.listRules());
}

export async function createRule(input: NewRule): Promise<Rule> {
  return unwrap(await commands.createRule(input));
}

export async function updateRule(ruleId: number, input: UpdateRule): Promise<Rule> {
  return unwrap(await commands.updateRule(ruleId, input));
}

export async function deleteRule(ruleId: number): Promise<void> {
  const r = await commands.deleteRule(ruleId);
  if (r.status === "error") throw new Error(r.error);
}

export async function applyRulesToUncategorized(
  accountId: number | null = null,
): Promise<number> {
  return unwrap(await commands.applyRulesToUncategorized(accountId));
}

export async function calendarEvents(month: string): Promise<CalendarEvent[]> {
  return unwrap(await commands.calendarEvents(month));
}
