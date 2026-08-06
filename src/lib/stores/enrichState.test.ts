import { describe, expect, it } from "vitest";
import type { EnrichEvent } from "$lib/bindings";
import { initialEnrichState, isTerminal, reduceEnrich } from "./enrichState";

const emptyReport = { created_rules: [], txs_classified: 0, unresolved: [] };

function run(events: EnrichEvent[]) {
  return events.reduce(reduceEnrich, initialEnrichState);
}

describe("reduceEnrich", () => {
  it("starts idle, with no total and no report", () => {
    expect(initialEnrichState.phase).toBe("idle");
    expect(initialEnrichState.total).toBe(0);
    expect(initialEnrichState.report).toBeNull();
  });

  it("Started sets the total and enters the running phase", () => {
    const s = run([{ kind: "Started", total: 42 }]);
    expect(s.phase).toBe("running");
    expect(s.total).toBe(42);
    expect(s.done).toBe(0);
  });

  it("Resolved advances the counter and keeps the displayed label", () => {
    const s = run([
      { kind: "Started", total: 2 },
      { kind: "Resolved", done: 1, label: "ENERGISA", rule: {} as never },
    ]);
    expect(s.done).toBe(1);
    expect(s.label).toBe("ENERGISA");
  });

  it("Unresolved uses the legal name as the label", () => {
    const s = run([
      { kind: "Started", total: 2 },
      {
        kind: "Unresolved",
        done: 1,
        resolution: {
          cnpj: "09.095.183/0001-40",
          razao_social: "FAZENDA LTDA",
          nome_fantasia: null,
          cnae_fiscal: "0111301",
          cnae_fiscal_descricao: null,
          suggested_category_id: null,
        },
      },
    ]);
    expect(s.done).toBe(1);
    expect(s.label).toBe("FAZENDA LTDA");
  });

  it("Failed counts the failure without leaving the running phase", () => {
    const s = run([
      { kind: "Started", total: 2 },
      { kind: "Failed", done: 1, tax_id: "09.095.183/0001-40" },
    ]);
    expect(s.phase).toBe("running");
    expect(s.failed).toBe(1);
    expect(s.done).toBe(1);
  });

  it("Finished keeps the report and ends", () => {
    const s = run([
      { kind: "Started", total: 1 },
      { kind: "Finished", report: emptyReport },
    ]);
    expect(s.phase).toBe("done");
    expect(s.report).toEqual(emptyReport);
  });

  it("Finished closes the bar even with skipped lookups", () => {
    // Tax ids that already have a rule emit no event: the counter stops at 1
    // but the total is 5. Without the adjustment the bar would freeze at 20% on
    // finished work.
    const s = run([
      { kind: "Started", total: 5 },
      { kind: "Resolved", done: 1, label: "ENERGISA", rule: {} as never },
      { kind: "Finished", report: emptyReport },
    ]);
    expect(s.done).toBe(5);
  });

  it("Cancelled keeps the report too — the partial one counts", () => {
    const s = run([
      { kind: "Started", total: 5 },
      { kind: "Cancelled", report: emptyReport },
    ]);
    expect(s.phase).toBe("cancelled");
    expect(s.report).toEqual(emptyReport);
  });

  it("Aborted keeps the message and invents no report", () => {
    const s = run([
      { kind: "Started", total: 5 },
      { kind: "Aborted", message: "banco indisponível" },
    ]);
    expect(s.phase).toBe("failed");
    expect(s.error).toBe("banco indisponível");
    expect(s.report).toBeNull();
  });

  it("a fresh Started wipes the previous state entirely", () => {
    const s = run([
      { kind: "Started", total: 2 },
      { kind: "Failed", done: 1, tax_id: "x" },
      { kind: "Finished", report: emptyReport },
      { kind: "Started", total: 7 },
    ]);
    expect(s).toEqual({ ...initialEnrichState, phase: "running", total: 7 });
  });

  it("isTerminal is only true once the work has ended", () => {
    expect(isTerminal("idle")).toBe(false);
    expect(isTerminal("running")).toBe(false);
    expect(isTerminal("done")).toBe(true);
    expect(isTerminal("cancelled")).toBe(true);
    expect(isTerminal("failed")).toBe(true);
  });
});
