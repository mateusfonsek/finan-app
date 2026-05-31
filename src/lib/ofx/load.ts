import { decodeOfxFile } from "./normalize";
import { parseOfx } from "./parse";
import { readFileBytes } from "$lib/api/files";
import type { ParsedOfx } from "./types";

/**
 * Lê um arquivo .ofx do disco (por caminho) e devolve `{ file, parsed }`,
 * reusando o mesmo decode/parse do file picker. Usado pelo "Abrir com finan".
 */
export async function loadOfxFromPath(path: string): Promise<{ file: File; parsed: ParsedOfx }> {
  const bytes = await readFileBytes(path);
  const name = path.split(/[\\/]/).pop() || "extrato.ofx";
  const file = new File([bytes as BlobPart], name);
  const content = await decodeOfxFile(file);
  return { file, parsed: parseOfx(content) };
}
