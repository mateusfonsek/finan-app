<script lang="ts">
  import { onMount } from "svelte";
  import Icon from "$lib/components/ui/Icon.svelte";
  import { locale } from "$lib/i18n/locale.svelte";
  import { popover } from "$lib/motion";
  import { portal } from "$lib/actions/portal";

  const t = locale.t;

  type Props = {
    /** "YYYY-MM" = one month, "YYYY" = a whole year, null = all periods */
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

  // A null from the store (legacy) normalizes to the current month without
  // exposing "all" in the UI.
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
      return `${monthShort(Number(m.slice(5, 7)) - 1)} ${m.slice(0, 4)}`;
    }
    // Fallback: current month, while the $effect has not normalized yet.
    return `${monthShort(currentMonth() - 1)} ${currentYear()}`;
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

  const POPOVER_W = 268;
  const POPOVER_MARGIN = 8;
  let popoverStyle = $state("");

  /** Popover position relative to the trigger button. Synchronous. */
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
    const top = r.bottom + 6;
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
    function handleKey(e: KeyboardEvent) {
      if (e.key === "Escape" && pickerOpen) {
        pickerOpen = false;
        triggerEl?.focus();
      }
    }
    function handleResize() {
      if (pickerOpen) popoverStyle = computePopoverStyle();
    }
    document.addEventListener("mousedown", handleClick);
    document.addEventListener("keydown", handleKey);
    window.addEventListener("resize", handleResize);
    window.addEventListener("scroll", handleResize, true);
    return () => {
      document.removeEventListener("mousedown", handleClick);
      document.removeEventListener("keydown", handleKey);
      window.removeEventListener("resize", handleResize);
      window.removeEventListener("scroll", handleResize, true);
    };
  });

  let selMonth = $derived(parseMonth(month));
  let selYear = $derived(parseYear(month));

  const STEP =
    "w-6 h-6 grid place-items-center rounded-[5px] text-fg-subtle " +
    "hover:bg-hover hover:text-fg active:scale-[0.94] " +
    "transition-[background-color,color,transform] duration-[var(--dur-instant)] ease-[var(--ease-snap)]";
</script>

<div class="inline-flex items-center gap-1.5 relative">
  <!-- macOS segmented control: ‹ label › in a single frame. -->
  <div
    class="inline-flex items-center gap-0.5 h-7 px-0.5 rounded-[var(--radius-md)] border border-border
           bg-surface-2 shadow-[var(--shadow-raised)]"
  >
    <button
      type="button"
      class={STEP}
      onclick={() => shift(-1)}
      aria-label={mode === "year" ? t("month_stepper.prev_year") : t("month_stepper.prev_month")}
    >
      <Icon name="chevronLeft" size={13} stroke={2} />
    </button>
    <span class="px-2 text-callout font-medium tabular min-w-[80px] text-center text-fg">
      {label}
    </span>
    <button
      type="button"
      class={STEP}
      onclick={() => shift(1)}
      aria-label={mode === "year" ? t("month_stepper.next_year") : t("month_stepper.next_month")}
    >
      <Icon name="chevronRight" size={13} stroke={2} />
    </button>
  </div>

  <button
    bind:this={triggerEl}
    type="button"
    onclick={togglePicker}
    aria-expanded={pickerOpen}
    aria-haspopup="dialog"
    class="press w-7 h-7 grid place-items-center rounded-[var(--radius-md)] border border-border
           bg-surface-2 shadow-[var(--shadow-raised)] transition-colors duration-[var(--dur-fast)]
           {pickerOpen ? 'bg-hover text-fg border-accent' : 'text-fg-subtle hover:bg-hover hover:text-fg'}"
    aria-label={t("month_stepper.calendar")}
  >
    <Icon name="calendar" size={14} />
  </button>

  {#if pickerOpen}
    <!-- Grows from the button that opened it, not from its own centre. -->
    <div
      bind:this={popoverEl}
      use:portal
      transition:popover={{ origin: "top right" }}
      role="dialog"
      aria-label={t("month_stepper.calendar")}
      class="material-pop z-30 w-[268px] p-2.5 flex flex-col gap-2"
      style={popoverStyle}
    >
      <div class="flex items-center justify-between">
        <button
          type="button"
          onclick={() => (pickerYear -= 1)}
          class={STEP}
          aria-label={t("month_stepper.prev_year")}
        >
          <Icon name="chevronLeft" size={13} stroke={2} />
        </button>
        <button
          type="button"
          onclick={() => pickYearOnly(pickerYear)}
          class="text-title3 font-semibold tabular rounded-[var(--radius-sm)] px-3 py-0.5
                 transition-colors duration-[var(--dur-fast)]
                 {selYear === pickerYear && selMonth === null
            ? 'bg-accent text-accent-on'
            : 'text-fg hover:bg-hover'}"
          title={t("month_stepper.filter_year")}
        >
          {pickerYear}
        </button>
        <button
          type="button"
          onclick={() => (pickerYear += 1)}
          class={STEP}
          aria-label={t("month_stepper.next_year")}
        >
          <Icon name="chevronRight" size={13} stroke={2} />
        </button>
      </div>

      <div class="grid grid-cols-3 gap-1">
        {#each locale.months as name, i}
          {@const monthNum = i + 1}
          {@const isSelected = selYear === pickerYear && selMonth === monthNum}
          {@const isNow = pickerYear === currentYear() && monthNum === currentMonth()}
          <button
            type="button"
            onclick={() => pickMonth(pickerYear, monthNum)}
            class="press h-7 rounded-[var(--radius-sm)] text-sub font-medium
                   transition-colors duration-[var(--dur-fast)]
                   {isSelected
              ? 'bg-accent text-accent-on'
              : isNow
                ? 'text-accent hover:bg-hover'
                : 'text-fg-muted hover:bg-hover hover:text-fg'}"
            title={name}
          >
            {monthShort(i)}
          </button>
        {/each}
      </div>

      <div class="hairline mt-0.5"></div>

      <div class="flex items-center justify-between text-foot">
        {#if mode === "month"}
          <button
            type="button"
            onclick={() => {
              toggleToYear();
              closePicker();
            }}
            class="text-fg-subtle hover:text-fg transition-colors duration-[var(--dur-fast)]
                   underline-offset-2 hover:underline px-1 py-0.5"
          >
            {t("month_stepper.all_months")}
          </button>
        {:else}
          <span></span>
        {/if}
        <button
          type="button"
          onclick={pickCurrentMonth}
          class="text-accent hover:underline underline-offset-2 px-1 py-0.5 font-medium
                 transition-colors duration-[var(--dur-fast)]"
        >
          {t("month_stepper.current_month")}
        </button>
      </div>
    </div>
  {/if}
</div>
