import { OfxReadError } from "./errors";
import { decodeOfxFile } from "./normalize";
import { parseOfx } from "./parse";
import { readFileBytes } from "$lib/api/files";
import type { ParsedOfx } from "./types";

/**
 * Lê um arquivo .ofx do disco (por caminho) e devolve `{ file, parsed }`,
 * reusando o mesmo decode/parse do file picker. Usado pelo "Abrir com finan".
 *
 * Falha em duas etapas bem diferentes, e o chamador precisa saber em qual:
 * a leitura em disco (`OfxReadError` — transitório, tenta de novo depois) e o
 * parse (erro comum — o conteúdo não é OFX, e isso não melhora com o tempo).
 * O decode acontece sobre bytes já em memória, então conta como parse.
 */
export async function loadOfxFromPath(path: string): Promise<{ file: File; parsed: ParsedOfx }> {
  let bytes: Uint8Array;
  try {
    bytes = await readFileBytes(path);
  } catch (e) {
    throw new OfxReadError(path, e);
  }
  const name = path.split(/[\\/]/).pop() || "extrato.ofx";
  const file = new File([bytes as BlobPart], name);
  const content = await decodeOfxFile(file);
  return { file, parsed: parseOfx(content) };
}
