/**
 * Where the month's first day sits in a week that may not start on Sunday.
 *
 * `weekdayOrder` is the contract the whole calendar rests on: `weekdays_short`
 * is indexed by `Date.getDay()` and therefore starts at Sunday in EVERY locale
 * pack, whatever day that locale's week begins on. Letting a pack reorder the
 * array instead would desync it from the grid with nothing to catch it.
 */

/** Blank cells before day 1. `startWeekday` and `firstDayOfWeek` are both
 *  `Date.getDay()` values (0 = Sunday). */
export function leadingBlanks(startWeekday: number, firstDayOfWeek: number): number {
  return (startWeekday - firstDayOfWeek + 7) % 7;
}


/** `weekdays_short` indices in display order, so the header reads Mon…Sun where
 *  the locale's week starts on Monday. */
export function weekdayOrder(firstDayOfWeek: number): number[] {
  return Array.from({ length: 7 }, (_, i) => (firstDayOfWeek + i) % 7);
}
