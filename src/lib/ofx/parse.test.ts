import { describe, it, expect } from "vitest";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
import { parseOfx } from "./parse";

const __dirname = dirname(fileURLToPath(import.meta.url));
const itauFixture = readFileSync(
  join(__dirname, "__fixtures__/itau-minimal.ofx"),
  "utf-8",
);

describe("parseOfx — Itaú minimal fixture", () => {
  const result = parseOfx(itauFixture);

  it("detects Itaú as the bank", () => {
    expect(result.account.bank).toBe("itau");
  });

  it("extracts ACCTID and BANKID", () => {
    expect(result.account.ofxAcctid).toBe("56789-0");
    expect(result.account.ofxBankid).toBe("0341");
  });

  it("formats a sensible display name", () => {
    expect(result.account.displayName).toContain("Itau");
    expect(result.account.displayName).toContain("1234");
    expect(result.account.displayName).toContain("56789-0");
  });

  it("extracts 3 transactions", () => {
    expect(result.transactions).toHaveLength(3);
  });

  it("each transaction has fitid, date, amount, description", () => {
    for (const t of result.transactions) {
      expect(t.fitid).toMatch(/^ITAU\d+$/);
      expect(t.date).toMatch(/^\d{4}-\d{2}-\d{2}$/);
      expect(typeof t.amount).toBe("string");
      expect(t.description.length).toBeGreaterThan(0);
    }
  });

  it("summary aggregates in/out/net correctly", () => {
    expect(result.summary.totalIn).toBe("3500.00");
    expect(result.summary.totalOut).toBe("79.90");
    expect(result.summary.net).toBe("3420.10");
  });

  it("summary captures date range", () => {
    expect(result.summary.earliest).toBe("2026-03-05");
    expect(result.summary.latest).toBe("2026-03-15");
  });
});
