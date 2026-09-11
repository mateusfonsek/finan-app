import { describe, it, expect } from "vitest";
import { computeFreshness } from "./freshness";
import type { AccountFreshness } from "$lib/bindings";

function account(name: string, latest_date: string | null, account_id = 1): AccountFreshness {
  return { account_id, name, latest_date };
}

/** Local midnight, the same construction the production code uses for "today". */
function day(iso: string): Date {
  const [y, m, d] = iso.split("-").map(Number);
  return new Date(y, m - 1, d);
}

describe("computeFreshness", () => {
  it("returns null when there is no account", () => {
    expect(computeFreshness([], day("2026-07-12"))).toBeNull();
  });

  it("returns null when no account carries a transaction", () => {
    const rows = [account("Checking", null, 1), account("Card", null, 2)];

    expect(computeFreshness(rows, day("2026-07-12"))).toBeNull();
  });

  it("counts the days between the newest transaction and today", () => {
    const rows = [account("Checking", "2026-06-30")];

    expect(computeFreshness(rows, day("2026-07-12"))?.daysBehind).toBe(12);
  });

  it("is zero on the day of the newest transaction", () => {
    const rows = [account("Checking", "2026-07-12")];

    expect(computeFreshness(rows, day("2026-07-12"))?.daysBehind).toBe(0);
  });

  /** A card statement can carry postings dated after today; that is not a gap. */
  it("is zero when the newest transaction is in the future", () => {
    const rows = [account("Card", "2026-08-05")];

    const result = computeFreshness(rows, day("2026-07-12"));

    expect(result?.daysBehind).toBe(0);
    expect(result?.latestDate).toBe("2026-08-05");
  });

  it("counts across a month boundary", () => {
    const rows = [account("Checking", "2026-01-28")];

    expect(computeFreshness(rows, day("2026-02-03"))?.daysBehind).toBe(6);
  });

  it("counts across a year boundary", () => {
    const rows = [account("Checking", "2025-12-30")];

    expect(computeFreshness(rows, day("2026-01-02"))?.daysBehind).toBe(3);
  });

  /** The headline is the ceiling of any estimate, so the account furthest
   *  behind wins — a checking account imported today must not hide it. */
  it("announces the account that is furthest behind", () => {
    const rows = [account("Checking", "2026-07-12", 1), account("Card", "2026-06-30", 2)];

    const result = computeFreshness(rows, day("2026-07-12"));

    expect(result?.latestDate).toBe("2026-06-30");
    expect(result?.daysBehind).toBe(12);
  });

  /** An account imported once and then abandoned would otherwise own the
   *  headline forever, and its gap says nothing about the current data. */
  it("ignores an account more than 60 days behind", () => {
    const rows = [account("Old", "2026-01-10", 1), account("Checking", "2026-07-07", 2)];

    const result = computeFreshness(rows, day("2026-07-12"));

    expect(result?.latestDate).toBe("2026-07-07");
    expect(result?.accounts.map((a) => a.name)).toEqual(["Checking"]);
  });

  it("keeps an account exactly 60 days behind", () => {
    const rows = [account("Checking", "2026-05-13")];

    expect(computeFreshness(rows, day("2026-07-12"))?.daysBehind).toBe(60);
  });

  it("drops an account 61 days behind", () => {
    const rows = [account("Checking", "2026-05-12")];

    expect(computeFreshness(rows, day("2026-07-12"))).toBeNull();
  });

  it("returns null when every account is beyond the cutoff", () => {
    const rows = [account("Old", "2026-01-10", 1), account("Older", "2025-11-02", 2)];

    expect(computeFreshness(rows, day("2026-07-12"))).toBeNull();
  });

  it("lists the accounts that carry data, furthest behind first", () => {
    const rows = [
      account("Checking", "2026-07-12", 1),
      account("Empty", null, 2),
      account("Card", "2026-06-30", 3),
    ];

    const result = computeFreshness(rows, day("2026-07-12"));

    expect(result?.accounts).toEqual([
      { accountId: 3, name: "Card", latestDate: "2026-06-30" },
      { accountId: 1, name: "Checking", latestDate: "2026-07-12" },
    ]);
  });
});
