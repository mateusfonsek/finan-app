<script lang="ts">
  import { push, router } from "svelte-spa-router";
  import Icon from "$lib/components/ui/Icon.svelte";
  import { Button } from "$lib/components/ui/button";
  import { locale } from "$lib/i18n/locale.svelte";
  import { watch } from "$lib/stores/watch.svelte";
  import type { Freshness } from "$lib/format/freshness";

  const t = locale.t;

  let { freshness }: { freshness: Freshness } = $props();

  const IMPORT_ROUTE = "/import";
  const SETTINGS_ROUTE = "/settings";

  /** Day and month — plus the year, but only when it is not the current one.
   *  Without that, data a year stale reads exactly like data from last month. */
  function formatDate(iso: string): string {
    const [y, m, d] = iso.split("-").map(Number);
    return new Date(y, m - 1, d).toLocaleDateString(locale.dateLocale, {
      day: "numeric",
      month: "short",
      year: y === new Date().getFullYear() ? undefined : "numeric",
    });
  }

  let gap = $derived(
    freshness.daysBehind === 0
      ? t("freshness.current")
      : freshness.daysBehind === 1
        ? t("freshness.behind_one")
        : t("freshness.behind_many", { n: freshness.daysBehind }),
  );

  /** More than one account carrying data means the headline is one account's
   *  date standing for all of them — which accounts, and how far apart they
   *  are, is then worth being able to open. */
  let expandable = $derived(freshness.accounts.length > 1);

  /** The same gesture as the discovery notification: Import is told which file
   *  to open through the store, because `push("/import")` while already on
   *  `/import` fires no `hashchange` and Import would never remount. */
  function review() {
    const first = watch.discoveries[0];
    if (!first) return;
    watch.requestOpen(first);
    if (router.location !== IMPORT_ROUTE) void push(IMPORT_ROUTE);
  }
</script>

{#snippet line()}
  <span class="text-sub truncate">
    <span class="text-fg font-medium">
      {t("freshness.up_to", { date: formatDate(freshness.latestDate) })}
    </span>
    <span class="text-fg-muted"> · {gap}</span>
  </span>
{/snippet}

<!-- Status, not a warning: no colour, no icon that alarms. The number of days
     is the whole argument, and the action sits next to the fact it answers. -->
<div class="card px-4 py-2.5 flex items-start justify-between gap-3">
  {#if expandable}
    <details class="group min-w-0">
      <summary
        title={t("freshness.by_account")}
        class="list-none marker:hidden flex items-center gap-2 text-fg-muted
               hover:text-fg transition-colors duration-[var(--dur-fast)]"
      >
        <Icon
          name="chevronRight"
          size={12}
          stroke={2.2}
          class="text-fg-faint transition-transform duration-[var(--dur)] ease-[var(--ease-snap)] group-open:rotate-90"
        />
        {@render line()}
      </summary>
      <ul class="pt-2 pl-5 flex flex-col gap-1 max-w-sm">
        {#each freshness.accounts as a (a.accountId)}
          <li class="text-foot text-fg-subtle flex items-baseline justify-between gap-3">
            <span class="truncate">{a.name}</span>
            <span class="tabular shrink-0">
              {t("freshness.account_up_to", { date: formatDate(a.latestDate) })}
            </span>
          </li>
        {/each}
      </ul>
    </details>
  {:else}
    <div class="flex items-center gap-2 min-w-0">
      <Icon name="clock" size={12} class="text-fg-faint" />
      {@render line()}
    </div>
  {/if}

  {#if watch.pendingCount > 0}
    <Button size="sm" variant="outline" onclick={review}>
      {watch.pendingCount === 1
        ? t("freshness.review_one")
        : t("freshness.review_many", { n: watch.pendingCount })}
    </Button>
  {:else if !watch.enabled}
    <!-- Quiet on purpose: an offer, not a call to action. Nothing is stale
         about a user who imports by hand. -->
    <button
      type="button"
      onclick={() => push(SETTINGS_ROUTE)}
      class="text-foot text-fg-subtle hover:text-fg shrink-0 mt-px
             transition-colors duration-[var(--dur-fast)]"
    >
      {t("freshness.enable_watch")}
    </button>
  {/if}
</div>
