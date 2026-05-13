<script lang="ts">
  import type { Category } from "$lib/bindings";

  type Props = {
    categories: Category[];
    /** YYYY-MM or null */
    month: string | null;
    categoryId: number | null;
    onMonthChange: (m: string | null) => void;
    onCategoryChange: (id: number | null) => void;
  };

  let { categories, month, categoryId, onMonthChange, onCategoryChange }: Props = $props();

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

  let currentCategory = $derived(categories.find((c) => c.id === categoryId));
</script>

<div class="flex items-center gap-2 flex-wrap">
  <div class="inline-flex items-center gap-px rounded-md border border-border bg-surface-2">
    <button
      type="button"
      class="px-2 py-1 text-fg-muted hover:bg-hover rounded-l-md"
      onclick={() => onMonthChange(shiftMonth(month, -1))}
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
      onclick={() => onMonthChange(shiftMonth(month, +1))}
      aria-label="Próximo mês"
    >
      ›
    </button>
  </div>

  {#if month}
    <button
      type="button"
      onclick={() => onMonthChange(null)}
      class="text-[11px] text-fg-faint hover:text-fg-muted underline-offset-2 hover:underline"
    >
      Todos os meses
    </button>
  {/if}

  <select
    value={categoryId === null ? "" : String(categoryId)}
    onchange={(e) => {
      const v = (e.currentTarget as HTMLSelectElement).value;
      onCategoryChange(v === "" ? null : Number(v));
    }}
    class="text-[12px] rounded-md border border-border bg-surface-2 px-2 py-1 text-fg"
  >
    <option value="">Todas as categorias</option>
    {#each categories as c}
      <option value={String(c.id)}>{c.name}</option>
    {/each}
  </select>

  {#if currentCategory}
    <span class="text-[11px] text-fg-faint">· {currentCategory.kind}</span>
  {/if}
</div>
