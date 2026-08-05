import { commands } from "../bindings";
import type { EnrichmentStatus } from "../bindings";

function unwrap<T>(result: { status: "ok"; data: T } | { status: "error"; error: string }): T {
  if (result.status === "error") throw new Error(result.error);
  return result.data;
}

/**
 * Tax-id enrichment: during import the app can call an external service to
 * identify the company behind a tax id and suggest a category. Off by default.
 *
 * `available` answers "can the active locale do this?" — ask that, never a
 * language code.
 */
export async function enrichmentStatus(): Promise<EnrichmentStatus> {
  return unwrap(await commands.enrichmentStatus());
}

export async function setEnrichmentEnabled(enabled: boolean): Promise<void> {
  unwrap(await commands.setEnrichmentEnabled(enabled));
}
