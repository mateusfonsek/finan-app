import { beforeEach, describe, expect, it, vi } from "vitest";
import type { EnrichEvent } from "$lib/bindings";

const api = vi.hoisted(() => ({
  startCnpjEnrichment: vi.fn(),
  cancelCnpjEnrichment: vi.fn(),
}));
vi.mock("$lib/api/enrichJob", () => api);

import { createActivityStore } from "./activity.svelte";

const emptyReport = { created_rules: [], txs_classified: 0, unresolved: [] };

/** Deixa o teste no controle de quando cada evento chega. */
function captureEmitter() {
  let emit: (e: EnrichEvent) => void = () => {};
  api.startCnpjEnrichment.mockImplementation(
    async (_id: number | null, onEvent: (e: EnrichEvent) => void) => {
      emit = onEvent;
    },
  );
  return { send: (e: EnrichEvent) => emit(e) };
}

beforeEach(() => {
  api.startCnpjEnrichment.mockReset();
  api.cancelCnpjEnrichment.mockReset();
});

describe("activity store", () => {
  it("começa ocioso, sem superfície para mostrar", () => {
    const s = createActivityStore();
    expect(s.busy).toBe(false);
    expect(s.settled).toBe(false);
  });

  it("fica ocupado só quando o backend confirma o início", async () => {
    const s = createActivityStore();
    const ch = captureEmitter();

    await s.start(1);
    expect(s.busy).toBe(false); // ainda nada — nenhum evento chegou

    ch.send({ kind: "Started", total: 3 });
    expect(s.busy).toBe(true);
    expect(s.enrich.total).toBe(3);
  });

  it("guarda o relatório ao terminar e sai de ocupado", async () => {
    const s = createActivityStore();
    const ch = captureEmitter();

    await s.start(1);
    ch.send({ kind: "Started", total: 1 });
    ch.send({ kind: "Finished", report: emptyReport });

    expect(s.busy).toBe(false);
    expect(s.settled).toBe(true);
    expect(s.enrich.report).toEqual(emptyReport);
  });

  it("o estado sobrevive à tela — nada aqui depende de componente montado", async () => {
    const s = createActivityStore();
    const ch = captureEmitter();

    await s.start(1);
    ch.send({ kind: "Started", total: 10 });
    ch.send({ kind: "Resolved", done: 4, label: "ENERGISA", rule: {} as never });

    // Nenhum ciclo de vida de componente é envolvido: ler depois devolve o mesmo.
    expect(s.enrich.done).toBe(4);
    expect(s.enrich.label).toBe("ENERGISA");
  });

  it("uma falha ao iniciar vira estado de erro, não exceção vazando", async () => {
    const s = createActivityStore();
    api.startCnpjEnrichment.mockRejectedValue(new Error("já em andamento"));

    await s.start(1);

    expect(s.enrich.phase).toBe("failed");
    expect(s.enrich.error).toBe("já em andamento");
  });

  it("cancel pede parada ao backend", async () => {
    const s = createActivityStore();
    await s.cancel();
    expect(api.cancelCnpjEnrichment).toHaveBeenCalledOnce();
  });

  it("patchReport troca o relatório sem tocar o resto do estado", async () => {
    const s = createActivityStore();
    const ch = captureEmitter();
    await s.start(1);
    ch.send({ kind: "Started", total: 2 });
    ch.send({ kind: "Finished", report: emptyReport });

    const next = { ...emptyReport, txs_classified: 9 };
    s.patchReport(next);

    expect(s.enrich.report).toEqual(next);
    expect(s.enrich.phase).toBe("done");
  });

  it("clear volta ao início", async () => {
    const s = createActivityStore();
    const ch = captureEmitter();
    await s.start(1);
    ch.send({ kind: "Started", total: 2 });

    s.clear();

    expect(s.busy).toBe(false);
    expect(s.enrich.total).toBe(0);
  });
});
