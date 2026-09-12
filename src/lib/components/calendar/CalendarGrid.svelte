<script lang="ts">
  import { formatMoney } from "$lib/format/money";
  import { locale } from "$lib/i18n/locale.svelte";
  import { leadingBlanks, weekdayOrder } from "./week";
  import Icon from "$lib/components/ui/Icon.svelte";
  import { billState, daysOverdue, dueDateOf, type BillState } from "./bill";
  import type { IconName } from "$lib/components/ui/icons";
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
    /** Rule events with a due_day or a payment, used to render bills. */
    events?: CalendarEvent[];
    selectedDay?: number | null;
    onSelectDay?: (day: number | null) => void;
    /** Opening a bill's popover. The anchor is the chip, so the popover can
     *  grow from where it was clicked. */
    onSelectBill?: (event: CalendarEvent, anchor: HTMLElement) => void;
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
    onSelectBill,
  }: Props = $props();

  type DayCell = {
    day: number | null;
    isToday: boolean;
    /** Rules falling due on this day (due_day == d). */
    due: CalendarEvent[];
  };

  let cells = $derived(buildGrid(month, today, events, locale.firstDayOfWeek));

  function buildGrid(
    monthStr: string,
    todayStr: string,
    evs: CalendarEvent[],
    firstDayOfWeek: number,
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
    for (let i = 0; i < leadingBlanks(startWeekday, firstDayOfWeek); i++) {
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

  const BILL_ICON: Record<BillState, IconName> = {
    paid: "check",
    overdue: "circleAlert",
    pending: "clock",
  };

  function billStyle(state: BillState): string {
    const token =
      state === "paid" ? "--color-pos" : state === "overdue" ? "--color-neg" : "--color-cat-amarelo";
    return `background: color-mix(in oklch, var(${token}) 16%, transparent); color: var(${token});`;
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
    {#each weekdayOrder(locale.firstDayOfWeek) as wd}
      <div class="px-2 py-2 text-cap font-medium text-fg-subtle text-center">
        {locale.weekdaysShort[wd]}
      </div>
    {/each}
  </div>

  <div class="grid grid-cols-7">
    {#each cells as cell, i}
      {@const flow = cell.day !== null ? dayFlows.get(cell.day) : undefined}
      {@const outPct = flow ? intensityPct(flow.outflow, maxOut) : 0}
      {@const inPct = flow ? intensityPct(flow.inflow, maxIn) : 0}
      <div
        class="min-h-[86px] border-r border-b border-border-subtle relative
               {cell.day === null ? 'bg-surface-2/35' : ''}
               {i % 7 === 6 ? 'border-r-0' : ''}"
      >
        {#if cell.day !== null}
          <!-- Behind the content so the bill chips above it stay clickable: a
               <button> cannot contain another one, and the cell used to be the
               button. -->
          <button
            type="button"
            onclick={() => handleClick(cell.day!)}
            aria-pressed={cell.day === selectedDay}
            aria-label={flow
              ? t("calendar.day_aria_flow", {
                  day: cell.day,
                  inflow: formatMoney(String(flow.inflow)),
                  outflow: formatMoney(String(flow.outflow)),
                })
              : t("calendar.day_aria", { day: cell.day })}
            class="absolute inset-0 transition-colors duration-[var(--dur-fast)] ease-[var(--ease-snap)]
                   {cell.day === selectedDay
              ? 'bg-accent-soft ring-[1.5px] ring-accent ring-inset'
              : 'hover:bg-hover'}"
          ></button>

          <div class="relative z-10 p-1.5 flex flex-col gap-1 pointer-events-none">
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
                  <span class="w-2 h-2 rounded-full" style={dotStyle("var(--color-neg)", outPct)}></span>
                {/if}
                {#if inPct > 0}
                  <span class="w-2 h-2 rounded-full" style={dotStyle("var(--color-pos)", inPct)}></span>
                {/if}
              </div>
            </div>

            <!-- Bills due this day (up to 2 visible, "+N" beyond). -->
            {#each cell.due.slice(0, 2) as e (e.rule_id)}
              {@const state = billState(e, month, today)}
              {@const due = dueDateOf(e, month)}
              {@const overdueDays = due != null ? daysOverdue(due, today) : 0}
              <button
                type="button"
                onclick={(ev) => onSelectBill?.(e, ev.currentTarget)}
                aria-haspopup="dialog"
                class="press-sm pointer-events-auto text-cap rounded-[4px] px-1 min-h-[18px] truncate
                       flex items-center gap-1 font-medium w-full text-left"
                style={billStyle(state)}
              >
                <Icon name={BILL_ICON[state]} size={10} stroke={2.2} />
                <span class="truncate">{e.pattern}</span>
                <span class="sr-only">
                  {state === "paid"
                    ? t("calendar.bill_state_paid")
                    : state === "overdue"
                      ? overdueDays === 1
                        ? t("calendar.bill_state_overdue_one")
                        : t("calendar.bill_state_overdue_days", { n: overdueDays })
                      : t("calendar.bill_state_pending")}
                </span>
              </button>
            {/each}
            {#if cell.due.length > 2}
              <div class="text-cap2 text-fg-subtle px-1">
                {t("calendar.more", { n: cell.due.length - 2 })}
              </div>
            {/if}
          </div>
        {/if}
      </div>
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
            <span class="flex items-center gap-1.5">
              <span
                class="w-[14px] h-[14px] rounded-[4px] grid place-items-center"
                style="color: var({token}); background: color-mix(in oklch, var({token}) 16%, transparent);"
              >
                <Icon name={BILL_ICON[key as BillState]} size={9} stroke={2.4} />
              </span>
              {t(label)}
            </span>
          {/each}
        </div>
      {/if}
    </div>
  {/if}
</div>
