import { push } from "svelte-spa-router";
import { loadOfxFromPath } from "./load";
import type { ParsedOfx } from "./types";

/** O que o Import lê ao montar. `watchHash` só existe quando veio da pasta
 *  observada — é o que permite marcar o arquivo como importado no fim. */
export type PendingImport = { file: File; parsed: ParsedOfx; watchHash?: string };

type StashWindow = Window & { __finanPending?: PendingImport };

export function stashPending(pending: PendingImport): void {
  (window as unknown as StashWindow).__finanPending = pending;
}

export function takeStashed(): PendingImport | undefined {
  const w = window as unknown as StashWindow;
  const stash = w.__finanPending;
  w.__finanPending = undefined;
  return stash;
}

/** Carrega um .ofx por caminho, guarda no stash e navega pro Import. Usado
 *  tanto pelo "Abrir com finan" quanto pela pasta observada. */
export async function openOfxPath(path: string, watchHash?: string): Promise<void> {
  const loaded = await loadOfxFromPath(path);
  stashPending({ ...loaded, watchHash });
  push("/import");
}
