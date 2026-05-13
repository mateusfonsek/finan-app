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
 * Actual lib API (ofx-data-extractor@1.5.0):
 *   new Ofx(data: string)  — constructor accepts string content
 *   .getContent()          — returns OfxStructure { OFX: { SIGNONMSGSRSV1, BANKMSGSRSV1, ... } }
 *   .getBankTransferList() — returns StatementTransaction[]
 */
export function parseOfx(content: string): ParsedOfx {
  const ofx = new Ofx(content);
  const structure = ofx.getContent();
  const rawTxs = ofx.getBankTransferList();

  const ofxNode = structure?.OFX ?? {};
  const signon = ofxNode?.SIGNONMSGSRSV1?.SONRS ?? {};
  const bankNode = ofxNode?.BANKMSGSRSV1?.STMTTRNRS?.STMTRS ?? {};
  const bankAcct = bankNode?.BANKACCTFROM ?? {};

  const fi = (signon?.FI ?? {}) as Record<string, unknown>;
  const branchid = (bankAcct?.BRANCHID as string | undefined) ?? null;

  const bankid = (bankAcct?.BANKID as string | undefined) ?? null;
  const acctid = (bankAcct?.ACCTID as string | undefined) ?? null;
  const fid = (fi["FID"] as string | undefined) ?? null;
  const org = (fi["ORG"] as string | undefined) ?? null;

  const bank = detectBank({ fid, org, bankid });
  const displayName = formatDisplayName(bank, branchid, acctid);

  const account: ParsedAccount = {
    bank,
    ofxAcctid: acctid,
    ofxBankid: bankid,
    ofxFid: fid,
    displayName,
  };

  const transactions: ParsedTransaction[] = rawTxs.map((t) => {
    const rawDate = (t.DTPOSTED as string | undefined) ?? "";
    // The lib may return an already-formatted ISO date (YYYY-MM-DD) or raw OFX
    // timestamp (YYYYMMDDHHmmss[tz]). Normalise both.
    const date = isIsoDate(rawDate) ? rawDate : parseOfxDate(rawDate);
    return {
      fitid: (t.FITID as string | undefined) ?? null,
      date,
      amount: String(t.TRNAMT ?? "0"),
      description: String(t.MEMO ?? (t as Record<string, unknown>).NAME ?? ""),
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
): string {
  const bankLabel = bank === "unknown" ? "Conta" : capitalize(bank);
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
