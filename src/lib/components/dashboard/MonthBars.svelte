<script lang="ts">
  import { formatMoney } from "$lib/format/money";
  import type { MonthSummary } from "$lib/bindings";

  type Props = {
    months: MonthSummary[];
  };

  let { months }: Props = $props();

  function shortLabel(yyyymm: string): string {
    const [y, mo] = yyyymm.split("-");
    const names = ["jan", "fev", "mar", "abr", "mai", "jun", "jul", "ago", "set", "out", "nov", "dez"];
    return `${names[Number(mo) - 1]}/${y.slice(-2)}`;
  }

  let maxValue = $derived(
    months.reduce((acc, m) => {
      const i = Number(m.income);
      const e = Number(m.expense);
      return Math.max(acc, isFinite(i) ? i : 0, isFinite(e) ? e : 0);
    }, 0),
  );

  function pct(amountStr: string): number {
    if (maxValue <= 0) return 0;
    const v = Number(amountStr);
    if (!isFinite(v) || v <= 0) return 0;
    return Math.min(100, (v / maxValue) * 100);
  }
</script>

<div class="grid grid-cols-12 gap-1.5 items-end h-[140px] pt-2">
  {#each months as m (m.month)}
    <div class="flex flex-col items-center gap-1.5 h-full">
      <div
        class="w-full flex-1 flex flex-col-reverse rounded-t overflow-hidden bg-surface-2"
        title={`${m.month} · in: ${formatMoney(m.income)} · out: ${formatMoney(m.expense)}`}
      >
        <span class="w-full bg-pos" style="height: {pct(m.income)}%"></span>
        <span class="w-full bg-neg" style="height: {pct(m.expense)}%; opacity: 0.6;"></span>
      </div>
      <span class="text-[9.5px] text-fg-faint tabular">{shortLabel(m.month)}</span>
    </div>
  {/each}
  {#if months.length === 0}
    <div class="col-span-12 text-center text-fg-faint italic py-8">
      Sem dados nos últimos 12 meses.
    </div>
  {/if}
</div>
