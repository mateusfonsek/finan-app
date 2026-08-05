/**
 * Estado do enriquecimento em segundo plano, como redução dos eventos que a
 * thread do backend emite.
 *
 * Fora do `.svelte` pelo mesmo motivo de `toastState.ts`: é a parte com regras,
 * e é a que precisa de teste. O componente só desenha o que sai daqui.
 */
import type { AutoClassifyReport, EnrichEvent } from "$lib/bindings";

export type EnrichPhase = "idle" | "running" | "done" | "cancelled" | "failed";

export type EnrichState = {
  phase: EnrichPhase;
  /** Consultas processadas — inclui as que falharam e as puladas por já ter regra. */
  done: number;
  /** Conhecido antes da primeira consulta: é ele que permite barra determinada. */
  total: number;
  /** Última empresa identificada. `null` até a primeira resolver. */
  label: string | null;
  failed: number;
  /** Só existe num estado terminal de sucesso ou cancelamento. */
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
    // Zera tudo: um job novo não pode herdar contagem, rótulo nem erro do
    // anterior. Reaproveitar o estado antigo faria a barra começar cheia.
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

    // `done: state.total` fecha a barra: consultas puladas por já terem regra
    // não emitem evento, então o contador pode parar antes do total e a barra
    // congelaria em 80% num trabalho que terminou.
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
