import { commands } from "../bindings";
import type { AccountFreshness } from "../bindings";

function unwrap<T>(result: { status: "ok"; data: T } | { status: "error"; error: string }): T {
  if (result.status === "error") throw new Error(result.error);
  return result.data;
}

/** `app_settings` key for the Dashboard's data-freshness notice. */
const FRESHNESS_ENABLED_KEY = "freshness_enabled";

/** The newest transaction date of each account, or `null` for an account that
 *  has none. Dates only — how far behind that is belongs to the reader's clock. */
export async function dataFreshness(): Promise<AccountFreshness[]> {
  return unwrap(await commands.dataFreshness());
}

/** Unset means on: the notice only reads dates already in the database, so
 *  there is nothing to opt into — unlike automatic import, which touches disk. */
export async function freshnessEnabled(): Promise<boolean> {
  return unwrap(await commands.getAppSetting(FRESHNESS_ENABLED_KEY)) !== "0";
}

export async function setFreshnessEnabled(enabled: boolean): Promise<void> {
  unwrap(await commands.setAppSetting(FRESHNESS_ENABLED_KEY, enabled ? "1" : "0"));
}
