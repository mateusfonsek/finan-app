<script lang="ts">
  import { onMount } from "svelte";
  import MonthStepper from "$lib/components/shell/MonthStepper.svelte";
  import KpiCard from "$lib/components/dashboard/KpiCard.svelte";
  import CategoryDonut from "$lib/components/dashboard/CategoryDonut.svelte";
  import MonthBars from "$lib/components/dashboard/MonthBars.svelte";
  import TopCategoriesList from "$lib/components/dashboard/TopCategoriesList.svelte";
  import RecentList from "$lib/components/dashboard/RecentList.svelte";
  import { filters } from "$lib/stores/filters.svelte";
  import { summaryByCategory, summaryByMonth, summaryKpis } from "$lib/api/summary";
  import { listTransactions } from "$lib/api/transactions";
  import type {
    CategorySpend,
    KpiSummary,
    MonthSummary,
    Transaction,
  } from "$lib/bindings";

  let kpis = $state<KpiSummary | null>(null);
  let byCategory = $state<CategorySpend[]>([]);
  let byMonth = $state<MonthSummary[]>([]);
  let recent = $state<Transaction[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  async function refresh() {
    try {
      const [k, c, recentTx] = await Promise.all([
        summaryKpis(filters.month),
        summaryByCategory(filters.month),
        listTransactions({
          account_id: null,
          month: filters.month,
          category_id: null,
          limit: 8,
        }),
      ]);
      kpis = k;
      byCategory = c;
      recent = recentTx;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  onMount(async () => {
    try {
      byMonth = await summaryByMonth(12);
      await refresh();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  });

  async function onMonthChange(m: string | null) {
    filters.month = m;
    await refresh();
  }
</script>

<section class="p-8 max-w-6xl mx-auto flex flex-col gap-5">
  <header class="flex items-baseline justify-between gap-4 flex-wrap">
    <h2 class="text-xl font-semibold tracking-tight" style="font-family: var(--font-display)">
      Dashboard
    </h2>
    <MonthStepper month={filters.month} onChange={onMonthChange} />
  </header>

  {#if loading}
    <div class="text-fg-faint text-sm">Carregando…</div>
  {:else if error}
    <div class="rounded-lg border border-border bg-surface p-3 text-sm text-neg">{error}</div>
  {:else}
    {#if kpis}
      <div class="grid grid-cols-4 gap-3">
        <KpiCard label="Renda" value={kpis.income} tone="pos" />
        <KpiCard label="Gastos" value={kpis.expense} />
        <KpiCard
          label="Saldo do mês"
          value={kpis.net}
          tone={Number(kpis.net) >= 0 ? "pos" : "neg"}
        />
        <KpiCard
          label="Transações"
          value={String(kpis.transaction_count)}
          raw={true}
          caption={kpis.transaction_count === 1 ? "uma transação" : "transações no período"}
        />
      </div>
    {/if}

    <div class="grid grid-cols-[380px_1fr] gap-4">
      <div class="rounded-xl bg-surface border border-border-subtle p-4 flex flex-col gap-3">
        <div class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint">
          Gastos por categoria
        </div>
        <CategoryDonut items={byCategory} total={kpis?.expense ?? "0"} />
      </div>
      <div class="rounded-xl bg-surface border border-border-subtle p-4 flex flex-col gap-3">
        <div class="flex items-baseline justify-between">
          <div class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint">
            Últimos 12 meses
          </div>
          <div class="text-[10px] text-fg-faint flex gap-3">
            <span class="flex items-center gap-1.5"><span class="w-2 h-2 rounded bg-pos"></span> entradas</span>
            <span class="flex items-center gap-1.5"><span class="w-2 h-2 rounded bg-neg opacity-60"></span> saídas</span>
          </div>
        </div>
        <MonthBars months={byMonth} />
      </div>
    </div>

    <div class="grid grid-cols-[1fr_360px] gap-4">
      <div class="rounded-xl bg-surface border border-border-subtle p-4 flex flex-col gap-3">
        <div class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint">
          Top categorias do mês
        </div>
        <TopCategoriesList items={byCategory} />
      </div>
      <div class="rounded-xl bg-surface border border-border-subtle flex flex-col">
        <div class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint px-4 pt-4 pb-2">
          Últimas transações
        </div>
        <RecentList transactions={recent} />
      </div>
    </div>
  {/if}
</section>
