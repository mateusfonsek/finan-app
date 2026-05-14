<script lang="ts">
  import { onMount } from "svelte";

  type Props = {
    /** "YYYY-MM" = mês específico, "YYYY" = ano inteiro, null = todos os períodos */
    month: string | null;
    onChange: (m: string | null) => void;
  };

  let { month, onChange }: Props = $props();

  const MONTH_NAMES = [
    "Jan", "Fev", "Mar", "Abr", "Mai", "Jun",
    "Jul", "Ago", "Set", "Out", "Nov", "Dez",
  ];
  const MONTH_NAMES_FULL = [
    "Janeiro", "Fevereiro", "Março", "Abril", "Maio", "Junho",
    "Julho", "Agosto", "Setembro", "Outubro", "Novembro", "Dezembro",
  ];

  function currentYear(): number {
    return new Date().getFullYear();
  }
  function currentMonth(): number {
    return new Date().getMonth() + 1;
  }
  function parseYear(m: string | null): number | null {
    if (!m) return null;
    return Number(m.slice(0, 4));
  }
  function parseMonth(m: string | null): number | null {
    if (!m || m.length !== 7) return null;
    return Number(m.slice(5, 7));
  }

  let mode = $derived<"month" | "year" | "all">(
    !month ? "all" : month.length === 7 ? "month" : "year",
  );

  let label = $derived(computeLabel(month, mode));

  function computeLabel(m: string | null, mo: "month" | "year" | "all"): string {
    if (mo === "all") return "Todos";
    if (mo === "month" && m) {
      return `${MONTH_NAMES[Number(m.slice(5, 7)) - 1]}/${m.slice(0, 4)}`;
    }
    if (mo === "year" && m) return m;
    return "—";
  }

  function shift(delta: number) {
    if (mode === "all") {
      const y = currentYear();
      const mm = String(currentMonth()).padStart(2, "0");
      onChange(`${y}-${mm}`);
      return;
    }
    if (mode === "month" && month) {
      const y = Number(month.slice(0, 4));
      const m = Number(month.slice(5, 7));
      const d = new Date(y, m - 1 + delta, 1);
      onChange(`${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}`);
    } else if (mode === "year" && month) {
      onChange(String(Number(month) + delta));
    }
  }

  function toggleMode() {
    if (mode === "month" && month) {
      onChange(month.slice(0, 4));
    } else if (mode === "year" && month) {
      const mm = String(currentMonth()).padStart(2, "0");
      onChange(`${month}-${mm}`);
    } else {
      const y = currentYear();
      const mm = String(currentMonth()).padStart(2, "0");
      onChange(`${y}-${mm}`);
    }
  }

  let toggleLabel = $derived(
    mode === "month" ? "Todos os meses" : mode === "year" ? "Mês específico" : "Filtrar período",
  );

  // ─── Calendar popover ────────────────────────────────────────────
  let pickerOpen = $state(false);
  let triggerEl: HTMLButtonElement | undefined = $state();
  let popoverEl: HTMLDivElement | undefined = $state();
  let pickerYear = $state(currentYear());

  $effect(() => {
    if (pickerOpen) {
      pickerYear = parseYear(month) ?? currentYear();
    }
  });

  function pickMonth(y: number, m: number) {
    onChange(`${y}-${String(m).padStart(2, "0")}`);
    pickerOpen = false;
  }
  function pickYearOnly(y: number) {
    onChange(String(y));
    pickerOpen = false;
  }
  function pickCurrentMonth() {
    const y = currentYear();
    onChange(`${y}-${String(currentMonth()).padStart(2, "0")}`);
    pickerOpen = false;
  }
  function pickAll() {
    onChange(null);
    pickerOpen = false;
  }

  onMount(() => {
    function handleClick(e: MouseEvent) {
      if (!pickerOpen) return;
      const t = e.target as Node | null;
      if (t && !triggerEl?.contains(t) && !popoverEl?.contains(t)) {
        pickerOpen = false;
      }
    }
    document.addEventListener("mousedown", handleClick);
    return () => document.removeEventListener("mousedown", handleClick);
  });

  let selMonth = $derived(parseMonth(month));
  let selYear = $derived(parseYear(month));
</script>

<div class="inline-flex items-center gap-2 relative">
  <div class="inline-flex items-center gap-px rounded-md border border-border bg-surface-2">
    <button
      type="button"
      class="px-2 py-1 text-fg-muted hover:bg-hover rounded-l-md"
      onclick={() => shift(-1)}
      aria-label={mode === "year" ? "Ano anterior" : "Mês anterior"}
    >
      ‹
    </button>
    <span class="px-2.5 text-[12px] font-medium tabular min-w-[88px] text-center">
      {label}
    </span>
    <button
      type="button"
      class="px-2 py-1 text-fg-muted hover:bg-hover rounded-r-md"
      onclick={() => shift(1)}
      aria-label={mode === "year" ? "Próximo ano" : "Próximo mês"}
    >
      ›
    </button>
  </div>

  <button
    type="button"
    onclick={toggleMode}
    class="text-[11px] text-fg-faint hover:text-fg-muted underline-offset-2 hover:underline"
  >
    {toggleLabel}
  </button>

  <button
    bind:this={triggerEl}
    type="button"
    onclick={() => (pickerOpen = !pickerOpen)}
    class="w-7 h-7 grid place-items-center rounded-md border border-border bg-surface-2 text-fg-muted hover:bg-hover hover:text-fg transition-colors"
    aria-label="Calendário"
  >
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.7"
      stroke-linecap="round"
      stroke-linejoin="round"
      class="w-3.5 h-3.5"
    >
      <rect x="3" y="4" width="18" height="18" rx="2" />
      <path d="M16 2v4M8 2v4M3 10h18" />
    </svg>
  </button>

  {#if pickerOpen}
    <div
      bind:this={popoverEl}
      class="absolute z-30 top-full mt-1 left-0 w-[260px] rounded-lg border border-border bg-surface p-3 flex flex-col gap-2.5"
      style="box-shadow: 0 12px 32px -8px rgba(0,0,0,.55), 0 0 0 1px var(--color-border)"
    >
      <div class="flex items-center justify-between">
        <button
          type="button"
          onclick={() => (pickerYear -= 1)}
          class="w-7 h-7 grid place-items-center text-fg-muted hover:bg-hover rounded-md"
          aria-label="Ano anterior"
        >
          ‹
        </button>
        <button
          type="button"
          onclick={() => pickYearOnly(pickerYear)}
          class="text-[13.5px] font-semibold tabular tracking-tight hover:bg-hover rounded-md px-3 py-0.5
                 {selYear === pickerYear && selMonth === null ? 'bg-accent-soft text-fg' : 'text-fg'}"
          title="Filtrar pelo ano inteiro"
          style="font-family: var(--font-display)"
        >
          {pickerYear}
        </button>
        <button
          type="button"
          onclick={() => (pickerYear += 1)}
          class="w-7 h-7 grid place-items-center text-fg-muted hover:bg-hover rounded-md"
          aria-label="Próximo ano"
        >
          ›
        </button>
      </div>

      <div class="grid grid-cols-3 gap-1">
        {#each MONTH_NAMES_FULL as name, i}
          {@const monthNum = i + 1}
          {@const isSelected = selYear === pickerYear && selMonth === monthNum}
          <button
            type="button"
            onclick={() => pickMonth(pickerYear, monthNum)}
            class="px-2 py-1.5 text-[11.5px] rounded-md transition-colors
                   {isSelected ? 'bg-accent text-accent-on font-medium' : 'text-fg-muted hover:bg-hover hover:text-fg'}"
            title={name}
          >
            {name.slice(0, 3)}
          </button>
        {/each}
      </div>

      <div class="flex items-center justify-between border-t border-border-subtle pt-2.5 text-[11px]">
        <button
          type="button"
          onclick={pickAll}
          class="text-fg-faint hover:text-fg-muted underline-offset-2 hover:underline"
        >
          Todos os períodos
        </button>
        <button
          type="button"
          onclick={pickCurrentMonth}
          class="text-accent hover:underline underline-offset-2"
        >
          Mês atual
        </button>
      </div>
    </div>
  {/if}
</div>

