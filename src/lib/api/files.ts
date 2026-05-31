import { commands } from "../bindings";

function unwrap<T>(result: { status: "ok"; data: T } | { status: "error"; error: string }): T {
  if (result.status === "error") throw new Error(result.error);
  return result.data;
}

/** Lê os bytes crus de um arquivo do disco (usado pelo drag-and-drop de OFX). */
export async function readFileBytes(path: string): Promise<Uint8Array> {
  return new Uint8Array(unwrap(await commands.readFileBytes(path)));
}

/** Drena os caminhos de .ofx abertos via Finder ("Abrir com finan"). */
export async function takePendingOfx(): Promise<string[]> {
  return commands.takePendingOfx();
}
