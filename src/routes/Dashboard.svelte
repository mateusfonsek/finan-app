<script lang="ts">
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
  import { listTransactions } from "$lib/api/transactions";
  import type {
    CategorySpend,
    IncomeSource,
    InvestmentSummary,
    KpiSummary,
    MonthSummary,
    Transaction,
    TransferSummary,
  } from "$lib/bindings";

  let kpis = $state<KpiSummary | null>(null);
  let byCategory = $state<CategorySpend[]>([]);
  let byMonth = $state<MonthSummary[]>([]);
  let recent = $state<Transaction[]>([]);
  let investments = $state<InvestmentSummary | null>(null);
  let transfers = $state<TransferSummary | null>(null);
  let sources = $state<IncomeSource[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  async function refresh() {
    try {
      const [k, c, recentTx, inv, tr, src] = await Promise.all([
        summaryKpis(filters.month),
        summaryByCategory(filters.month),
        listTransactions({
          account_id: null,
          month: filters.month,
          category_id: null,
          q: null,
          limit: 8,
        }),
        investmentSummary(filters.month),
        transferSummary(filters.month),
        incomeSources(filters.month),
      ]);
      kpis = k;
      byCategory = c;
      recent = recentTx;
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
      <!-- KPIs principais: só contam gasto/renda REAL. Subtítulo explica o que NÃO entra. -->
      <div class="grid grid-cols-4 gap-3">
        <KpiCard
          label="Renda"
          value={kpis.income}
          tone="pos"
          caption={Number(kpis.income) > 0 ? "entradas reais (sem transf. e investimentos)" : undefined}
        />
        <KpiCard
          label="Gastos"
          value={kpis.expense}
          caption={Number(kpis.expense) > 0 ? "saídas reais (sem transf. e investimentos)" : undefined}
        />
        <KpiCard
          label="Saldo do mês"
          value={kpis.net}
          tone={Number(kpis.net) >= 0 ? "pos" : "neg"}
          caption="renda − gastos"
        />
        <KpiCard
          label="Transações"
          value={String(kpis.transaction_count)}
          raw={true}
          caption={kpis.transaction_count === 1 ? "uma tx no período (s/ transf.)" : "tx no período (s/ transf.)"}
        />
      </div>

      <!-- Reconciliação: mostra que a matemática fecha. Duas colunas independentes,
           cada uma com seu próprio fluxo bruto → exclusões → resultado real. -->
      {#if (transfers && transfers.count > 0) || (investments && (investments.aplicacoes_count + investments.resgates_count) > 0)}
        <details class="rounded-lg border border-border-subtle bg-surface px-4 py-2.5">
          <summary class="text-[11.5px] text-fg-muted cursor-pointer flex items-center gap-2">
            <span class="text-fg-faint">↻</span>
            <span>Como esses números são calculados? <span class="text-fg-faint">(clique pra expandir)</span></span>
          </summary>
          <div class="pt-3 grid grid-cols-2 gap-x-6 text-[11.5px]">
            <!-- Coluna ENTRADAS -->
            <div class="flex flex-col gap-1">
              <div class="text-[10px] uppercase tracking-wider font-semibold text-fg-faint pb-0.5">
                Entradas
              </div>
              <div class="flex justify-between">
                <span class="text-fg-muted">Tudo que entrou (bruto)</span>
                <span class="tabular text-fg">{formatMoney(String(grossInflow))}</span>
              </div>
              {#if transfers && Number(transfers.total_in) > 0}
                <div class="flex justify-between text-fg-faint">
                  <span>− Transferências internas</span>
                  <span class="tabular">{formatMoney(transfers.total_in)}</span>
                </div>
              {/if}
              {#if investments && Number(investments.resgatado_no_mes) > 0}
                <div class="flex justify-between text-fg-faint">
                  <span>− Resgate de investimentos</span>
                  <span class="tabular">{formatMoney(investments.resgatado_no_mes)}</span>
                </div>
              {/if}
              <div class="flex justify-between border-t border-border-subtle pt-1 mt-0.5">
                <span class="text-fg-muted font-medium">= Renda real</span>
                <span class="tabular text-pos font-medium">{formatMoney(kpis.income)}</span>
              </div>
            </div>

            <!-- Coluna SAÍDAS -->
            <div class="flex flex-col gap-1">
              <div class="text-[10px] uppercase tracking-wider font-semibold text-fg-faint pb-0.5">
                Saídas
              </div>
              <div class="flex justify-between">
                <span class="text-fg-muted">Tudo que saiu (bruto)</span>
                <span class="tabular text-fg">{formatMoney(String(grossOutflow))}</span>
              </div>
              {#if transfers && Number(transfers.total_out) > 0}
                <div class="flex justify-between text-fg-faint">
                  <span>− Transferências internas</span>
                  <span class="tabular">{formatMoney(transfers.total_out)}</span>
                </div>
              {/if}
              {#if investments && Number(investments.aplicado_no_mes) > 0}
                <div class="flex justify-between text-fg-faint">
                  <span>− Aplicação em investimentos</span>
                  <span class="tabular">{formatMoney(investments.aplicado_no_mes)}</span>
                </div>
              {/if}
              <div class="flex justify-between border-t border-border-subtle pt-1 mt-0.5">
                <span class="text-fg-muted font-medium">= Gastos reais</span>
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
              Investimentos
            </span>
          </div>
          <span class="text-[10px] text-fg-faint">conta separada · não conta como gasto nem renda</span>
        </div>
        <div class="grid grid-cols-3 gap-4">
          <div class="flex flex-col gap-0.5">
            <span class="text-[10px] uppercase tracking-wider text-fg-faint">Aplicado no mês</span>
            <span class="text-[15px] tabular font-semibold" style="color: var(--color-cat-investimento);">
              {Number(investments.aplicado_no_mes) > 0 ? formatMoney(investments.aplicado_no_mes) : "—"}
            </span>
            <span class="text-[10px] text-fg-faint">
              {investments.aplicacoes_count} {investments.aplicacoes_count === 1 ? "aplicação" : "aplicações"}
            </span>
          </div>
          <div class="flex flex-col gap-0.5">
            <span class="text-[10px] uppercase tracking-wider text-fg-faint">Resgatado no mês</span>
            <span class="text-[15px] tabular font-semibold text-fg">
              {Number(investments.resgatado_no_mes) > 0 ? formatMoney(investments.resgatado_no_mes) : "—"}
            </span>
            <span class="text-[10px] text-fg-faint">
              {investments.resgates_count} {investments.resgates_count === 1 ? "resgate" : "resgates"}
            </span>
          </div>
          <div class="flex flex-col gap-0.5 border-l border-border-subtle pl-4">
            <span class="text-[10px] uppercase tracking-wider text-fg-faint" title="Aplicado no mês − Resgatado no mês. Positivo: investiu líquido. Negativo: retirou líquido do investimento.">
              Saldo investido
            </span>
            <span class="text-[15px] tabular font-semibold {investmentNetMonth >= 0 ? 'text-fg' : 'text-neg'}">
              {formatMoney(String(investmentNetMonth))}
            </span>
            <span class="text-[10px] text-fg-faint">
              no mês · aplicações − resgates
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
            Gastos por categoria
          </div>
          <span class="text-[10px] text-fg-faint">{byCategory.length} {byCategory.length === 1 ? "categoria" : "categorias"}</span>
        </div>
        <CategoryDonut items={byCategory} total={kpis?.expense ?? "0"} />
      </div>
      <div class="rounded-xl bg-surface border border-border-subtle p-4 flex flex-col gap-3">
        <div class="flex items-baseline justify-between">
          <div class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint">
            Fontes de renda
          </div>
          <span class="text-[10px] text-fg-faint">{sources.length} {sources.length === 1 ? "fonte" : "fontes"}</span>
        </div>
        <IncomeSourcesPanel items={sources} total={kpis?.income ?? "0"} />
      </div>
    </div>

    <!-- Tendência mensal: barras dos últimos 12 meses (full width). -->
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

    <div class="grid grid-cols-[1fr_360px] gap-4">
      <div class="rounded-xl bg-surface border border-border-subtle p-4 flex flex-col gap-3">
        <div class="flex items-baseline justify-between">
          <div class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint">
            Top categorias do mês
          </div>
          <span class="text-[10px] text-fg-faint">por valor</span>
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
