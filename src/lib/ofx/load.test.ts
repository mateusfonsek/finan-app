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
    // Whatever decides to mark a file `invalid` (permanent) relies on exactly
    // this distinction: if parse threw OfxReadError, junk would stay pending
    // forever; if a read threw a plain error, a good statement would be buried.
    readFileBytes.mockResolvedValue(new TextEncoder().encode("<OFX>"));

    await expect(loadOfxFromPath("/tmp/lixo.ofx")).rejects.toThrow();
    await expect(loadOfxFromPath("/tmp/lixo.ofx")).rejects.not.toBeInstanceOf(OfxReadError);
  });
});
