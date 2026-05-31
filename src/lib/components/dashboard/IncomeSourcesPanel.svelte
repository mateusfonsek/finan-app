<script lang="ts">
  import { formatMoney } from "$lib/format/money";
  import type { IncomeSource } from "$lib/bindings";

  type Props = {
    items: IncomeSource[];
    total: string;
    size?: number;
  };

  let { items, total, size = 156 }: Props = $props();

  /** Paleta de tons verde/azul-esverdeado pra distinguir fontes visualmente.
   *  Fontes não têm color_token persistente (são derivadas da contraparte),
   *  então geramos as cores deterministicamente pelo índice. */
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
      <div class="text-[10px] uppercase tracking-wider text-fg-faint">Renda</div>
      <div class="text-[18px] font-semibold mt-px" style="font-family: var(--font-display)">
        {formatMoney(total)}
      </div>
    </div>
  </div>

  <ul class="flex-1 flex flex-col gap-2 text-[11.5px] min-w-0 max-h-[156px] overflow-y-auto pr-1">
    {#each items as it, idx (it.key)}
      <li class="flex items-start gap-2 min-w-0">
        <span
          class="w-2.5 h-2.5 rounded-sm shrink-0 mt-1"
          style="background: {colorFor(idx)}"
        ></span>
        <div class="flex-1 min-w-0 flex flex-col gap-px">
          <div class="flex items-baseline gap-2">
            <span class="text-fg font-medium truncate flex-1 min-w-0" title={it.label}>
              {it.label}
            </span>
            {#if it.is_recurring}
              <span
                class="text-[9px] uppercase tracking-wider font-semibold px-1 py-px rounded shrink-0"
                style="color: var(--color-pos); border: 1px solid var(--color-pos); opacity: 0.85;"
                title={`Recorrente · ${it.recurring_months} meses distintos`}
              >
                recorrente
              </span>
            {/if}
            <span class="text-[10.5px] tabular text-fg-faint shrink-0">
              {it.percent.toFixed(1)}%
            </span>
          </div>
          <span class="text-fg-muted tabular text-[11px]">{formatMoney(it.total)}</span>
        </div>
      </li>
    {:else}
      <li class="text-fg-faint italic">Sem entradas no período.</li>
    {/each}
  </ul>
</div>
