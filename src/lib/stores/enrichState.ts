/**
 * State of the background enrichment, as a reduction of the events the backend
 * thread emits.
 *
 * Outside the `.svelte` for the same reason as `toastState.ts`: this is the
 * part with real rules, and the part that needs tests. The component only draws
 * whatever comes out of here.
 */
import type { AutoClassifyReport, EnrichEvent } from "$lib/bindings";

export type EnrichPhase = "idle" | "running" | "done" | "cancelled" | "failed";

export type EnrichState = {
  phase: EnrichPhase;
  /** Lookups processed — includes failed ones and those skipped for having a rule. */
  done: number;
  /** Known before the first lookup: this is what allows a determinate bar. */
  total: number;
  /** Last company identified. `null` until the first one resolves. */
  label: string | null;
  failed: number;
  /** Only exists in a terminal success or cancellation state. */
  report: AutoClassifyReport | null;
  error: string | null;
};

export const initialEnrichState: EnrichState = {
  phase: "idle",
  done: 0,
  total: 0,
  label: null,
  failed: 0,
  report: null,
  error: null,
};

export function isTerminal(phase: EnrichPhase): boolean {
  return phase === "done" || phase === "cancelled" || phase === "failed";
}

export function reduceEnrich(state: EnrichState, event: EnrichEvent): EnrichState {
  switch (event.kind) {
    // Resets everything: a new job cannot inherit the count, label or error of
    // the previous one. Reusing the old state would start the bar full.
    case "Started":
      return { ...initialEnrichState, phase: "running", total: event.total };

    case "Resolved":
      return { ...state, done: event.done, label: event.label };

    case "Unresolved":
      return {
        ...state,
        done: event.done,
        label: event.resolution.razao_social ?? event.resolution.nome_fantasia ?? state.label,
      };

    case "Failed":
      return { ...state, done: event.done, failed: state.failed + 1 };

    // `done: state.total` closes the bar: lookups skipped for already having a
    // rule emit no event, so the counter can stop short of the total and the bar
    // would freeze at 80% on work that finished.
    case "Finished":
      return { ...state, phase: "done", done: state.total, report: event.report };

    case "Cancelled":
      return { ...state, phase: "cancelled", report: event.report };

    case "Aborted":
      return { ...state, phase: "failed", error: event.message };

    default:
      return state;
  }
}
