import { beforeEach, describe, expect, it, vi } from "vitest";
import type { EnrichEvent } from "$lib/bindings";

const api = vi.hoisted(() => ({
  startCnpjEnrichment: vi.fn(),
  cancelCnpjEnrichment: vi.fn(),
}));
vi.mock("$lib/api/enrichJob", () => api);

import { createActivityStore } from "./activity.svelte";

const emptyReport = { created_rules: [], txs_classified: 0, unresolved: [] };

/** Puts the test in control of when each event arrives. */
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
  it("starts idle, with no surface to show", () => {
    const s = createActivityStore();
    expect(s.busy).toBe(false);
    expect(s.settled).toBe(false);
  });

  it("only becomes busy once the backend confirms the start", async () => {
    const s = createActivityStore();
    const ch = captureEmitter();

    await s.start(1);
    expect(s.busy).toBe(false); // nothing yet — no event has arrived

    ch.send({ kind: "Started", total: 3 });
    expect(s.busy).toBe(true);
    expect(s.enrich.total).toBe(3);
  });

  it("keeps the report on finish and leaves the busy state", async () => {
    const s = createActivityStore();
    const ch = captureEmitter();

    await s.start(1);
    ch.send({ kind: "Started", total: 1 });
    ch.send({ kind: "Finished", report: emptyReport });

    expect(s.busy).toBe(false);
    expect(s.settled).toBe(true);
    expect(s.enrich.report).toEqual(emptyReport);
  });

  it("state outlives the screen — nothing here depends on a mounted component", async () => {
    const s = createActivityStore();
    const ch = captureEmitter();

    await s.start(1);
    ch.send({ kind: "Started", total: 10 });
    ch.send({ kind: "Resolved", done: 4, label: "ENERGISA", rule: {} as never });

    // No component lifecycle is involved: reading later returns the same.
    expect(s.enrich.done).toBe(4);
    expect(s.enrich.label).toBe("ENERGISA");
  });

  it("a failure to start becomes an error state, not a leaking exception", async () => {
    const s = createActivityStore();
    api.startCnpjEnrichment.mockRejectedValue(new Error("enrichment already running"));

    await s.start(1);

    expect(s.enrich.phase).toBe("failed");
    expect(s.enrich.error).toBe("enrichment already running");
  });

  it("with enrichment off, nothing shows on screen", async () => {
    const s = createActivityStore();
    const ch = captureEmitter();

    // What the backend emits when the feature is off: an empty
    // Started/Finished pair, only so the screen has a single path.
    await s.start(1);
    ch.send({ kind: "Started", total: 0 });
    ch.send({ kind: "Finished", report: emptyReport });

    expect(s.settled).toBe(true);
    expect(s.visible).toBe(false);
  });

  it("real work that finishes stays visible", async () => {
    const s = createActivityStore();
    const ch = captureEmitter();

    await s.start(1);
    ch.send({ kind: "Started", total: 3 });
    ch.send({ kind: "Finished", report: emptyReport });

    expect(s.visible).toBe(true);
  });

  it("an error shows even when no lookup ever happened", async () => {
    const s = createActivityStore();
    api.startCnpjEnrichment.mockRejectedValue(new Error("enrichment already running"));

    await s.start(1);

    expect(s.enrich.total).toBe(0);
    expect(s.visible).toBe(true);
  });

  it("cancel asks the backend to stop", async () => {
    const s = createActivityStore();
    await s.cancel();
    expect(api.cancelCnpjEnrichment).toHaveBeenCalledOnce();
  });

  it("patchReport swaps the report without touching the rest of the state", async () => {
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

  it("clear returns to the initial state", async () => {
    const s = createActivityStore();
    const ch = captureEmitter();
    await s.start(1);
    ch.send({ kind: "Started", total: 2 });

    s.clear();

    expect(s.busy).toBe(false);
    expect(s.enrich.total).toBe(0);
  });
});
