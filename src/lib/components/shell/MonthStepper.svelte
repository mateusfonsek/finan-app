<script lang="ts">
  import { onMount } from "svelte";
  import { locale } from "$lib/i18n/locale.svelte";

  const t = locale.t;

  type Props = {
    /** "YYYY-MM" = mês específico, "YYYY" = ano inteiro, null = todos os períodos */
    month: string | null;
    onChange: (m: string | null) => void;
  };

  let { month, onChange }: Props = $props();

  function monthShort(i: number): string {
    return locale.monthsShort[i] ?? String(i + 1);
  }

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

  let mode = $derived<"month" | "year">(
    !month || month.length === 7 ? "month" : "year",
  );

  // Se a store vier null (legado), normaliza pro mês atual sem expor o "all" no UI.
  $effect(() => {
    if (month === null) {
      const y = currentYear();
      const mm = String(currentMonth()).padStart(2, "0");
      onChange(`${y}-${mm}`);
    }
  });

  let label = $derived(computeLabel(month, mode));

  function computeLabel(m: string | null, mo: "month" | "year"): string {
    if (mo === "year" && m) return m;
    if (mo === "month" && m && m.length === 7) {
      return `${monthShort(Number(m.slice(5, 7)) - 1)}/${m.slice(0, 4)}`;
    }
    // fallback: mês atual (enquanto $effect ainda não normalizou)
    return `${monthShort(currentMonth() - 1)}/${currentYear()}`;
  }

  function shift(delta: number) {
    if (mode === "month") {
      const base = month && month.length === 7
        ? month
        : `${currentYear()}-${String(currentMonth()).padStart(2, "0")}`;
      const y = Number(base.slice(0, 4));
      const m = Number(base.slice(5, 7));
      const d = new Date(y, m - 1 + delta, 1);
      onChange(`${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}`);
    } else if (month) {
      onChange(String(Number(month) + delta));
    }
  }

  function toggleToYear() {
    const base = month && month.length === 7 ? month : `${currentYear()}-01`;
    onChange(base.slice(0, 4));
  }

  // ─── Calendar popover ────────────────────────────────────────────
  let pickerOpen = $state(false);
  let triggerEl: HTMLButtonElement | undefined = $state();
  let popoverEl: HTMLDivElement | undefined = $state();
  let pickerYear = $state(currentYear());

  const POPOVER_W = 260;
  const POPOVER_MARGIN = 8;
  let popoverStyle = $state("");

  /** Calcula a posição do popover relativa ao trigger button. Síncrono. */
  function computePopoverStyle(): string {
    if (!triggerEl) return "";
    const r = triggerEl.getBoundingClientRect();
    const vw = window.innerWidth;
    // Prefere alinhar à direita do trigger (popover abre pra esquerda).
    let left = r.right - POPOVER_W;
    // Se cortar no edge esquerdo, flipa pra abrir pra direita.
    if (left < POPOVER_MARGIN) {
      left = r.left;
      if (left + POPOVER_W > vw - POPOVER_MARGIN) {
        left = vw - POPOVER_W - POPOVER_MARGIN;
      }
    }
    const top = r.bottom + 4;
    return `position: fixed; top: ${top}px; left: ${left}px;`;
  }

  function openPicker() {
    popoverStyle = computePopoverStyle();
    pickerYear = parseYear(month) ?? currentYear();
    pickerOpen = true;
  }

  function closePicker() {
    pickerOpen = false;
  }

  function togglePicker() {
    if (pickerOpen) closePicker();
    else openPicker();
  }

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

  onMount(() => {
    function handleClick(e: MouseEvent) {
      if (!pickerOpen) return;
      const t = e.target as Node | null;
      if (t && !triggerEl?.contains(t) && !popoverEl?.contains(t)) {
        pickerOpen = false;
      }
    }
    function handleResize() {
      if (pickerOpen) popoverStyle = computePopoverStyle();
    }
    document.addEventListener("mousedown", handleClick);
    window.addEventListener("resize", handleResize);
    window.addEventListener("scroll", handleResize, true);
    return () => {
      document.removeEventListener("mousedown", handleClick);
      window.removeEventListener("resize", handleResize);
      window.removeEventListener("scroll", handleResize, true);
    };
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
      aria-label={mode === "year" ? t("month_stepper.prev_year") : t("month_stepper.prev_month")}
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
      aria-label={mode === "year" ? t("month_stepper.next_year") : t("month_stepper.next_month")}
    >
      ›
    </button>
  </div>

  <button
    bind:this={triggerEl}
    type="button"
    onclick={togglePicker}
    class="w-7 h-7 grid place-items-center rounded-md border border-border bg-surface-2 text-fg-muted hover:bg-hover hover:text-fg transition-colors"
    aria-label={t("month_stepper.calendar")}
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
      class="z-30 w-[260px] rounded-lg border border-border bg-surface p-3 flex flex-col gap-2.5"
      style="{popoverStyle}; box-shadow: 0 12px 32px -8px rgba(0,0,0,.55), 0 0 0 1px var(--color-border);"
    >
      <div class="flex items-center justify-between">
        <button
          type="button"
          onclick={() => (pickerYear -= 1)}
          class="w-7 h-7 grid place-items-center text-fg-muted hover:bg-hover rounded-md"
          aria-label={t("month_stepper.prev_year")}
        >
          ‹
        </button>
        <button
          type="button"
          onclick={() => pickYearOnly(pickerYear)}
          class="text-[13.5px] font-semibold tabular tracking-tight hover:bg-hover rounded-md px-3 py-0.5
                 {selYear === pickerYear && selMonth === null ? 'bg-accent-soft text-fg' : 'text-fg'}"
          title={t("month_stepper.filter_year")}
          style="font-family: var(--font-display)"
        >
          {pickerYear}
        </button>
        <button
          type="button"
          onclick={() => (pickerYear += 1)}
          class="w-7 h-7 grid place-items-center text-fg-muted hover:bg-hover rounded-md"
          aria-label={t("month_stepper.next_year")}
        >
          ›
        </button>
      </div>

      <div class="grid grid-cols-3 gap-1">
        {#each locale.months as name, i}
          {@const monthNum = i + 1}
          {@const isSelected = selYear === pickerYear && selMonth === monthNum}
          <button
            type="button"
            onclick={() => pickMonth(pickerYear, monthNum)}
            class="px-2 py-1.5 text-[11.5px] rounded-md transition-colors
                   {isSelected ? 'bg-accent text-accent-on font-medium' : 'text-fg-muted hover:bg-hover hover:text-fg'}"
            title={name}
          >
            {monthShort(i)}
          </button>
        {/each}
      </div>

      <div class="flex items-center justify-between border-t border-border-subtle pt-2.5 text-[11px]">
        {#if mode === "month"}
          <button
            type="button"
            onclick={() => {
              toggleToYear();
              closePicker();
            }}
            class="text-fg-faint hover:text-fg-muted underline-offset-2 hover:underline"
          >
            {t("month_stepper.all_months")}
          </button>
        {:else}
          <span></span>
        {/if}
        <button
          type="button"
          onclick={pickCurrentMonth}
          class="text-accent hover:underline underline-offset-2"
        >
          {t("month_stepper.current_month")}
        </button>
      </div>
    </div>
  {/if}
</div>

