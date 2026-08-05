<script lang="ts">
  import { formatMoney } from "$lib/format/money";
  import { locale } from "$lib/i18n/locale.svelte";
  import type { CalendarEvent } from "$lib/bindings";

  const t = locale.t;

  /** Per-day totals, computed in Calendar.svelte from the month's transactions. */
  export type DayFlow = { inflow: number; outflow: number };

  type Props = {
    /** "YYYY-MM" of the displayed month */
    month: string;
    /** Today, as "YYYY-MM-DD" */
    today: string;
    /** Per-day totals (key = day of month, 1..31). */
    dayFlows?: Map<number, DayFlow>;
    maxOut?: number;
    maxIn?: number;
    /** Rule events with a due_day or paid_day, used to render bills. */
    events?: CalendarEvent[];
    selectedDay?: number | null;
    onSelectDay?: (day: number | null) => void;
  };

  let {
    month,
    today,
    dayFlows = new Map(),
    maxOut = 0,
    maxIn = 0,
    events = [],
    selectedDay = null,
    onSelectDay,
  }: Props = $props();

  type DayCell = {
    day: number | null;
    isToday: boolean;
    /** Regras vencendo neste dia (com due_day == d) */
    due: CalendarEvent[];
  };

  let todayDayInMonth = $derived(
    today.slice(0, 7) === month ? Number(today.slice(8, 10)) : -1,
  );

  let cells = $derived(buildGrid(month, today, events));

  function buildGrid(
    monthStr: string,
    todayStr: string,
    evs: CalendarEvent[],
  ): DayCell[] {
    const [yStr, mStr] = monthStr.split("-");
    const year = Number(yStr);
    const monthIdx = Number(mStr) - 1;
    const first = new Date(year, monthIdx, 1);
    const daysInMonth = new Date(year, monthIdx + 1, 0).getDate();
    const startWeekday = first.getDay();

    const todayPrefix = todayStr.slice(0, 7);
    const todayDay = todayPrefix === monthStr ? Number(todayStr.slice(8, 10)) : -1;

    // Buckets events by due_day (anchors day 31 to the month's last day).
    const dueByDay = new Map<number, CalendarEvent[]>();
    for (const e of evs) {
      if (e.due_day == null) continue;
      const d = Math.min(e.due_day, daysInMonth);
      const list = dueByDay.get(d) ?? [];
      list.push(e);
      dueByDay.set(d, list);
    }

    const out: DayCell[] = [];
    for (let i = 0; i < startWeekday; i++) {
      out.push({ day: null, isToday: false, due: [] });
    }
    for (let d = 1; d <= daysInMonth; d++) {
      out.push({
        day: d,
        isToday: d === todayDay,
        due: dueByDay.get(d) ?? [],
      });
    }
    // Fills the last week: a grid ending mid-row leaves an unbordered step in
    // the card's corner.
    while (out.length % 7 !== 0) {
      out.push({ day: null, isToday: false, due: [] });
    }
    return out;
  }

  /** Estado visual de uma conta: pago / vencido / pendente (futuro). */
  type BillState = "paid" | "overdue" | "pending";

  function billState(e: CalendarEvent, todayDay: number): BillState {
    if (e.paid_day != null) return "paid";
    if (e.due_day != null && todayDay > 0 && e.due_day < todayDay) return "overdue";
    return "pending";
  }

  function billStyle(state: BillState): string {
    switch (state) {
      case "paid":
        return "background: color-mix(in oklch, var(--color-pos) 16%, transparent); color: var(--color-pos);";
      case "overdue":
        return "background: color-mix(in oklch, var(--color-neg) 16%, transparent); color: var(--color-neg);";
      case "pending":
        return "background: color-mix(in oklch, var(--color-cat-amarelo) 15%, transparent); color: var(--color-cat-amarelo);";
    }
  }

  function billIcon(state: BillState): string {
    return state === "paid" ? "✓" : state === "overdue" ? "!" : "•";
  }

  /** Maps intensity [0,1] to visible opacity [20%, 100%]. */
  function intensityPct(value: number, max: number): number {
    if (max <= 0 || value <= 0) return 0;
    const ratio = Math.min(1, value / max);
    return Math.round(20 + 80 * ratio);
  }

  function dotStyle(color: string, pct: number): string {
    return `background: color-mix(in oklch, ${color} ${pct}%, transparent);`;
  }

  function handleClick(day: number) {
    if (!onSelectDay) return;
    onSelectDay(selectedDay === day ? null : day);
  }
</script>

<div class="card overflow-hidden">
  <div class="grid grid-cols-7 border-b border-border-subtle">
    {#each locale.weekdaysShort as wd}
      <div class="px-2 py-2 text-cap font-medium text-fg-subtle text-center">
        {wd}
      </div>
    {/each}
  </div>

  <div class="grid grid-cols-7">
    {#each cells as cell, i}
      {@const flow = cell.day !== null ? dayFlows.get(cell.day) : undefined}
      {@const outPct = flow ? intensityPct(flow.outflow, maxOut) : 0}
      {@const inPct = flow ? intensityPct(flow.inflow, maxIn) : 0}
      {@const isSelected = cell.day !== null && cell.day === selectedDay}
      <button
        type="button"
        disabled={cell.day === null}
        onclick={() => cell.day !== null && handleClick(cell.day)}
        class="text-left min-h-[86px] border-r border-b border-border-subtle p-1.5 flex flex-col gap-1 relative
               transition-colors duration-[var(--dur-fast)] ease-[var(--ease-snap)]
               {cell.day === null ? 'bg-surface-2/35' : 'hover:bg-hover'}
               {isSelected ? 'bg-accent-soft hover:bg-accent-soft ring-[1.5px] ring-accent ring-inset z-10' : ''}
               {i % 7 === 6 ? 'border-r-0' : ''}"
        aria-pressed={isSelected}
        aria-label={cell.day === null
          ? undefined
          : flow
            ? t("calendar.day_aria_flow", { day: cell.day, inflow: flow.inflow, outflow: flow.outflow })
            : t("calendar.day_aria", { day: cell.day })}
      >
        {#if cell.day !== null}
          <div class="flex items-center justify-between gap-1 w-full">
            <!-- Today gets the accent disc, as in the macOS Calendar. -->
            <span
              class="grid place-items-center min-w-[19px] h-[19px] px-1 rounded-full text-foot tabular
                     {cell.isToday
                ? 'bg-accent text-accent-on font-semibold'
                : 'text-fg font-medium'}"
            >
              {cell.day}
            </span>
            <div class="flex items-center gap-1">
              {#if outPct > 0}
                <span
                  class="w-2 h-2 rounded-full"
                  style={dotStyle("var(--color-neg)", outPct)}
                  title={t("calendar.outflows_title", { value: formatMoney(String(flow?.outflow ?? 0)) })}
                ></span>
              {/if}
              {#if inPct > 0}
                <span
                  class="w-2 h-2 rounded-full"
                  style={dotStyle("var(--color-pos)", inPct)}
                  title={t("calendar.inflows_title", { value: formatMoney(String(flow?.inflow ?? 0)) })}
                ></span>
              {/if}
            </div>
          </div>

          <!-- Bills due this day (up to 2 visible, "+N" beyond). -->
          {#each cell.due.slice(0, 2) as e (e.rule_id)}
            {@const state = billState(e, todayDayInMonth)}
            <div
              class="text-cap2 rounded-[4px] px-1 py-0.5 truncate flex items-center gap-1 font-medium w-full"
              style={billStyle(state)}
              title={`${e.pattern}${
                state === "paid"
                  ? e.paid_amount
                    ? t("calendar.bill_paid_amount", { day: e.paid_day ?? "", amount: formatMoney(e.paid_amount) })
                    : t("calendar.bill_paid", { day: e.paid_day ?? "" })
                  : state === "overdue"
                    ? t("calendar.bill_overdue", { day: e.due_day ?? "" })
                    : t("calendar.bill_due", { day: e.due_day ?? "" })
              }`}
            >
              <span class="shrink-0 font-bold">{billIcon(state)}</span>
              <span class="truncate">{e.pattern}</span>
            </div>
          {/each}
          {#if cell.due.length > 2}
            <div class="text-cap2 text-fg-subtle px-1">
              {t("calendar.more", { n: cell.due.length - 2 })}
            </div>
          {/if}
        {/if}
      </button>
    {/each}
  </div>

  {#if maxOut > 0 || maxIn > 0 || events.some((e) => e.due_day != null)}
    <div
      class="border-t border-border-subtle px-3 py-2 flex items-center gap-4 text-cap text-fg-subtle flex-wrap"
    >
      {#if maxOut > 0}
        <div class="flex items-center gap-1.5">
          <span>{t("calendar.outflow")}</span>
          <div class="flex gap-0.5">
            {#each [20, 40, 60, 80, 100] as p}
              <span class="w-1.5 h-1.5 rounded-full" style={dotStyle("var(--color-neg)", p)}></span>
            {/each}
          </div>
        </div>
      {/if}
      {#if maxIn > 0}
        <div class="flex items-center gap-1.5">
          <span>{t("calendar.inflow")}</span>
          <div class="flex gap-0.5">
            {#each [20, 40, 60, 80, 100] as p}
              <span class="w-1.5 h-1.5 rounded-full" style={dotStyle("var(--color-pos)", p)}></span>
            {/each}
          </div>
        </div>
      {/if}
      {#if events.some((e) => e.due_day != null)}
        <div class="flex items-center gap-2 ml-auto">
          {#each [["pending", "--color-cat-amarelo", "calendar.legend_pending"], ["overdue", "--color-neg", "calendar.legend_overdue"], ["paid", "--color-pos", "calendar.legend_paid"]] as [key, token, label]}
            <span
              class="inline-flex items-center gap-1 rounded-full px-1.5 py-px font-medium"
              style="color: var({token}); background: color-mix(in oklch, var({token}) 14%, transparent);"
            >
              <span class="font-bold">
                {key === "paid" ? "✓" : key === "overdue" ? "!" : "•"}
              </span>
              {t(label)}
            </span>
          {/each}
        </div>
      {/if}
    </div>
  {/if}
</div>
