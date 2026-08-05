/**
 * Centro de atividade: o que o app está fazendo em segundo plano.
 *
 * Existe porque o enriquecimento por CNPJ deixou de segurar o import. Com o
 * trabalho rodando depois que a tela de resultado já apareceu, alguém precisa
 * ser dono do progresso E do relatório final — e não pode ser o componente, que
 * desmonta assim que a pessoa navega para outra tela.
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
    /** Há trabalho em andamento — o que decide se a superfície aparece. */
    get busy() {
      return enrich.phase === "running";
    },
    get settled() {
      return isTerminal(enrich.phase);
    },

    /** Vale ocupar espaço na tela?
     *
     *  Terminar não basta. Com o enriquecimento desligado o backend emite
     *  `Started { total: 0 }` seguido de `Finished` com relatório vazio — para
     *  a tela ter um caminho só — e sem esta distinção o app anunciaria
     *  "nenhuma regra nova" depois de todo import, para quem nunca ligou a
     *  funcionalidade. Erro sempre aparece: falha silenciosa é o que este
     *  trabalho existe para acabar. */
    get visible() {
      if (enrich.phase === "running" || enrich.phase === "failed") return true;
      return isTerminal(enrich.phase) && enrich.total > 0;
    },

    async start(accountId: number | null) {
      // Sem otimismo aqui: a fase só vira "running" quando o backend emitir
      // `Started`. Antecipar faria uma barra aparecer para um job que o backend
      // pode recusar (já há um rodando) ou nem iniciar (enriquecimento off).
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

    /** Chamado ao começar um import novo, para a tela não herdar o anterior. */
    clear() {
      enrich = initialEnrichState;
    },

    /** Substitui o relatório — a tela de Import edita as regras criadas
     *  (renomear, trocar categoria, apagar) e o store é quem tem a verdade. */
    patchReport(next: AutoClassifyReport) {
      enrich = { ...enrich, report: next };
    },
  };
}

export const activity = createActivityStore();
