<script lang="ts">
  import { locale } from "$lib/i18n/locale.svelte";

  const t = locale.t;
  import { onMount } from "svelte";
  import MonthStepper from "$lib/components/shell/MonthStepper.svelte";
  import CalendarGrid, { type DayFlow } from "$lib/components/calendar/CalendarGrid.svelte";
  import DayDetails from "$lib/components/calendar/DayDetails.svelte";
  import { formatMoney } from "$lib/format/money";
  import { filters } from "$lib/stores/filters.svelte";
  import { calendarEvents } from "$lib/api/rules";
  import { listTransactions } from "$lib/api/transactions";
  import { listCategories } from "$lib/api/categories";
  import type { CalendarEvent, Category, Transaction } from "$lib/bindings";

  let events = $state<CalendarEvent[]>([]);
  let transactions = $state<Transaction[]>([]);
  let categories = $state<Category[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);

  /** Dia selecionado (1..31). null = nenhum. */
  let selectedDay = $state<number | null>(null);

  const today = new Date().toISOString().slice(0, 10);

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

  /** Set de category_ids que são kind='transfer' (inclui Investimentos). Heatmap exclui esses. */
  let transferCatIds = $derived.by(() => {
    const set = new Set<number>();
    for (const c of categories) if (c.kind === "transfer") set.add(c.id);
    return set;
  });

  let dayFlows = $derived.by(() => {
    const map = new Map<number, DayFlow>();
    for (const t of transactions) {
      if (!t.date.startsWith(viewMonth)) continue;
      // Exclui transferências/investimentos — heatmap mostra apenas movimento "real".
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

  /**
   * Carrega categorias UMA VEZ. Separado do fluxo mensal pra não fazer
   * o $effect que carrega dados do mês rastrear `categories.length` e
   * disparar de novo após a atribuição (era a causa do piscar do painel
   * lateral ao clicar num dia).
   */
  async function loadCategoriesOnce(): Promise<void> {
    if (categories.length > 0) return;
    try {
      categories = await listCategories();
    } catch (e) {
      console.error("[calendar] failed to load categories", e);
    }
  }

  /** Função pura: recebe o mês como parâmetro, não lê state reativo. */
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
  });

  // Único efeito reativo: re-fetch e reset da seleção quando o mês muda.
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

<section class="p-8 max-w-6xl mx-auto flex flex-col gap-5">
  <header class="flex items-baseline justify-between gap-4 flex-wrap">
    <div class="flex flex-col gap-1">
      <h2 class="text-xl font-semibold tracking-tight" style="font-family: var(--font-display)">
        {t("nav.calendar")}
      </h2>
      <p class="text-xs text-fg-faint max-w-xl">
        {t("calendar_page.desc_1")} <strong>{t("calendar_page.desc_strong")}</strong> {t("calendar_page.desc_2")}
      </p>
    </div>
    <MonthStepper month={viewMonth} onChange={onMonthChange} />
  </header>

  {#if error}
    <div class="rounded-lg border border-border bg-surface p-3 text-sm text-neg">{error}</div>
  {/if}

  <!-- Resumo do mês compacto (sempre visível, mesmo em mês vazio) -->
  <div class="rounded-lg border border-border-subtle bg-surface px-4 py-2.5 flex items-center gap-6 text-[12px]">
    <div class="flex items-center gap-2">
      <span class="text-fg-faint text-[10.5px] uppercase tracking-wider">{t("calendar_page.inflows")}</span>
      <span class="tabular text-pos font-medium">
        {monthTotals.inflow > 0 ? formatMoney(String(monthTotals.inflow)) : "—"}
      </span>
    </div>
    <div class="flex items-center gap-2">
      <span class="text-fg-faint text-[10.5px] uppercase tracking-wider">{t("calendar_page.outflows")}</span>
      <span class="tabular text-neg font-medium">
        {monthTotals.outflow > 0 ? formatMoney(String(monthTotals.outflow)) : "—"}
      </span>
    </div>
    <div class="flex items-center gap-2 ml-auto">
      {#if loading}
        <span class="text-fg-faint text-[11px]">{t("calendar_page.loading")}</span>
      {/if}
      <span class="text-fg-faint text-[10.5px] uppercase tracking-wider">{t("calendar_page.net")}</span>
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
    />

    <aside class="flex flex-col gap-4">
      <DayDetails {selectedDate} {transactions} {categories} {events} {today} />
    </aside>
  </div>
</section>
