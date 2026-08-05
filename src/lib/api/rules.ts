import { commands } from "../bindings";
import type {
  CalendarEvent,
  NewRule,
  Rule,
  RuleChoice,
  RuleMatches,
  RulePreviewRow,
  RuleWithCount,
  UpdateRule,
} from "../bindings";

function unwrap<T>(result: { status: "ok"; data: T } | { status: "error"; error: string }): T {
  if (result.status === "error") throw new Error(result.error);
  return result.data;
}

export async function listRules(): Promise<Rule[]> {
  return unwrap(await commands.listRules());
}

/** Rules plus how many transactions each reaches. Only the Rules screen needs
 *  the count, which scans transactions. */
export async function listRulesWithCount(): Promise<RuleWithCount[]> {
  return unwrap(await commands.listRulesWithCount());
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

export async function deleteRuleWithCleanup(ruleId: number): Promise<number> {
  return unwrap(await commands.deleteRuleWithCleanup(ruleId));
}

export async function applyRulesToUncategorized(
  accountId: number | null = null,
): Promise<number> {
  return unwrap(await commands.applyRulesToUncategorized(accountId));
}

/** The transactions a rule reaches, with the total. Same criterion as the
 *  count shown in the rules table. */
export async function transactionsMatchingRule(ruleId: number): Promise<RuleMatches> {
  return unwrap(await commands.transactionsMatchingRule(ruleId));
}

/** Everything applying the rules would change, writing nothing. */
export async function previewRuleApplication(
  accountId: number | null = null,
): Promise<RulePreviewRow[]> {
  return unwrap(await commands.previewRuleApplication(accountId));
}

/** Writes only what the user ticked in the review. */
export async function applyRuleChoices(choices: RuleChoice[]): Promise<number> {
  return unwrap(await commands.applyRuleChoices(choices));
}

export async function calendarEvents(month: string): Promise<CalendarEvent[]> {
  return unwrap(await commands.calendarEvents(month));
}
