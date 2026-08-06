import type { ParsedTransaction } from "./types";

/**
 * A transaction's role within a pair. Four values because the UI labels
 * chargebacks and refunds differently, but all mean "part of a pair summing to
 * zero".
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
/** Nubank credit-card chargeback: `Estorno de "Merchant" (Merchant)`. */
const ESTORNO_CC_RE = /^Estorno\s+de\s+"([^"]+)"\s*(?:\([^)]+\))?\s*$/i;

/** Takes the quoted merchant out of `Estorno de "Merchant" (Merchant)`. */
function extractEstornoCcMerchant(description: string): string | null {
  const m = description.match(ESTORNO_CC_RE);
  return m ? m[1].trim() : null;
}

/** Distance in days between two ISO YYYY-MM-DD dates. */
function daysBetween(a: string, b: string): number {
  const da = Date.parse(a + "T00:00:00Z");
  const db = Date.parse(b + "T00:00:00Z");
  if (Number.isNaN(da) || Number.isNaN(db)) return Infinity;
  return Math.abs((da - db) / 86_400_000);
}

/**
 * From a Pix description `<Prefix> - NAME - ID - BANK/BRANCH/ACCOUNT`, takes
 * everything after the name. That identifies the counterparty uniquely even
 * when the NAME varies (legal vs trade name, as Amazon refunds do).
 *
 * `null` when the description does not start with a known Pix prefix.
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
 * Finds reversal pairs in a list of OFX transactions. Three phases:
 *
 * **Phase 1 — checking-account chargebacks**: description "Estorno - X", paired
 * with the tx whose description is exactly X. Window: +/-7 days.
 *
 * **Phase 2 — Pix refunds**: description "Reembolso recebido pelo Pix - ".
 * Paired by counterparty signature (tax id plus bank details) rather than name
 * — Amazon sends one trade name on the outgoing Pix and the legal name on the
 * refund. Window: +/-30 days.
 *
 * **Phase 3 — Nubank card chargebacks**: description `Estorno de "Merchant"`,
 * paired with the original purchase whose MEMO equals the quoted merchant
 * (case-insensitive). Window: +/-30 days.
 *
 * In every phase amounts must be opposite in sign and equal in magnitude.
 * FIFO pairing when several candidates exist.
 */
export function detectReversalPairs(
  txs: ParsedTransaction[],
): Map<string, ReversalInfo> {
  const result = new Map<string, ReversalInfo>();
  const used = new Set<string>();

  // Phase 1: chargebacks (exact description match).
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
      // The original is an outgoing Pix with the same signature.
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

  // Phase 3: credit-card chargebacks (Nubank).
  // Formato: `Estorno de "Merchant" (Merchant)` (CREDIT, positivo).
  // Pairing: an earlier tx whose normalized description (lowercase, trimmed) is
  // equal to the quoted merchant, opposite sign, same magnitude, +/-30 days.
  for (const e of txs) {
    if (!e.fitid || used.has(e.fitid)) continue;
    const merchant = extractEstornoCcMerchant(e.description);
    if (!merchant) continue;
    const merchantLower = merchant.toLowerCase();
    const estornoAmt = Number(e.amount);
    if (!Number.isFinite(estornoAmt)) continue;

    const candidate = txs.find((t) => {
      if (!t.fitid || used.has(t.fitid) || t.fitid === e.fitid) return false;
      // Original card purchase: description == merchant (case-insensitive trim).
      if (t.description.trim().toLowerCase() !== merchantLower) return false;
      const amt = Number(t.amount);
      if (!Number.isFinite(amt)) return false;
      if (Math.abs(amt + estornoAmt) > 0.01) return false;
      return daysBetween(t.date, e.date) <= 30;
    });

    if (candidate?.fitid) {
      result.set(e.fitid, { role: "estorno", pairFitid: candidate.fitid });
      result.set(candidate.fitid, { role: "estornada", pairFitid: e.fitid });
      used.add(e.fitid);
      used.add(candidate.fitid);
    }
  }

  return result;
}
