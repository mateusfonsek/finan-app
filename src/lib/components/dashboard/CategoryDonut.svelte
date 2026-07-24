<script lang="ts">
  import { locale } from "$lib/i18n/locale.svelte";

  const t = locale.t;
  import { formatMoney } from "$lib/format/money";
  import type { CategorySpend } from "$lib/bindings";

  type Props = {
    items: CategorySpend[];
    total: string;
    size?: number;
  };

  let { items, total, size = 156 }: Props = $props();

  let gradient = $derived(buildGradient(items));

  function buildGradient(items: CategorySpend[]): string {
    if (items.length === 0) return "var(--color-surface-2)";
    const stops: string[] = [];
    let acc = 0;
    for (const it of items) {
      const start = acc;
      const end = acc + it.percent;
      const color = it.color_token ? `var(${it.color_token})` : "var(--color-cat-outros)";
      stops.push(`${color} ${start.toFixed(2)}% ${end.toFixed(2)}%`);
      acc = end;
    }
    if (acc < 99.999) {
      stops.push(`var(--color-surface-2) ${acc.toFixed(2)}% 100%`);
    }
    return `conic-gradient(${stops.join(", ")})`;
  }
</script>

<div class="flex items-center gap-5">
  <div
    class="relative shrink-0 rounded-full grid place-items-center"
    style="width: {size}px; height: {size}px; background: {gradient}"
  >
    <div class="absolute inset-[18px] rounded-full bg-surface border border-border-subtle"></div>
    <div class="relative text-center tabular">
      <div class="text-[10px] uppercase tracking-wider text-fg-faint">{t("dashboard.expenses")}</div>
      <div class="text-[18px] font-semibold mt-px" style="font-family: var(--font-display)">
        {formatMoney(total)}
      </div>
    </div>
  </div>

  <ul class="flex-1 flex flex-col gap-2 text-[11.5px] min-w-0 max-h-[156px] overflow-y-auto pr-1">
    {#each items as it}
      <li class="flex items-start gap-2 min-w-0">
        <span
          class="w-2.5 h-2.5 rounded-sm shrink-0 mt-1"
          style="background: {it.color_token ? `var(${it.color_token})` : 'var(--color-cat-outros)'}"
        ></span>
        <div class="flex-1 min-w-0 flex flex-col gap-px">
          <div class="flex items-baseline gap-2">
            <span class="text-fg font-medium truncate flex-1 min-w-0" title={it.name}>{it.name}</span>
            <span class="text-[10.5px] tabular text-fg-faint shrink-0">{it.percent.toFixed(1)}%</span>
          </div>
          <span class="text-fg-muted tabular text-[11px]">{formatMoney(it.total)}</span>
        </div>
      </li>
    {/each}
    {#if items.length === 0}
      <li class="text-fg-faint italic">{t("dashboard.no_expenses_period")}</li>
    {/if}
  </ul>
</div>
