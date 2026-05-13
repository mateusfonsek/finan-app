import { describe, it, expect } from "vitest";
import { formatMoney } from "./money";

describe("formatMoney", () => {
  it("formats positive amount as BRL with two decimals", () => {
    expect(formatMoney("123.45")).toMatch(/R\$\s*123,45/);
  });

  it("formats negative amount with minus sign", () => {
    expect(formatMoney("-50.00")).toMatch(/-R\$\s*50,00|R\$\s*-50,00/);
  });

  it("formats zero", () => {
    expect(formatMoney("0")).toMatch(/R\$\s*0,00/);
  });

  it("throws on non-numeric input", () => {
    expect(() => formatMoney("abc")).toThrow(/invalid amount/);
  });
});
