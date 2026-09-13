import type { McpToolState } from "$lib/bindings";

/** Months the agent may reach back. `0` is "no limit", and comes last because
 *  it is the widest, not because it is the least. */
export const WINDOW_OPTIONS = [3, 6, 12, 24, 0];

/** Reads and writes are drawn apart: turning on a write is a different kind of
 *  decision than turning on a read, and a single flat list hides that. */
export function splitTools(tools: McpToolState[]): {
  read: McpToolState[];
  write: McpToolState[];
} {
  return {
    read: tools.filter((t) => !t.write),
    write: tools.filter((t) => t.write),
  };
}

/** The oldest month inside the window, as `YYYY-MM`. The current month counts
 *  as one of them — "3 months" reaching back four would be a lie. */
export function cutoffLabel(months: number, today = new Date()): string | null {
  if (months === 0) return null;
  const total = today.getFullYear() * 12 + today.getMonth() - (months - 1);
  const year = Math.floor(total / 12);
  const month = total - year * 12;
  return `${year}-${String(month + 1).padStart(2, "0")}`;
}
