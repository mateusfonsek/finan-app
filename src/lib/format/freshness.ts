import type { AccountFreshness } from "$lib/bindings";

export type DatedAccount = {
  accountId: number;
  name: string;
  latestDate: string;
};

export type Freshness = {
  /** Date of the account furthest behind — the ceiling of any estimate the
   *  screen shows. */
  latestDate: string;
  /** Whole days from `latestDate` to today. Zero when up to date or ahead. */
  daysBehind: number;
  /** Only accounts that carry data, furthest behind first. */
  accounts: DatedAccount[];
};

/**
 * Whole calendar days from `isoDate` to `today`, both read as local midnight so
 * a DST change cannot shift the count by one. Never negative: a card statement
 * may carry postings dated after today, and that is not a gap in the data.
 */
function daysBehind(isoDate: string, today: Date): number {
  const [y, m, d] = isoDate.split("-").map(Number);
  const from = new Date(y, m - 1, d).getTime();
  const to = new Date(today.getFullYear(), today.getMonth(), today.getDate()).getTime();
  return Math.max(0, Math.round((to - from) / 86_400_000));
}

/**
 * Past this gap an account stops being late and starts being over: a statement
 * imported once and never again is not a reminder, and letting it own the
 * headline would freeze the line on a number that says nothing about the data
 * the screen is actually showing.
 */
const ABANDONED_AFTER_DAYS = 60;

/**
 * Two kinds of account are not data. One has no transaction at all — an import
 * that created the account and then failed. The other stopped so long ago it
 * reads as abandoned. With neither left there is nothing to say, and the caller
 * shows nothing rather than nagging someone who already moved on.
 */
export function computeFreshness(rows: AccountFreshness[], today: Date): Freshness | null {
  const accounts: DatedAccount[] = rows
    .filter((r) => r.latest_date !== null)
    .map((r) => ({ accountId: r.account_id, name: r.name, latestDate: r.latest_date! }))
    .filter((a) => daysBehind(a.latestDate, today) <= ABANDONED_AFTER_DAYS)
    .sort((a, b) => a.latestDate.localeCompare(b.latestDate));

  if (accounts.length === 0) return null;

  const latestDate = accounts[0].latestDate;
  return { latestDate, daysBehind: daysBehind(latestDate, today), accounts };
}
