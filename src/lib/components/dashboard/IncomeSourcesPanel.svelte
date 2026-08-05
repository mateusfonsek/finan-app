<script lang="ts">
  import { locale } from "$lib/i18n/locale.svelte";
  import { formatMoney } from "$lib/format/money";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import type { IncomeSource } from "$lib/bindings";

  const t = locale.t;

  type Props = {
    items: IncomeSource[];
    total: string;
    size?: number;
  };

  let { items, total, size = 148 }: Props = $props();

  let hovered = $state<number | null>(null);

  /** Green/teal palette to tell sources apart. Sources have no persistent
   *  color_token (they are derived from the counterparty), so colours are
   *  generated deterministically from the index. */
  const PALETTE = [
    "oklch(70% 0.13 145)",
    "oklch(72% 0.12 165)",
    "oklch(68% 0.13 125)",
    "oklch(74% 0.10 185)",
    "oklch(64% 0.13 105)",
    "oklch(70% 0.10 205)",
    "oklch(60% 0.13 145)",
    "oklch(55% 0.10 155)",
  ];
  function colorFor(i: number): string {
    return PALETTE[i % PALETTE.length];
  }

  let gradient = $derived(buildGradient(items));

  function buildGradient(its: IncomeSource[]): string {
    if (its.length === 0) return "var(--color-surface-2)";
    const stops: string[] = [];
    let acc = 0;
    its.forEach((it, idx) => {
      const start = acc;
      const end = acc + it.percent;
      stops.push(`${colorFor(idx)} ${start.toFixed(2)}% ${end.toFixed(2)}%`);
      acc = end;
    });
    if (acc < 99.999) {
      stops.push(`var(--color-surface-2) ${acc.toFixed(2)}% 100%`);
    }
    return `conic-gradient(from -90deg, ${stops.join(", ")})`;
  }
</script>

{#if items.length === 0}
  <EmptyState icon="handCoins" title={t("dashboard.no_income_period")} compact />
{:else}
  <div class="flex items-center gap-5">
    <div
      class="relative shrink-0 rounded-full grid place-items-center"
      style="width: {size}px; height: {size}px; background: {gradient}"
    >
      <div class="absolute inset-[19px] rounded-full bg-surface border border-border-subtle"></div>
      <div class="relative text-center tabular px-2">
        <div class="text-cap2 text-fg-subtle uppercase tracking-[0.06em]">
          {t("dashboard.income")}
        </div>
        <div class="text-title3 font-semibold mt-0.5 text-fg">
          {formatMoney(total)}
        </div>
      </div>
    </div>

    <ul class="flex-1 flex flex-col min-w-0 max-h-[148px] overflow-y-auto overflow-x-hidden pr-1 -my-0.5">
      {#each items as it, idx (it.key)}
        <li
          role="presentation"
          onmouseenter={() => (hovered = idx)}
          onmouseleave={() => (hovered = null)}
          class="flex items-start gap-2 min-w-0 rounded-[var(--radius-sm)] px-1.5 py-1 -mx-1.5
                 transition-colors duration-[var(--dur-fast)]
                 {hovered === idx ? 'bg-hover' : ''}"
        >
          <span
            class="w-2.5 h-2.5 rounded-[3px] shrink-0 mt-1 transition-transform duration-[var(--dur-fast)] ease-[var(--ease-snap)]
                   {hovered === idx ? 'scale-125' : ''}"
            style="background: {colorFor(idx)}"
          ></span>
          <div class="flex-1 min-w-0 flex flex-col">
            <div class="flex items-baseline gap-1.5">
              <span class="text-sub text-fg font-medium truncate flex-1 min-w-0" title={it.label}>
                {it.label}
              </span>
              {#if it.is_recurring}
                <span
                  class="text-cap2 font-semibold px-1.5 py-px rounded-full shrink-0"
                  style="color: var(--color-pos); background: color-mix(in oklch, var(--color-pos) 15%, transparent);"
                  title={t("dashboard.recurring_title", { months: it.recurring_months })}
                >
                  {t("dashboard.recurring")}
                </span>
              {/if}
              <span class="text-cap tabular text-fg-subtle shrink-0">{it.percent.toFixed(1)}%</span>
            </div>
            <span class="text-foot text-fg-muted tabular">{formatMoney(it.total)}</span>
          </div>
        </li>
      {/each}
    </ul>
  </div>
{/if}
