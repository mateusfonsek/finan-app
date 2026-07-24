<script lang="ts">
  import { locale } from "$lib/i18n/locale.svelte";

  const t = locale.t;
  import { onMount } from "svelte";
  import MonthStepper from "$lib/components/shell/MonthStepper.svelte";
  import KpiCard from "$lib/components/dashboard/KpiCard.svelte";
  import CategoryDonut from "$lib/components/dashboard/CategoryDonut.svelte";
  import MonthBars from "$lib/components/dashboard/MonthBars.svelte";
  import TopCategoriesList from "$lib/components/dashboard/TopCategoriesList.svelte";
  import IncomeSourcesPanel from "$lib/components/dashboard/IncomeSourcesPanel.svelte";
  import RecentList from "$lib/components/dashboard/RecentList.svelte";
  import { formatMoney } from "$lib/format/money";
  import { filters } from "$lib/stores/filters.svelte";
  import {
    incomeSources,
    investmentSummary,
    summaryByCategory,
    summaryByMonth,
    summaryKpis,
    transferSummary,
  } from "$lib/api/summary";
  import { topExpenses } from "$lib/api/transactions";
  import type {
    CategorySpend,
    ExpenseRow,
    IncomeSource,
    InvestmentSummary,
    KpiSummary,
    MonthSummary,
    TransferSummary,
  } from "$lib/bindings";

  let kpis = $state<KpiSummary | null>(null);
  let byCategory = $state<CategorySpend[]>([]);
  let byMonth = $state<MonthSummary[]>([]);
  let topSpends = $state<ExpenseRow[]>([]);
  let investments = $state<InvestmentSummary | null>(null);
  let transfers = $state<TransferSummary | null>(null);
  let sources = $state<IncomeSource[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  async function refresh() {
    try {
      const [k, c, topTx, inv, tr, src] = await Promise.all([
        summaryKpis(filters.month),
        summaryByCategory(filters.month),
        topExpenses(filters.month, 8),
        investmentSummary(filters.month),
        transferSummary(filters.month),
        incomeSources(filters.month),
      ]);
      kpis = k;
      byCategory = c;
      topSpends = topTx;
      investments = inv;
      transfers = tr;
      sources = src;
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

  /** Movimentação bruta = entradas reais + transferências/investimentos in. Para reconciliar com cálculo manual. */
  let grossInflow = $derived.by(() => {
    if (!kpis || !investments || !transfers) return 0;
    return Number(kpis.income) + Number(investments.resgatado_no_mes) + Number(transfers.total_in);
  });
  let grossOutflow = $derived.by(() => {
    if (!kpis || !investments || !transfers) return 0;
    return Number(kpis.expense) + Number(investments.aplicado_no_mes) + Number(transfers.total_out);
  });

  /** Saldo investido NO MÊS: aplicado − resgatado (positivo = entrou mais que saiu de investimento). */
  let investmentNetMonth = $derived.by(() => {
    if (!investments) return 0;
    return Number(investments.aplicado_no_mes) - Number(investments.resgatado_no_mes);
  });
</script>

<section class="p-8 max-w-6xl mx-auto flex flex-col gap-5">
  <header class="flex items-baseline justify-between gap-4 flex-wrap">
    <h2 class="text-xl font-semibold tracking-tight" style="font-family: var(--font-display)">
      {t("nav.dashboard")}
    </h2>
    <MonthStepper month={filters.month} onChange={onMonthChange} />
  </header>

  {#if loading}
    <div class="text-fg-faint text-sm">{t("common.loading")}</div>
  {:else if error}
    <div class="rounded-lg border border-border bg-surface p-3 text-sm text-neg">{error}</div>
  {:else}
    {#if kpis}
      <!-- KPIs principais: só contam gasto/renda REAL. Subtítulo explica o que NÃO entra. -->
      <div class="grid grid-cols-4 gap-3">
        <KpiCard
          label={t("dashboard.kpi_income")}
          value={kpis.income}
          tone="pos"
          caption={Number(kpis.income) > 0 ? t("dashboard.caption_income") : undefined}
        />
        <KpiCard
          label={t("dashboard.kpi_expenses")}
          value={kpis.expense}
          caption={Number(kpis.expense) > 0 ? t("dashboard.caption_expenses") : undefined}
        />
        <KpiCard
          label={t("dashboard.kpi_balance")}
          value={kpis.net}
          tone={Number(kpis.net) >= 0 ? "pos" : "neg"}
          caption={t("dashboard.caption_balance")}
        />
        <KpiCard
          label={t("dashboard.kpi_transactions")}
          value={String(kpis.transaction_count)}
          raw={true}
          caption={kpis.transaction_count === 1 ? t("dashboard.caption_tx_one") : t("dashboard.caption_tx_many")}
        />
      </div>

      <!-- Reconciliação: mostra que a matemática fecha. Duas colunas independentes,
           cada uma com seu próprio fluxo bruto → exclusões → resultado real. -->
      {#if (transfers && transfers.count > 0) || (investments && (investments.aplicacoes_count + investments.resgates_count) > 0)}
        <details class="rounded-lg border border-border-subtle bg-surface px-4 py-2.5">
          <summary class="text-[11.5px] text-fg-muted flex items-center gap-2">
            <span class="text-fg-faint">↻</span>
            <span>{t("dashboard.calc_title")} <span class="text-fg-faint">{t("dashboard.calc_hint")}</span></span>
          </summary>
          <div class="pt-3 grid grid-cols-2 gap-x-6 text-[11.5px]">
            <!-- Coluna ENTRADAS -->
            <div class="flex flex-col gap-1">
              <div class="text-[10px] uppercase tracking-wider font-semibold text-fg-faint pb-0.5">
                {t("dashboard.inflows")}
              </div>
              <div class="flex justify-between">
                <span class="text-fg-muted">{t("dashboard.gross_in")}</span>
                <span class="tabular text-fg">{formatMoney(String(grossInflow))}</span>
              </div>
              {#if transfers && Number(transfers.total_in) > 0}
                <div class="flex justify-between text-fg-faint">
                  <span>{t("dashboard.minus_transfers")}</span>
                  <span class="tabular">{formatMoney(transfers.total_in)}</span>
                </div>
              {/if}
              {#if investments && Number(investments.resgatado_no_mes) > 0}
                <div class="flex justify-between text-fg-faint">
                  <span>{t("dashboard.minus_redemption")}</span>
                  <span class="tabular">{formatMoney(investments.resgatado_no_mes)}</span>
                </div>
              {/if}
              <div class="flex justify-between border-t border-border-subtle pt-1 mt-0.5">
                <span class="text-fg-muted font-medium">{t("dashboard.real_income")}</span>
                <span class="tabular text-pos font-medium">{formatMoney(kpis.income)}</span>
              </div>
            </div>

            <!-- Coluna SAÍDAS -->
            <div class="flex flex-col gap-1">
              <div class="text-[10px] uppercase tracking-wider font-semibold text-fg-faint pb-0.5">
                {t("dashboard.outflows")}
              </div>
              <div class="flex justify-between">
                <span class="text-fg-muted">{t("dashboard.gross_out")}</span>
                <span class="tabular text-fg">{formatMoney(String(grossOutflow))}</span>
              </div>
              {#if transfers && Number(transfers.total_out) > 0}
                <div class="flex justify-between text-fg-faint">
                  <span>{t("dashboard.minus_transfers")}</span>
                  <span class="tabular">{formatMoney(transfers.total_out)}</span>
                </div>
              {/if}
              {#if investments && Number(investments.aplicado_no_mes) > 0}
                <div class="flex justify-between text-fg-faint">
                  <span>{t("dashboard.minus_application")}</span>
                  <span class="tabular">{formatMoney(investments.aplicado_no_mes)}</span>
                </div>
              {/if}
              <div class="flex justify-between border-t border-border-subtle pt-1 mt-0.5">
                <span class="text-fg-muted font-medium">{t("dashboard.real_expenses")}</span>
                <span class="tabular text-neg font-medium">{formatMoney(kpis.expense)}</span>
              </div>
            </div>
          </div>
        </details>
      {/if}
    {/if}

    <!-- Seção dedicada de Investimentos (kind=transfer + is_investment=1). -->
    {#if investments && investments.aplicacoes_count + investments.resgates_count > 0}
      <div class="rounded-xl bg-surface border p-4 flex flex-col gap-3" style="border-color: color-mix(in oklch, var(--color-cat-investimento) 30%, var(--color-border-subtle));">
        <div class="flex items-baseline justify-between">
          <div class="flex items-center gap-2">
            <span class="w-2 h-2 rounded-full" style="background: var(--color-cat-investimento);"></span>
            <span class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint">
              {t("dashboard.investments")}
            </span>
          </div>
          <span class="text-[10px] text-fg-faint">{t("dashboard.investments_note")}</span>
        </div>
        <div class="grid grid-cols-3 gap-4">
          <div class="flex flex-col gap-0.5">
            <span class="text-[10px] uppercase tracking-wider text-fg-faint">{t("dashboard.applied_month")}</span>
            <span class="text-[15px] tabular font-semibold" style="color: var(--color-cat-investimento);">
              {Number(investments.aplicado_no_mes) > 0 ? formatMoney(investments.aplicado_no_mes) : "—"}
            </span>
            <span class="text-[10px] text-fg-faint">
              {investments.aplicacoes_count === 1 ? t("dashboard.applications_one", { n: investments.aplicacoes_count }) : t("dashboard.applications_many", { n: investments.aplicacoes_count })}
            </span>
          </div>
          <div class="flex flex-col gap-0.5">
            <span class="text-[10px] uppercase tracking-wider text-fg-faint">{t("dashboard.redeemed_month")}</span>
            <span class="text-[15px] tabular font-semibold text-fg">
              {Number(investments.resgatado_no_mes) > 0 ? formatMoney(investments.resgatado_no_mes) : "—"}
            </span>
            <span class="text-[10px] text-fg-faint">
              {investments.resgates_count === 1 ? t("dashboard.redemptions_one", { n: investments.resgates_count }) : t("dashboard.redemptions_many", { n: investments.resgates_count })}
            </span>
          </div>
          <div class="flex flex-col gap-0.5 border-l border-border-subtle pl-4">
            <span class="text-[10px] uppercase tracking-wider text-fg-faint" title={t("dashboard.invested_balance_title")}>
              {t("dashboard.invested_balance")}
            </span>
            <span class="text-[15px] tabular font-semibold {investmentNetMonth >= 0 ? 'text-fg' : 'text-neg'}">
              {formatMoney(String(investmentNetMonth))}
            </span>
            <span class="text-[10px] text-fg-faint">
              {t("dashboard.invested_balance_sub")}
            </span>
          </div>
        </div>
      </div>
    {/if}

    <!-- Donut Gastos + Painel Fontes de Renda lado a lado, visualmente simétricos:
         um pra onde o dinheiro foi (categorias), outro pra de onde veio (fontes). -->
    <div class="grid grid-cols-2 gap-4">
      <div class="rounded-xl bg-surface border border-border-subtle p-4 flex flex-col gap-3">
        <div class="flex items-baseline justify-between">
          <div class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint">
            {t("dashboard.spend_by_category")}
          </div>
          <span class="text-[10px] text-fg-faint">{byCategory.length === 1 ? t("dashboard.categories_one", { n: byCategory.length }) : t("dashboard.categories_many", { n: byCategory.length })}</span>
        </div>
        <CategoryDonut items={byCategory} total={kpis?.expense ?? "0"} />
      </div>
      <div class="rounded-xl bg-surface border border-border-subtle p-4 flex flex-col gap-3">
        <div class="flex items-baseline justify-between">
          <div class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint">
            {t("dashboard.income_sources")}
          </div>
          <span class="text-[10px] text-fg-faint">{sources.length === 1 ? t("dashboard.sources_one", { n: sources.length }) : t("dashboard.sources_many", { n: sources.length })}</span>
        </div>
        <IncomeSourcesPanel items={sources} total={kpis?.income ?? "0"} />
      </div>
    </div>

    <!-- Tendência mensal: barras dos últimos 12 meses (full width). -->
    <div class="rounded-xl bg-surface border border-border-subtle p-4 flex flex-col gap-3">
      <div class="flex items-baseline justify-between">
        <div class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint">
          {t("dashboard.last_12m")}
        </div>
        <div class="text-[10px] text-fg-faint flex gap-3">
          <span class="flex items-center gap-1.5"><span class="w-2 h-2 rounded bg-pos"></span> {t("dashboard.legend_inflows")}</span>
          <span class="flex items-center gap-1.5"><span class="w-2 h-2 rounded bg-neg opacity-60"></span> {t("dashboard.legend_outflows")}</span>
        </div>
      </div>
      <MonthBars months={byMonth} />
    </div>

    <div class="grid grid-cols-[1fr_360px] gap-4">
      <div class="rounded-xl bg-surface border border-border-subtle p-4 flex flex-col gap-3">
        <div class="flex items-baseline justify-between">
          <div class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint">
            {t("dashboard.top_categories")}
          </div>
          <span class="text-[10px] text-fg-faint">{t("dashboard.by_value")}</span>
        </div>
        <div class="h-[260px] overflow-y-auto pr-1">
          <TopCategoriesList items={byCategory} />
        </div>
      </div>
      <div class="rounded-xl bg-surface border border-border-subtle flex flex-col">
        <div class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint px-4 pt-4 pb-2 flex items-baseline justify-between">
          <span>{t("dashboard.biggest_expenses")}</span>
          <span class="text-[10px] normal-case tracking-normal text-fg-faint font-normal">{t("dashboard.no_transf_invest")}</span>
        </div>
        <div class="h-[260px] overflow-y-auto">
          <RecentList transactions={topSpends} />
        </div>
      </div>
    </div>
  {/if}
</section>
