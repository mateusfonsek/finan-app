import { describe, expect, it } from "vitest";
import type { EnrichEvent } from "$lib/bindings";
import { initialEnrichState, isTerminal, reduceEnrich } from "./enrichState";

const emptyReport = { created_rules: [], txs_classified: 0, unresolved: [] };

function run(events: EnrichEvent[]) {
  return events.reduce(reduceEnrich, initialEnrichState);
}

describe("reduceEnrich", () => {
  it("começa parado, sem total e sem relatório", () => {
    expect(initialEnrichState.phase).toBe("idle");
    expect(initialEnrichState.total).toBe(0);
    expect(initialEnrichState.report).toBeNull();
  });

  it("Started define o total e entra em execução", () => {
    const s = run([{ kind: "Started", total: 42 }]);
    expect(s.phase).toBe("running");
    expect(s.total).toBe(42);
    expect(s.done).toBe(0);
  });

  it("Resolved avança o contador e guarda o rótulo mostrado", () => {
    const s = run([
      { kind: "Started", total: 2 },
      { kind: "Resolved", done: 1, label: "ENERGISA", rule: {} as never },
    ]);
    expect(s.done).toBe(1);
    expect(s.label).toBe("ENERGISA");
  });

  it("Unresolved usa a razão social como rótulo", () => {
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

  it("Failed conta a falha sem sair da execução", () => {
    const s = run([
      { kind: "Started", total: 2 },
      { kind: "Failed", done: 1, tax_id: "09.095.183/0001-40" },
    ]);
    expect(s.phase).toBe("running");
    expect(s.failed).toBe(1);
    expect(s.done).toBe(1);
  });

  it("Finished guarda o relatório e encerra", () => {
    const s = run([
      { kind: "Started", total: 1 },
      { kind: "Finished", report: emptyReport },
    ]);
    expect(s.phase).toBe("done");
    expect(s.report).toEqual(emptyReport);
  });

  it("Finished fecha a barra mesmo com consultas puladas", () => {
    // CNPJs que já têm regra não emitem evento: o contador para em 1 mas o
    // total é 5. Sem o ajuste, a barra congelaria em 20% num trabalho pronto.
    const s = run([
      { kind: "Started", total: 5 },
      { kind: "Resolved", done: 1, label: "ENERGISA", rule: {} as never },
      { kind: "Finished", report: emptyReport },
    ]);
    expect(s.done).toBe(5);
  });

  it("Cancelled também guarda o relatório — o parcial vale", () => {
    const s = run([
      { kind: "Started", total: 5 },
      { kind: "Cancelled", report: emptyReport },
    ]);
    expect(s.phase).toBe("cancelled");
    expect(s.report).toEqual(emptyReport);
  });

  it("Aborted guarda a mensagem e não inventa relatório", () => {
    const s = run([
      { kind: "Started", total: 5 },
      { kind: "Aborted", message: "banco indisponível" },
    ]);
    expect(s.phase).toBe("failed");
    expect(s.error).toBe("banco indisponível");
    expect(s.report).toBeNull();
  });

  it("um Started novo zera o estado anterior por completo", () => {
    const s = run([
      { kind: "Started", total: 2 },
      { kind: "Failed", done: 1, tax_id: "x" },
      { kind: "Finished", report: emptyReport },
      { kind: "Started", total: 7 },
    ]);
    expect(s).toEqual({ ...initialEnrichState, phase: "running", total: 7 });
  });

  it("isTerminal só é verdadeiro depois que o trabalho acabou", () => {
    expect(isTerminal("idle")).toBe(false);
    expect(isTerminal("running")).toBe(false);
    expect(isTerminal("done")).toBe(true);
    expect(isTerminal("cancelled")).toBe(true);
    expect(isTerminal("failed")).toBe(true);
  });
});
