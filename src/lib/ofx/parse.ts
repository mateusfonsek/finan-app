import { Ofx } from "ofx-data-extractor";
import type {
  ParsedAccount,
  ParsedOfx,
  ParsedSummary,
  ParsedTransaction,
} from "./types";

/**
 * Parse raw OFX text into our normalized shape. Throws on malformed input.
 *
 * Suporta dois layouts:
 *   - Conta corrente: BANKMSGSRSV1 / STMTTRNRS / STMTRS / BANKACCTFROM
 *   - Cartão de crédito: CREDITCARDMSGSRSV1 / CCSTMTTRNRS / CCSTMTRS / CCACCTFROM
 *
 * Lib `ofx-data-extractor@1.5.0`:
 *   - `.getBankTransferList()`        — tx do BANK (conta corrente)
 *   - `.getCreditCardTransferList()`  — tx do CC (cartão)
 */
export function parseOfx(content: string): ParsedOfx {
  const ofx = new Ofx(content);
  const structure = ofx.getContent();

  const ofxNode = structure?.OFX ?? {};
  const signon = ofxNode?.SIGNONMSGSRSV1?.SONRS ?? {};
  const fi = (signon?.FI ?? {}) as Record<string, unknown>;
  const fid = (fi["FID"] as string | undefined) ?? null;
  const org = (fi["ORG"] as string | undefined) ?? null;

  // Detecta CC pela presença do nó CREDITCARDMSGSRSV1.
  // Quando ambos existirem por algum motivo, BANK tem prioridade
  // (caso esperado: OFXs do Nubank vêm com APENAS um dos dois).
  const ccNode =
    (ofxNode as Record<string, unknown>)?.CREDITCARDMSGSRSV1 as
      | Record<string, unknown>
      | undefined;
  const bankNode = ofxNode?.BANKMSGSRSV1?.STMTTRNRS?.STMTRS ?? {};
  const hasBank = !!bankNode?.BANKACCTFROM;
  const isCreditCard = !hasBank && !!ccNode;

  let acctid: string | null = null;
  let branchid: string | null = null;
  let bankid: string | null = null;
  let rawTxs: Array<Record<string, unknown>> = [];

  if (isCreditCard) {
    const ccStmt =
      ((ccNode as { CCSTMTTRNRS?: { CCSTMTRS?: Record<string, unknown> } })
        ?.CCSTMTTRNRS?.CCSTMTRS as Record<string, unknown> | undefined) ?? {};
    const ccAcct =
      (ccStmt?.CCACCTFROM as Record<string, unknown> | undefined) ?? {};
    acctid = (ccAcct["ACCTID"] as string | undefined) ?? null;
    // CC não tem branchid nem bankid em CCACCTFROM.
    try {
      rawTxs = ofx.getCreditCardTransferList() as Array<Record<string, unknown>>;
    } catch {
      rawTxs = [];
    }
  } else {
    const bankAcct = (bankNode?.BANKACCTFROM ?? {}) as Record<string, unknown>;
    branchid = (bankAcct["BRANCHID"] as string | undefined) ?? null;
    bankid = (bankAcct["BANKID"] as string | undefined) ?? null;
    acctid = (bankAcct["ACCTID"] as string | undefined) ?? null;
    rawTxs = ofx.getBankTransferList() as Array<Record<string, unknown>>;
  }

  const bank = detectBank({ fid, org, bankid });
  const type: "checking" | "credit_card" = isCreditCard ? "credit_card" : "checking";
  const displayName = formatDisplayName(bank, branchid, acctid, type);

  const account: ParsedAccount = {
    bank,
    ofxAcctid: acctid,
    ofxBankid: bankid,
    ofxFid: fid,
    displayName,
    type,
  };

  // Nubank (e outros bancos BR) reusa o mesmo FITID em transações distintas
  // do mesmo extrato — caso real: IOF + compra principal compartilham FITID,
  // idem pro par estorno-IOF + estorno-compra. Mantemos o FITID original na
  // primeira ocorrência e adicionamos sufixo "#2", "#3", ... nas seguintes
  // pra: (a) não quebrar o `{#each ... (fitid)}` keyed na UI, (b) não violar
  // a UNIQUE(account_id, ofx_fitid) do banco, (c) permitir seleção individual.
  // A ordenação é a do arquivo OFX, estável entre re-imports do mesmo extrato.
  const seenFitid = new Map<string, number>();
  const transactions: ParsedTransaction[] = rawTxs.map((t) => {
    const rawDate = (t.DTPOSTED as string | undefined) ?? "";
    const date = isIsoDate(rawDate) ? rawDate : parseOfxDate(rawDate);
    const rawFitid = (t.FITID as string | undefined) ?? null;
    let fitid = rawFitid;
    if (rawFitid) {
      const count = seenFitid.get(rawFitid) ?? 0;
      if (count > 0) fitid = `${rawFitid}#${count + 1}`;
      seenFitid.set(rawFitid, count + 1);
    }
    return {
      fitid,
      date,
      amount: String(t.TRNAMT ?? "0"),
      description: String(t.MEMO ?? t.NAME ?? ""),
    };
  });

  const summary = computeSummary(transactions);

  return { account, transactions, summary };
}

/** Returns true if the string is already in YYYY-MM-DD format. */
function isIsoDate(raw: string): boolean {
  return /^\d{4}-\d{2}-\d{2}/.test(raw);
}

/** Parse raw OFX timestamp (YYYYMMDDHHmmss...) → YYYY-MM-DD. */
function parseOfxDate(raw: string): string {
  const m = raw.match(/^(\d{4})(\d{2})(\d{2})/);
  if (!m) return "";
  return `${m[1]}-${m[2]}-${m[3]}`;
}

function detectBank(meta: {
  fid: string | null;
  org: string | null;
  bankid: string | null;
}): string {
  const fingerprint = [meta.fid, meta.org, meta.bankid]
    .filter(Boolean)
    .join(" ")
    .toLowerCase();
  if (/itau|341/.test(fingerprint)) return "itau";
  if (/bradesco|237/.test(fingerprint)) return "bradesco";
  if (/nubank|260|nu pagamentos|nu_pagamentos/.test(fingerprint)) return "nubank";
  if (/santander|033/.test(fingerprint)) return "santander";
  if (/inter|077/.test(fingerprint)) return "inter";
  if (/c6\b|c6 bank|336/.test(fingerprint)) return "c6";
  return "unknown";
}

function formatDisplayName(
  bank: string,
  branchid: string | null,
  acctid: string | null,
  type: "checking" | "credit_card",
): string {
  const bankLabel = bank === "unknown" ? "Conta" : capitalize(bank);
  if (type === "credit_card") {
    // CC só tem ACCTID (UUID longo). Mostra "Banco · cartão · final XXXX".
    const tail = acctid ? acctid.slice(-4) : null;
    const parts = [bankLabel, "cartão"];
    if (tail) parts.push(`final ${tail}`);
    return parts.join(" · ");
  }
  const parts = [bankLabel];
  if (branchid) parts.push(`ag ${branchid}`);
  if (acctid) parts.push(`cc ${acctid}`);
  return parts.join(" · ");
}

function capitalize(s: string): string {
  return s ? s.charAt(0).toUpperCase() + s.slice(1) : s;
}

function computeSummary(txs: ParsedTransaction[]): ParsedSummary {
  let totalIn = 0;
  let totalOut = 0;
  let earliest: string | null = null;
  let latest: string | null = null;
  for (const t of txs) {
    const n = Number(t.amount);
    if (Number.isFinite(n)) {
      if (n >= 0) totalIn += n;
      else totalOut += -n;
    }
    if (t.date) {
      if (earliest === null || t.date < earliest) earliest = t.date;
      if (latest === null || t.date > latest) latest = t.date;
    }
  }
  return {
    totalIn: totalIn.toFixed(2),
    totalOut: totalOut.toFixed(2),
    net: (totalIn - totalOut).toFixed(2),
    earliest,
    latest,
  };
}
