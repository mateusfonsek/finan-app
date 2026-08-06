/**
 * Activity centre: what the app is doing in the background.
 *
 * It exists because tax-id enrichment stopped holding up the import. With the
 * work running after the result screen has already appeared, something has to
 * own the progress AND the final report — and it cannot be the component, which
 * unmounts as soon as the user navigates to another screen.
 */
import { cancelCnpjEnrichment, startCnpjEnrichment } from "$lib/api/enrichJob";
import type { AutoClassifyReport } from "$lib/bindings";
import {
  initialEnrichState,
  isTerminal,
  reduceEnrich,
  type EnrichState,
} from "./enrichState";

export function createActivityStore() {
  let enrich = $state<EnrichState>(initialEnrichState);

  return {
    get enrich() {
      return enrich;
    },
    /** Work is in progress — what decides whether the surface shows. */
    get busy() {
      return enrich.phase === "running";
    },
    get settled() {
      return isTerminal(enrich.phase);
    },

    /** Is this worth screen space?
     *
     *  Finishing is not enough. With enrichment off the backend emits
     *  `Started { total: 0 }` followed by `Finished` with an empty report — so
     *  the screen has a single path — and without this distinction the app
     *  would announce "no new rules" after every import, to people who never
     *  turned the feature on. An error always shows: silent failure is exactly
     *  what this work exists to end. */
    get visible() {
      if (enrich.phase === "running" || enrich.phase === "failed") return true;
      return isTerminal(enrich.phase) && enrich.total > 0;
    },

    async start(accountId: number | null) {
      // No optimism here: the phase only turns "running" once the backend emits
      // `Started`. Anticipating would show a bar for a job the backend may
      // refuse (one already running) or never start (enrichment off).
      try {
        await startCnpjEnrichment(accountId, (event) => {
          enrich = reduceEnrich(enrich, event);
        });
      } catch (e) {
        enrich = {
          ...enrich,
          phase: "failed",
          error: e instanceof Error ? e.message : String(e),
        };
      }
    },

    async cancel() {
      await cancelCnpjEnrichment();
    },

    /** Called when a new import starts, so the screen does not inherit the previous one. */
    clear() {
      enrich = initialEnrichState;
    },

    /** Replaces the report — the Import screen edits the created rules (rename,
     *  change category, delete) and the store is what holds the truth. */
    patchReport(next: AutoClassifyReport) {
      enrich = { ...enrich, report: next };
    },
  };
}

export const activity = createActivityStore();
