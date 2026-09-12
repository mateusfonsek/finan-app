<script lang="ts">
  import { locale } from "$lib/i18n/locale.svelte";

  const t = locale.t;
  import { onMount } from "svelte";
  import Page from "$lib/components/ui/Page.svelte";
  import ErrorNote from "$lib/components/ui/ErrorNote.svelte";
  import Spinner from "$lib/components/ui/Spinner.svelte";
  import MonthStepper from "$lib/components/shell/MonthStepper.svelte";
  import CalendarGrid, { type DayFlow } from "$lib/components/calendar/CalendarGrid.svelte";
  import BillPopover from "$lib/components/calendar/BillPopover.svelte";
  import DayDetails from "$lib/components/calendar/DayDetails.svelte";
  import RuleDuePicker from "$lib/components/calendar/RuleDuePicker.svelte";
  import { formatMoney } from "$lib/format/money";
  import { filters } from "$lib/stores/filters.svelte";
  import { calendarEvents, listRules } from "$lib/api/rules";
  import { listTransactions } from "$lib/api/transactions";
  import { listCategories } from "$lib/api/categories";
  import { monthBillTotals } from "$lib/components/calendar/bill";
  import type { CalendarEvent, Category, Rule, Transaction } from "$lib/bindings";

  let events = $state<CalendarEvent[]>([]);
  let transactions = $state<Transaction[]>([]);
  let categories = $state<Category[]>([]);
  let rules = $state<Rule[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);

  /** Dia selecionado (1..31). null = nenhum. */
  let selectedDay = $state<number | null>(null);

  let openBill = $state<{ event: CalendarEvent; anchor: HTMLElement } | null>(null);

  // Two entry points, one dialog: a day clicked in the grid fixes the day, the
  // month strip leaves it for the dialog to ask.
  let pickerOpen = $state(false);
  let markDueDay = $state<number | null>(null);

  /** Today in the reader's timezone. `toISOString()` returns UTC: at night,
   *  west of Greenwich, it has already rolled over — and the calendar was
   *  highlighting tomorrow. */
  const today = (() => {
    const d = new Date();
    return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
  })();

  function monthForCalendar(m: string | null): string {
    if (!m) {
      const d = new Date();
      return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}`;
    }
    if (m.length === 7) return m;
    if (m.length === 4) {
      const mm = String(new Date().getMonth() + 1).padStart(2, "0");
      return `${m}-${mm}`;
    }
    return m;
  }

  let viewMonth = $derived(monthForCalendar(filters.month));

  let selectedDate = $derived(
    selectedDay == null ? null : `${viewMonth}-${String(selectedDay).padStart(2, "0")}`,
  );

  /** category_ids with kind='transfer' (including investments). The heatmap
   *  excludes them. */
  let transferCatIds = $derived.by(() => {
    const set = new Set<number>();
    for (const c of categories) if (c.kind === "transfer") set.add(c.id);
    return set;
  });

  let dayFlows = $derived.by(() => {
    const map = new Map<number, DayFlow>();
    for (const t of transactions) {
      if (!t.date.startsWith(viewMonth)) continue;
      // Excludes transfers and investments — the heatmap shows real movement.
      if (t.category_id != null && transferCatIds.has(t.category_id)) continue;
      const d = Number(t.date.slice(8, 10));
      const n = Number(t.amount);
      if (!Number.isFinite(n) || n === 0) continue;
      const bucket = map.get(d) ?? { inflow: 0, outflow: 0 };
      if (n > 0) bucket.inflow += n;
      else bucket.outflow += -n;
      map.set(d, bucket);
    }
    return map;
  });

  let maxOut = $derived.by(() => {
    let m = 0;
    for (const { outflow } of dayFlows.values()) if (outflow > m) m = outflow;
    return m;
  });

  let maxIn = $derived.by(() => {
    let m = 0;
    for (const { inflow } of dayFlows.values()) if (inflow > m) m = inflow;
    return m;
  });

  let monthTotals = $derived.by(() => {
    let inflow = 0;
    let outflow = 0;
    for (const f of dayFlows.values()) {
      inflow += f.inflow;
      outflow += f.outflow;
    }
    return { inflow, outflow, net: inflow - outflow };
  });

  let billTotals = $derived(monthBillTotals(events, viewMonth, today));

  /** Rules with no due day never reach `events` — `calendar_events` only
   *  returns a rule with a due day or a payment — so the month strip needs its
   *  own load to count them. */
  let withoutDueDay = $derived(rules.filter((r) => r.due_day === null).length);

  async function loadRulesOnce(): Promise<void> {
    try {
      rules = await listRules();
    } catch (e) {
      console.error("[calendar] failed to load rules", e);
    }
  }

  /**
   * Loads categories ONCE. Separate from the monthly flow so the $effect that
   * loads month data does not track `categories.length` and fire again after
   * the assignment — which was what made the side panel flicker on day click.
   */
  async function loadCategoriesOnce(): Promise<void> {
    if (categories.length > 0) return;
    try {
      categories = await listCategories();
    } catch (e) {
      console.error("[calendar] failed to load categories", e);
    }
  }

  /** Pure: takes the month as a parameter, reads no reactive state. */
  async function loadMonthData(month: string): Promise<void> {
    loading = true;
    error = null;
    try {
      const [evs, txs] = await Promise.all([
        calendarEvents(month),
        listTransactions({
          account_id: null,
          month,
          category_id: null,
          q: null,
          limit: null,
        }),
      ]);
      events = evs;
      transactions = txs;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    void loadCategoriesOnce();
    void loadRulesOnce();
  });

  // The only reactive effect: refetch and reset the selection when the month
  // changes.
  $effect(() => {
    const m = viewMonth;
    const todayMonth = today.slice(0, 7);
    selectedDay = todayMonth === m ? Number(today.slice(8, 10)) : null;
    void loadMonthData(m);
  });

  function onMonthChange(m: string | null) {
    filters.month = m;
  }

</script>

<Page
  title={t("nav.calendar")}
  subtitle={t("calendar_page.subtitle")}
  width="wide"
>
  {#snippet toolbar()}
    <MonthStepper month={viewMonth} onChange={onMonthChange} />
  {/snippet}

  {#if error}
    <ErrorNote message={error} />
  {/if}

  <!-- Compact month summary, always visible even in an empty month -->
  <div class="card px-4 py-2.5 flex items-center gap-7 text-callout">
    <div class="flex items-center gap-2">
      <span class="text-foot text-fg-subtle">{t("calendar_page.inflows")}</span>
      <span class="tabular text-pos font-medium">
        {monthTotals.inflow > 0 ? formatMoney(String(monthTotals.inflow)) : "—"}
      </span>
    </div>
    <div class="flex items-center gap-2">
      <span class="text-foot text-fg-subtle">{t("calendar_page.outflows")}</span>
      <span class="tabular text-neg font-medium">
        {monthTotals.outflow > 0 ? formatMoney(String(monthTotals.outflow)) : "—"}
      </span>
    </div>
    {#if billTotals.upcoming > 0 || billTotals.overdue > 0}
      <div class="flex items-center gap-2">
        <span class="text-foot text-fg-subtle">{t("calendar_page.bills")}</span>
        <span class="text-callout text-fg">
          {billTotals.upcoming === 1
            ? t("calendar_page.upcoming_one")
            : t("calendar_page.upcoming_many", { n: billTotals.upcoming })}
        </span>
        {#if billTotals.overdue > 0}
          <span class="text-callout text-neg font-medium">
            {billTotals.overdue === 1
              ? t("calendar_page.overdue_one")
              : t("calendar_page.overdue_many", { n: billTotals.overdue })}
          </span>
        {/if}
      </div>
    {/if}
    {#if withoutDueDay > 0}
      <button
        type="button"
        onclick={() => { markDueDay = null; pickerOpen = true; }}
        class="text-foot text-fg-subtle hover:text-fg transition-colors duration-[var(--dur-fast)]"
      >
        {withoutDueDay === 1
          ? t("calendar_page.no_due_one")
          : t("calendar_page.no_due_many", { n: withoutDueDay })}
      </button>
    {/if}
    <div class="flex items-center gap-2 ml-auto">
      {#if loading}
        <Spinner size={12} class="text-fg-faint" />
      {/if}
      <span class="text-foot text-fg-subtle">{t("calendar_page.net")}</span>
      <span class="tabular font-semibold {monthTotals.net >= 0 ? 'text-pos' : 'text-neg'}">
        {formatMoney(String(monthTotals.net))}
      </span>
    </div>
  </div>

  <div class="grid grid-cols-[1fr_340px] gap-4 items-start">
    <CalendarGrid
      month={viewMonth}
      {today}
      {dayFlows}
      {maxOut}
      {maxIn}
      {events}
      {selectedDay}
      onSelectDay={(d) => (selectedDay = d)}
      onSelectBill={(event, anchor) => (openBill = { event, anchor })}
    />

    <aside class="flex flex-col gap-4">
      <DayDetails
        {selectedDate}
        {transactions}
        {categories}
        {events}
        {today}
        onMarkDue={(d) => {
          markDueDay = d;
          pickerOpen = true;
        }}
      />
    </aside>
  </div>

  {#if openBill}
    <BillPopover
      event={openBill.event}
      dueMonth={viewMonth}
      anchor={openBill.anchor}
      {today}
      onClose={() => (openBill = null)}
      onChanged={() => void loadMonthData(viewMonth)}
    />
  {/if}
</Page>

{#if pickerOpen}
  <RuleDuePicker
    day={markDueDay}
    onClose={() => (pickerOpen = false)}
    onChanged={() => {
      void loadMonthData(viewMonth);
      void loadRulesOnce();
    }}
  />
{/if}
