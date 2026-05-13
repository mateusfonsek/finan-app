export interface ParsedOfx {
  account: ParsedAccount;
  transactions: ParsedTransaction[];
  summary: ParsedSummary;
}

export interface ParsedAccount {
  /** Banco normalizado: 'itau' | 'nubank' | 'bradesco' | 'unknown' */
  bank: string;
  ofxAcctid: string | null;
  ofxBankid: string | null;
  ofxFid: string | null;
  displayName: string;
}

export interface ParsedTransaction {
  fitid: string | null;
  /** ISO 8601 YYYY-MM-DD */
  date: string;
  /** Decimal como string (ex: "-123.45") */
  amount: string;
  description: string;
}

export interface ParsedSummary {
  totalIn: string;
  totalOut: string;
  net: string;
  earliest: string | null;
  latest: string | null;
}
