import { describe, it, expect, vi, beforeEach } from "vitest";

const readFileBytes = vi.hoisted(() => vi.fn());
vi.mock("$lib/api/files", () => ({ readFileBytes }));

import { OfxReadError } from "./errors";
import { loadOfxFromPath } from "./load";

describe("loadOfxFromPath", () => {
  beforeEach(() => vi.clearAllMocks());

  it("classifica falha de disco como OfxReadError", async () => {
    readFileBytes.mockRejectedValue(new Error("No such file or directory (os error 2)"));

    await expect(loadOfxFromPath("/tmp/sumiu.ofx")).rejects.toBeInstanceOf(OfxReadError);
  });

  it("falha de conteúdo NÃO é OfxReadError", async () => {
    // Quem decide marcar o arquivo como `invalid` (permanente) usa exatamente
    // essa distinção — se o parse virasse OfxReadError, lixo ficaria pendente
    // pra sempre; se a leitura virasse erro comum, extrato bom seria enterrado.
    readFileBytes.mockResolvedValue(new TextEncoder().encode("<OFX>"));

    await expect(loadOfxFromPath("/tmp/lixo.ofx")).rejects.toThrow();
    await expect(loadOfxFromPath("/tmp/lixo.ofx")).rejects.not.toBeInstanceOf(OfxReadError);
  });
});
