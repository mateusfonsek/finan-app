import { describe, it, expect } from "vitest";
import { splitTools, cutoffLabel, WINDOW_OPTIONS } from "./toolGroups";
import type { McpToolState } from "$lib/bindings";

function tool(over: Partial<McpToolState> = {}): McpToolState {
  return { name: "list_transactions", description: "d", write: false, enabled: true, ...over };
}

describe("splitTools", () => {
  it("separates reads from writes", () => {
    const { read, write } = splitTools([
      tool({ name: "list_bills" }),
      tool({ name: "create_rule", write: true, enabled: false }),
    ]);

    expect(read.map((t) => t.name)).toEqual(["list_bills"]);
    expect(write.map((t) => t.name)).toEqual(["create_rule"]);
  });

  it("keeps the backend's order inside each group", () => {
    const { read } = splitTools([
      tool({ name: "b" }),
      tool({ name: "a" }),
    ]);

    expect(read.map((t) => t.name)).toEqual(["b", "a"]);
  });

  it("handles an empty list", () => {
    expect(splitTools([])).toEqual({ read: [], write: [] });
  });
});

describe("cutoffLabel", () => {
  /** The control shows what it causes, not just the number it holds. */
  it("names the first month the agent can reach", () => {
    expect(cutoffLabel(12, new Date("2026-09-13"))).toBe("2025-10");
  });

  it("counts the current month as one of them", () => {
    expect(cutoffLabel(1, new Date("2026-09-13"))).toBe("2026-09");
  });

  it("crosses the year boundary", () => {
    expect(cutoffLabel(3, new Date("2026-01-15"))).toBe("2025-11");
  });

  it("is null when there is no limit", () => {
    expect(cutoffLabel(0, new Date("2026-09-13"))).toBeNull();
  });
});

describe("WINDOW_OPTIONS", () => {
  it("offers the five choices the screen draws, unlimited last", () => {
    expect(WINDOW_OPTIONS).toEqual([3, 6, 12, 24, 0]);
  });
});
