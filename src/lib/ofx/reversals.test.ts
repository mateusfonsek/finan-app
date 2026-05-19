import { describe, it, expect } from "vitest";
import { detectReversalPairs } from "./reversals";
import type { ParsedTransaction } from "./types";

function tx(
  fitid: string,
  date: string,
  amount: string,
  description: string,
): ParsedTransaction {
  return { fitid, date, amount, description };
}

describe("detectReversalPairs — estornos", () => {
  it("matches a single estorno with its original tx (same day, opposite amount)", () => {
    const a = tx("A", "2026-01-20", "-1385.43", "Transferência enviada pelo Pix - Mateus");
    const b = tx("B", "2026-01-20", "1385.43", "Estorno - Transferência enviada pelo Pix - Mateus");
    const map = detectReversalPairs([a, b]);
    expect(map.size).toBe(2);
    expect(map.get("A")).toEqual({ role: "estornada", pairFitid: "B" });
    expect(map.get("B")).toEqual({ role: "estorno", pairFitid: "A" });
  });

  it("pairs N estornos with N of N+1 originals (FIFO), leaving one original unpaired", () => {
    const o1 = tx("O1", "2026-01-20", "-1385.43", "Pix - Mateus");
    const o2 = tx("O2", "2026-01-20", "-1385.43", "Pix - Mateus");
    const o3 = tx("O3", "2026-01-20", "-1385.43", "Pix - Mateus");
    const o4 = tx("O4", "2026-01-20", "-1385.43", "Pix - Mateus");
    const e1 = tx("E1", "2026-01-20", "1385.43", "Estorno - Pix - Mateus");
    const e2 = tx("E2", "2026-01-20", "1385.43", "Estorno - Pix - Mateus");
    const e3 = tx("E3", "2026-01-20", "1385.43", "Estorno - Pix - Mateus");

    const map = detectReversalPairs([o1, o2, o3, o4, e1, e2, e3]);

    expect(map.size).toBe(6);
    expect(map.has("O4")).toBe(false);
    for (const e of ["E1", "E2", "E3"]) {
      expect(map.get(e)?.role).toBe("estorno");
    }
  });

  it("does not pair estorno when descriptions differ", () => {
    const a = tx("A", "2026-01-20", "-100", "Pix - João");
    const b = tx("B", "2026-01-20", "100", "Estorno - Pix - Maria");
    expect(detectReversalPairs([a, b]).size).toBe(0);
  });

  it("does not pair estorno when amounts are not opposite", () => {
    const a = tx("A", "2026-01-20", "-100", "Pix - João");
    const b = tx("B", "2026-01-20", "50", "Estorno - Pix - João");
    expect(detectReversalPairs([a, b]).size).toBe(0);
  });

  it("does not pair estorno when dates are >7 days apart", () => {
    const a = tx("A", "2026-01-01", "-100", "Pix - João");
    const b = tx("B", "2026-01-10", "100", "Estorno - Pix - João");
    expect(detectReversalPairs([a, b]).size).toBe(0);
  });

  it("returns empty for txs without any Estorno prefix", () => {
    const a = tx("A", "2026-01-20", "-100", "Pix - X");
    const b = tx("B", "2026-01-20", "100", "Pix recebido - X");
    expect(detectReversalPairs([a, b]).size).toBe(0);
  });

  it("ignores tx without fitid", () => {
    const a: ParsedTransaction = { fitid: null, date: "2026-01-20", amount: "-100", description: "Pix - X" };
    const b = tx("B", "2026-01-20", "100", "Estorno - Pix - X");
    expect(detectReversalPairs([a, b]).size).toBe(0);
  });
});

describe("detectReversalPairs — reembolsos", () => {
  const ENVIADO = "Transferência enviada pelo Pix - AMAZON.COM.BR - 15.436.940/0001-03 - EBANX IP LTDA. (0383) Agência: 1 Conta: 1000826878-6";
  const REEMBOLSO = "Reembolso recebido pelo Pix - AMAZON SERVICOS DE VAREJO DO BRASIL LTDA. - 15.436.940/0001-03 - EBANX IP LTDA. (0383) Agência: 1 Conta: 1000826878-6";

  it("pairs reembolso with the matching Pix sent (name varies, CNPJ+account matches)", () => {
    const sent = tx("S", "2026-02-10", "-99.90", ENVIADO);
    const refund = tx("R", "2026-02-12", "99.90", REEMBOLSO);
    const map = detectReversalPairs([sent, refund]);
    expect(map.size).toBe(2);
    expect(map.get("S")?.role).toBe("reembolsada");
    expect(map.get("R")?.role).toBe("reembolso");
    expect(map.get("S")?.pairFitid).toBe("R");
  });

  it("pairs FIFO when there are 6 sent + 6 reembolsos (real Amazon Feb case)", () => {
    const sents = [];
    const refunds = [];
    for (let i = 0; i < 6; i++) {
      sents.push(tx(`S${i}`, "2026-02-10", "-99.90", ENVIADO));
      refunds.push(tx(`R${i}`, "2026-02-12", "99.90", REEMBOLSO));
    }
    const map = detectReversalPairs([...sents, ...refunds]);
    expect(map.size).toBe(12);
    for (let i = 0; i < 6; i++) {
      expect(map.has(`S${i}`)).toBe(true);
      expect(map.has(`R${i}`)).toBe(true);
    }
  });

  it("does not pair reembolso with sent of different CNPJ", () => {
    const sent = tx("S", "2026-02-10", "-99.90",
      "Transferência enviada pelo Pix - Outra Loja - 99.999.999/0001-99 - BCO X (0001) Agência: 1 Conta: 1-1");
    const refund = tx("R", "2026-02-12", "99.90", REEMBOLSO);
    expect(detectReversalPairs([sent, refund]).size).toBe(0);
  });

  it("does not pair reembolso when amounts are different", () => {
    const sent = tx("S", "2026-02-10", "-100.00", ENVIADO);
    const refund = tx("R", "2026-02-12", "50.00", REEMBOLSO);
    expect(detectReversalPairs([sent, refund]).size).toBe(0);
  });

  it("does not pair reembolso when window > 30 days", () => {
    const sent = tx("S", "2026-01-01", "-99.90", ENVIADO);
    const refund = tx("R", "2026-02-15", "99.90", REEMBOLSO);
    expect(detectReversalPairs([sent, refund]).size).toBe(0);
  });

  it("pairs reembolso even with delay up to 30 days", () => {
    const sent = tx("S", "2026-02-01", "-99.90", ENVIADO);
    const refund = tx("R", "2026-02-28", "99.90", REEMBOLSO);
    expect(detectReversalPairs([sent, refund]).size).toBe(2);
  });
});

describe("detectReversalPairs — combined", () => {
  it("handles a mix of estornos and reembolsos in the same batch", () => {
    const eOrig = tx("EO", "2026-01-20", "-100", "Pix - João");
    const e = tx("E", "2026-01-20", "100", "Estorno - Pix - João");
    const rOrig = tx("RO", "2026-02-10", "-50",
      "Transferência enviada pelo Pix - AMAZON.COM.BR - 15.436.940/0001-03 - EBANX IP LTDA. (0383) Agência: 1 Conta: 1");
    const r = tx("R", "2026-02-12", "50",
      "Reembolso recebido pelo Pix - AMAZON SERVICOS - 15.436.940/0001-03 - EBANX IP LTDA. (0383) Agência: 1 Conta: 1");
    const map = detectReversalPairs([eOrig, e, rOrig, r]);
    expect(map.size).toBe(4);
    expect(map.get("E")?.role).toBe("estorno");
    expect(map.get("EO")?.role).toBe("estornada");
    expect(map.get("R")?.role).toBe("reembolso");
    expect(map.get("RO")?.role).toBe("reembolsada");
  });
});
