import type { CalendarEvent } from "$lib/bindings";
import type { IconName } from "$lib/components/ui/icons";

export type BillState = "paid" | "overdue" | "pending";

/** Single source for what a bill state looks like — CalendarGrid, DayDetails
 *  and the legend all render from this, so they cannot silently disagree. */
export const BILL_ICON: Record<BillState, IconName> = {
  paid: "check",
  overdue: "circleAlert",
  pending: "clock",
};

/** The occurrence's due date in the month being viewed, `null` for a rule with
 *  no due day. Day 31 lands on the last day of a short month rather than
 *  vanishing. */
export function dueDateOf(e: CalendarEvent, viewMonth: string): string | null {
  if (e.due_day == null) return null;
  const [y, m] = viewMonth.split("-").map(Number);
  const lastDay = new Date(y, m, 0).getDate();
  const day = Math.min(e.due_day, lastDay);
  return `${viewMonth}-${String(day).padStart(2, "0")}`;
}

/**
 * A bill is paid when the statement shows it OR when the user said so — those
 * are the same fact from two sources, and the calendar must not rank one above
 * the other in what it shows.
 */
export function billState(e: CalendarEvent, viewMonth: string, today: string): BillState {
  if (e.paid_date != null || e.manually_settled) return "paid";
  const due = dueDateOf(e, viewMonth);
  if (due != null && due < today) return "overdue";
  return "pending";
}

/**
 * How many bills the month still owes and how many it already missed.
 *
 * No amount, on purpose: an unpaid occurrence has no value anywhere — a rule
 * carries snippets, a category, a priority and a due day, never a sum.
 * Estimating from the last payment would be inventing a number.
 */
export function monthBillTotals(
  events: CalendarEvent[],
  viewMonth: string,
  today: string,
): { upcoming: number; overdue: number } {
  let upcoming = 0;
  let overdue = 0;
  for (const e of events) {
    if (e.due_day == null) continue;
    const state = billState(e, viewMonth, today);
    if (state === "pending") upcoming += 1;
    else if (state === "overdue") overdue += 1;
  }
  return { upcoming, overdue };
}

/** Whole days from the due date to today. Both read as local midnight so a DST
 *  change cannot shift the count by one. */
export function daysOverdue(dueDate: string, today: string): number {
  const at = (iso: string) => {
    const [y, m, d] = iso.split("-").map(Number);
    return new Date(y, m - 1, d).getTime();
  };
  return Math.max(0, Math.round((at(today) - at(dueDate)) / 86_400_000));
}
