/**
 * Sort state for a table.
 *
 * Lives here so two tables cannot invent different rules for the same gesture:
 * clicking a new column adopts its most useful direction, clicking the active
 * one inverts. Same promise in every table.
 */
export type SortDir = "asc" | "desc";

export type Sort<K extends string> = {
  readonly key: K;
  readonly dir: SortDir;
  /** Inverts when already active, otherwise adopts the column's first direction. */
  toggle(key: K): void;
  /** `aria-sort` value for the `<th>`. */
  aria(key: K): "ascending" | "descending" | "none";
  /** The direction the NEXT click on this column will produce. */
  next(key: K): SortDir;
  /** `1` ascending, `-1` descending — multiplies the comparator. */
  readonly sign: 1 | -1;
};

/**
 * @param firstDir first-click direction per column. Not one global rule
 *   because the column decides what is useful: dates start newest, names start
 *   at A. Finder behaves the same way.
 * @param initial the column and direction the table opens with — usually
 *   mirroring the backend's order, so nothing reorders on load.
 */
export function createSort<K extends string>(
  firstDir: Record<K, SortDir>,
  initial: { key: K; dir?: SortDir },
): Sort<K> {
  let key = $state<K>(initial.key);
  let dir = $state<SortDir>(initial.dir ?? firstDir[initial.key]);

  return {
    get key() {
      return key;
    },
    get dir() {
      return dir;
    },
    get sign() {
      return dir === "asc" ? 1 : -1;
    },
    toggle(next: K) {
      if (key === next) {
        dir = dir === "asc" ? "desc" : "asc";
      } else {
        key = next;
        dir = firstDir[next];
      }
    },
    aria(k: K) {
      if (k !== key) return "none";
      return dir === "asc" ? "ascending" : "descending";
    },
    next(k: K) {
      if (k !== key) return firstDir[k];
      return dir === "asc" ? "desc" : "asc";
    },
  };
}

/**
 * Text comparison in the user's language — "Água" must land next to "Agua",
 * not at the end of the alphabet.
 */
export function compareText(a: string, b: string, localeCode: string): number {
  return a.localeCompare(b, localeCode, { sensitivity: "base", numeric: true });
}

/**
 * Tie-break for columns that may have no value (a blank due date).
 *
 * Absence is not "smaller": it goes LAST in both directions, otherwise
 * inverting would fill the top of the table with dashes.
 *
 * Returns the comparator's FINAL result (do not multiply by `sign`), or `null`
 * when both have values, leaving the comparison to the caller.
 */
export function nullsLast(a: unknown, b: unknown): number | null {
  const aEmpty = a === null || a === undefined;
  const bEmpty = b === null || b === undefined;
  if (aEmpty && bEmpty) return 0;
  if (aEmpty) return 1;
  if (bEmpty) return -1;
  return null;
}
