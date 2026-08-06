import { Channel } from "@tauri-apps/api/core";
import { commands } from "../bindings";
import type { EnrichEvent } from "../bindings";

function unwrap<T>(result: { status: "ok"; data: T } | { status: "error"; error: string }): T {
  if (result.status === "error") throw new Error(result.error);
  return result.data;
}

/**
 * Dispara o enriquecimento em segundo plano. Resolve assim que a thread começa
 * — não quando ela termina. O progresso chega por `onEvent`.
 *
 * O canal é criado por quem chama e precisa viver enquanto o trabalho durar:
 * por isso o dono é o store, nunca um componente.
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
