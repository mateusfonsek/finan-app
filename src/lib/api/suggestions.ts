import { commands } from "../bindings";
import type { CnpjResolution, RuleSuggestion } from "../bindings";

function unwrap<T>(result: { status: "ok"; data: T } | { status: "error"; error: string }): T {
  if (result.status === "error") throw new Error(result.error);
  return result.data;
}

export async function suggestRules(minCount: number): Promise<RuleSuggestion[]> {
  return unwrap(await commands.suggestRules(minCount));
}

export async function suggestPatternFor(description: string): Promise<string> {
  return commands.suggestPatternFor(description);
}

export async function resolveCnpj(cnpj: string): Promise<CnpjResolution> {
  return unwrap(await commands.resolveCnpj(cnpj));
}
