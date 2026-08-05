<script lang="ts">
  import { locale } from "$lib/i18n/locale.svelte";
  import { formatMoney } from "$lib/format/money";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import type { CategorySpend } from "$lib/bindings";

  const t = locale.t;

  type Props = {
    items: CategorySpend[];
    top?: number;
  };

  let { items, top = 5 }: Props = $props();

  /** The uncategorized bucket has no name from the backend — `category_id` is
   *  null and the wording comes from the locale pack. */
  function labelOf(it: CategorySpend): string {
    return it.category_id === null ? t("dashboard.no_category") : it.name;
  }
  let shown = $derived(items.slice(0, top));
  let topValue = $derived(
    shown.reduce((acc, i) => Math.max(acc, Number(i.total) || 0), 0),
  );

  function widthPct(totalStr: string): number {
    if (topValue <= 0) return 0;
    const v = Number(totalStr);
    if (!isFinite(v)) return 0;
    return Math.min(100, (v / topValue) * 100);
  }
</script>

{#if shown.length === 0}
  <EmptyState icon="tags" title={t("dashboard.no_expenses_period")} compact />
{:else}
  <div class="flex flex-col gap-3">
    {#each shown as it}
      <div class="flex flex-col gap-1.5">
        <div class="flex items-baseline gap-2">
          <span
            class="w-2.5 h-2.5 rounded-[3px] shrink-0 translate-y-px"
            style="background: {it.color_token ? `var(${it.color_token})` : 'var(--color-cat-outros)'}"
          ></span>
          <span class="text-callout text-fg font-medium truncate flex-1 min-w-0">{labelOf(it)}</span>
          <span class="text-sub text-fg-muted tabular shrink-0">{formatMoney(it.total)}</span>
        </div>
        <!-- The bar is comparative, not absolute: the largest category is 100%. -->
        <div class="h-1.5 bg-surface-2 rounded-full overflow-hidden">
          <span
            class="block h-full rounded-full transition-[width] duration-[var(--dur-slow)] ease-[var(--ease-snap)]"
            style="width: {widthPct(it.total)}%; background: {it.color_token
              ? `var(${it.color_token})`
              : 'var(--color-accent)'}"
          ></span>
        </div>
      </div>
    {/each}
  </div>
{/if}
