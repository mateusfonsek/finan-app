import { Channel } from "@tauri-apps/api/core";
import { commands } from "../bindings";
import type { EnrichEvent } from "../bindings";

function unwrap<T>(result: { status: "ok"; data: T } | { status: "error"; error: string }): T {
  if (result.status === "error") throw new Error(result.error);
  return result.data;
}

/**
 * Starts the enrichment in the background. Resolves as soon as the thread
 * begins — not when it finishes. Progress arrives through `onEvent`.
 *
 * The channel is created by the caller and must live as long as the work does:
 * that is why the store owns it, never a component.
 */
export async function startCnpjEnrichment(
  accountId: number | null,
  onEvent: (event: EnrichEvent) => void,
): Promise<void> {
  const channel = new Channel<EnrichEvent>();
  channel.onmessage = onEvent;
  unwrap(await commands.startCnpjEnrichment(accountId, channel));
}

export async function cancelCnpjEnrichment(): Promise<void> {
  await commands.cancelCnpjEnrichment();
}
