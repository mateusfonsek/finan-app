<script lang="ts">
  import { formatMoney } from "$lib/format/money";
  import type { CategorySpend } from "$lib/bindings";

  type Props = {
    items: CategorySpend[];
    top?: number;
  };

  let { items, top = 5 }: Props = $props();

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

<div class="flex flex-col gap-2.5">
  {#each shown as it}
    <div class="grid grid-cols-[16px_1fr_auto] gap-2 items-center">
      <span
        class="w-2.5 h-2.5 rounded-sm shrink-0"
        style="background: {it.color_token ? `var(${it.color_token})` : 'var(--color-cat-outros)'}"
      ></span>
      <span class="text-[12px] text-fg font-medium truncate">{it.name}</span>
      <span class="text-[11.5px] text-fg-muted tabular">{formatMoney(it.total)}</span>
      <div class="col-start-2 col-span-2 h-1 bg-surface-2 rounded-full overflow-hidden">
        <span
          class="block h-full rounded-full"
          style="width: {widthPct(it.total)}%; background: {it.color_token ? `var(${it.color_token})` : 'var(--color-accent)'}"
        ></span>
      </div>
    </div>
  {:else}
    <div class="text-fg-faint italic text-[12px]">Sem gastos no período.</div>
  {/each}
</div>
