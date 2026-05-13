<script lang="ts">
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
      <div class="text-[10px] uppercase tracking-wider text-fg-faint">Gastos</div>
      <div class="text-[18px] font-semibold mt-px" style="font-family: var(--font-display)">
        {formatMoney(total)}
      </div>
    </div>
  </div>

  <ul class="flex-1 flex flex-col gap-1 text-[11.5px]">
    {#each items.slice(0, 8) as it}
      <li class="grid grid-cols-[10px_1fr_auto_44px] gap-2 items-center text-fg-muted">
        <span
          class="w-2.5 h-2.5 rounded-sm shrink-0"
          style="background: {it.color_token ? `var(${it.color_token})` : 'var(--color-cat-outros)'}"
        ></span>
        <span class="text-fg truncate">{it.name}</span>
        <span class="tabular">{formatMoney(it.total)}</span>
        <span class="tabular text-fg-faint text-right">{it.percent.toFixed(1)}%</span>
      </li>
    {/each}
    {#if items.length === 0}
      <li class="text-fg-faint italic">Sem gastos no período.</li>
    {/if}
  </ul>
</div>
