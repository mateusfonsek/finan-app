<script lang="ts">
  type Props = {
    month: string | null;
    onChange: (m: string | null) => void;
    showClear?: boolean;
  };

  let { month, onChange, showClear = true }: Props = $props();

  function monthLabel(m: string | null): string {
    if (!m) return "Todos os meses";
    const [y, mo] = m.split("-");
    const names = ["Jan", "Fev", "Mar", "Abr", "Mai", "Jun", "Jul", "Ago", "Set", "Out", "Nov", "Dez"];
    return `${names[Number(mo) - 1]}/${y.slice(-2)}`;
  }

  function shiftMonth(m: string | null, delta: number): string | null {
    if (!m) {
      const now = new Date();
      now.setMonth(now.getMonth() + delta);
      const y = now.getFullYear();
      const mo = String(now.getMonth() + 1).padStart(2, "0");
      return `${y}-${mo}`;
    }
    const [y, mo] = m.split("-").map((s) => Number(s));
    const d = new Date(y, mo - 1 + delta, 1);
    return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}`;
  }
</script>

<div class="inline-flex items-center gap-2">
  <div class="inline-flex items-center gap-px rounded-md border border-border bg-surface-2">
    <button
      type="button"
      class="px-2 py-1 text-fg-muted hover:bg-hover rounded-l-md"
      onclick={() => onChange(shiftMonth(month, -1))}
      aria-label="Mês anterior"
    >
      ‹
    </button>
    <span class="px-2.5 text-[12px] font-medium tabular min-w-[88px] text-center">
      {monthLabel(month)}
    </span>
    <button
      type="button"
      class="px-2 py-1 text-fg-muted hover:bg-hover rounded-r-md"
      onclick={() => onChange(shiftMonth(month, +1))}
      aria-label="Próximo mês"
    >
      ›
    </button>
  </div>

  {#if showClear && month}
    <button
      type="button"
      onclick={() => onChange(null)}
      class="text-[11px] text-fg-faint hover:text-fg-muted underline-offset-2 hover:underline"
    >
      Todos os meses
    </button>
  {/if}
</div>
