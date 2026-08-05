<script lang="ts">
  import { locale } from "$lib/i18n/locale.svelte";
  import { formatMoney } from "$lib/format/money";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import type { CategorySpend } from "$lib/bindings";

  const t = locale.t;

  type Props = {
    items: CategorySpend[];
    total: string;
    size?: number;
  };

  let { items, total, size = 148 }: Props = $props();

  /** Fatia sob o cursor: destacar no anel E na legenda ao mesmo tempo mostra
   *  que os dois são a mesma coisa vista de dois jeitos. */
  let hovered = $state<number | null>(null);

  let gradient = $derived(buildGradient(items));

  function buildGradient(list: CategorySpend[]): string {
    if (list.length === 0) return "var(--color-surface-2)";
    const stops: string[] = [];
    let acc = 0;
    for (const it of list) {
      const start = acc;
      const end = acc + it.percent;
      const color = it.color_token ? `var(${it.color_token})` : "var(--color-cat-outros)";
      stops.push(`${color} ${start.toFixed(2)}% ${end.toFixed(2)}%`);
      acc = end;
    }
    if (acc < 99.999) {
      stops.push(`var(--color-surface-2) ${acc.toFixed(2)}% 100%`);
    }
    return `conic-gradient(from -90deg, ${stops.join(", ")})`;
  }

  function colorOf(it: CategorySpend): string {
    return it.color_token ? `var(${it.color_token})` : "var(--color-cat-outros)";
  }
</script>

{#if items.length === 0}
  <EmptyState icon="chartPie" title={t("dashboard.no_expenses_period")} compact />
{:else}
  <div class="flex items-center gap-5">
    <div
      class="relative shrink-0 rounded-full grid place-items-center transition-transform duration-[var(--dur)] ease-[var(--ease-snap)]"
      style="width: {size}px; height: {size}px; background: {gradient}"
    >
      <div class="absolute inset-[19px] rounded-full bg-surface border border-border-subtle"></div>
      <div class="relative text-center tabular px-2">
        <div class="text-cap2 text-fg-subtle uppercase tracking-[0.06em]">
          {t("dashboard.expenses")}
        </div>
        <div class="text-title3 font-semibold mt-0.5 text-fg">
          {formatMoney(total)}
        </div>
      </div>
    </div>

    <ul class="flex-1 flex flex-col min-w-0 max-h-[148px] overflow-y-auto overflow-x-hidden pr-1 -my-0.5">
      {#each items as it, i}
        <li
          role="presentation"
          onmouseenter={() => (hovered = i)}
          onmouseleave={() => (hovered = null)}
          class="flex items-start gap-2 min-w-0 rounded-[var(--radius-sm)] px-1.5 py-1 -mx-1.5
                 transition-colors duration-[var(--dur-fast)]
                 {hovered === i ? 'bg-hover' : ''}"
        >
          <span
            class="w-2.5 h-2.5 rounded-[3px] shrink-0 mt-1 transition-transform duration-[var(--dur-fast)] ease-[var(--ease-snap)]
                   {hovered === i ? 'scale-125' : ''}"
            style="background: {colorOf(it)}"
          ></span>
          <div class="flex-1 min-w-0 flex flex-col">
            <div class="flex items-baseline gap-2">
              <span class="text-sub text-fg font-medium truncate flex-1 min-w-0" title={it.name}>
                {it.name}
              </span>
              <span class="text-cap tabular text-fg-subtle shrink-0">{it.percent.toFixed(1)}%</span>
            </div>
            <span class="text-foot text-fg-muted tabular">{formatMoney(it.total)}</span>
          </div>
        </li>
      {/each}
    </ul>
  </div>
{/if}
