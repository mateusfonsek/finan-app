import type { ParsedTransaction } from "./types";

/**
 * Papel de uma transação dentro de um par.
 * Quatro valores porque queremos rótulos distintos para estorno vs reembolso
 * na UI, mas todos significam "esta transação faz parte de um par que soma zero".
 */
export type ReversalRole = "estorno" | "estornada" | "reembolso" | "reembolsada";

export interface ReversalInfo {
  role: ReversalRole;
  /** fitid da outra ponta do par. */
  pairFitid: string;
}

const ESTORNO_PREFIX_RE = /^Estorno\s*-\s*/i;
const REEMBOLSO_PREFIX_RE = /^Reembolso recebido pelo Pix\s*-\s*/i;
const PIX_ENVIADO_PREFIX_RE = /^Transferência enviada pelo Pix\s*-\s*/i;

/** Distância em dias entre duas datas ISO YYYY-MM-DD. */
function daysBetween(a: string, b: string): number {
  const da = Date.parse(a + "T00:00:00Z");
  const db = Date.parse(b + "T00:00:00Z");
  if (Number.isNaN(da) || Number.isNaN(db)) return Infinity;
  return Math.abs((da - db) / 86_400_000);
}

/**
 * De uma description Pix `<Prefixo> - NOME - ID - BANCO/AG/CONTA`, extrai
 * `ID - BANCO/AG/CONTA` (tudo depois do nome). Isso identifica unicamente a
 * contraparte mesmo quando o NOME varia (razão social vs nome fantasia, como
 * acontece em reembolsos da Amazon).
 *
 * Retorna `null` se a descrição não começa com um prefixo Pix conhecido.
 */
function pixCounterpartySignature(description: string): string | null {
  let body: string | null = null;
  for (const re of [PIX_ENVIADO_PREFIX_RE, REEMBOLSO_PREFIX_RE]) {
    const m = description.match(re);
    if (m) {
      body = description.substring(m[0].length);
      break;
    }
  }
  if (body === null) return null;
  const firstDash = body.indexOf(" - ");
  if (firstDash === -1) return null;
  return body.substring(firstDash + 3).trim();
}

/**
 * Encontra pares (estorno↔revertida e reembolso↔revertida) numa lista de
 * transações OFX.
 *
 * **Estornos**: a description do estorno começa com "Estorno - " e o resto é
 * IGUAL à description da transação original (depois de `trim`). Janela: ±7 dias.
 *
 * **Reembolsos**: a description começa com "Reembolso recebido pelo Pix - ".
 * O par é encontrado por **assinatura da contraparte** (CNPJ + dados bancários),
 * não pelo nome — porque a Amazon manda "AMAZON.COM.BR" no Pix saindo e
 * "AMAZON SERVICOS DE VAREJO DO BRASIL LTDA." no reembolso. Janela: ±30 dias.
 *
 * Em ambos os casos, valores precisam ser opostos em sinal e iguais em magnitude.
 * Pareamento FIFO quando há múltiplos candidatos.
 */
export function detectReversalPairs(
  txs: ParsedTransaction[],
): Map<string, ReversalInfo> {
  const result = new Map<string, ReversalInfo>();
  const used = new Set<string>();

  // Fase 1: estornos (match por descrição exata).
  for (const e of txs) {
    if (!e.fitid || used.has(e.fitid)) continue;
    if (!ESTORNO_PREFIX_RE.test(e.description)) continue;

    const core = e.description.replace(ESTORNO_PREFIX_RE, "").trim();
    const estornoAmt = Number(e.amount);
    if (!Number.isFinite(estornoAmt)) continue;

    const candidate = txs.find((t) => {
      if (!t.fitid || used.has(t.fitid) || t.fitid === e.fitid) return false;
      if (t.description.trim() !== core) return false;
      const amt = Number(t.amount);
      if (!Number.isFinite(amt)) return false;
      if (Math.abs(amt + estornoAmt) > 0.01) return false;
      return daysBetween(t.date, e.date) <= 7;
    });

    if (candidate?.fitid) {
      result.set(e.fitid, { role: "estorno", pairFitid: candidate.fitid });
      result.set(candidate.fitid, { role: "estornada", pairFitid: e.fitid });
      used.add(e.fitid);
      used.add(candidate.fitid);
    }
  }

  // Fase 2: reembolsos (match por assinatura CNPJ+conta).
  for (const r of txs) {
    if (!r.fitid || used.has(r.fitid)) continue;
    if (!REEMBOLSO_PREFIX_RE.test(r.description)) continue;

    const sig = pixCounterpartySignature(r.description);
    if (!sig) continue;
    const reembolsoAmt = Number(r.amount);
    if (!Number.isFinite(reembolsoAmt)) continue;

    const candidate = txs.find((t) => {
      if (!t.fitid || used.has(t.fitid) || t.fitid === r.fitid) return false;
      // O original é um "Transferência enviada pelo Pix" com a mesma assinatura.
      if (!PIX_ENVIADO_PREFIX_RE.test(t.description)) return false;
      const otherSig = pixCounterpartySignature(t.description);
      if (otherSig !== sig) return false;
      const amt = Number(t.amount);
      if (!Number.isFinite(amt)) return false;
      if (Math.abs(amt + reembolsoAmt) > 0.01) return false;
      return daysBetween(t.date, r.date) <= 30;
    });

    if (candidate?.fitid) {
      result.set(r.fitid, { role: "reembolso", pairFitid: candidate.fitid });
      result.set(candidate.fitid, { role: "reembolsada", pairFitid: r.fitid });
      used.add(r.fitid);
      used.add(candidate.fitid);
    }
  }

  return result;
}
