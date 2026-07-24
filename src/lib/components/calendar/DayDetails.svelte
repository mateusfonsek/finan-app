<script lang="ts">
  import { formatMoney } from "$lib/format/money";
  import { locale } from "$lib/i18n/locale.svelte";
  import type { CalendarEvent, Category, Transaction } from "$lib/bindings";

  const t = locale.t;

  type Props = {
    selectedDate: string | null;
    transactions: Transaction[];
    categories: Category[];
    /** Eventos de regras do mês (pra mostrar contas vencendo no dia selecionado). */
    events?: CalendarEvent[];
    /** Hoje YYYY-MM-DD pra decidir overdue vs pending. */
    today?: string;
  };

  let { selectedDate, transactions, categories, events = [], today = "" }: Props = $props();

  /** Eventos com vencimento no dia selecionado. */
  let dueOnSelectedDay = $derived.by<CalendarEvent[]>(() => {
    if (!selectedDate) return [];
    const day = Number(selectedDate.slice(8, 10));
    return events.filter((e) => e.due_day === day);
  });

  type BillState = "paid" | "overdue" | "pending";

  function billState(e: CalendarEvent): BillState {
    if (e.paid_day != null) return "paid";
    if (!selectedDate || !today) return "pending";
    // Overdue se o dia selecionado é anterior a hoje E não foi paga.
    return selectedDate < today.slice(0, 10) ? "overdue" : "pending";
  }

  function billColor(state: BillState): string {
    return state === "paid"
      ? "var(--color-pos)"
      : state === "overdue"
        ? "var(--color-neg)"
        : "var(--color-cat-amarelo)";
  }

  function billStatusText(e: CalendarEvent, state: BillState): string {
    if (state === "paid") {
      return e.paid_amount
        ? t("day_details.paid_amount", { day: e.paid_day ?? "", amount: formatMoney(e.paid_amount) })
        : t("day_details.paid", { day: e.paid_day ?? "" });
    }
    return state === "overdue" ? t("day_details.overdue") : t("day_details.pending");
  }

  type Bucket = "gastos" | "renda" | "transfer" | "investimento";

  function bucketOf(tx: Transaction, cats: Category[]): Bucket {
    if (tx.category_id != null) {
      const c = cats.find((x) => x.id === tx.category_id);
      if (c?.is_investment) return "investimento";
      if (c?.kind === "transfer") return "transfer";
    }
    return Number(tx.amount) >= 0 ? "renda" : "gastos";
  }

  type Group = {
    bucket: Bucket;
    label: string;
    color: string;
    txs: Transaction[];
    total: number; // valor agregado para o header do grupo
  };

  let groups = $derived.by<Group[]>(() => {
    if (!selectedDate) return [];
    const day = transactions.filter((t) => t.date === selectedDate);
    const out: Group[] = [
      { bucket: "renda",        label: t("day_details.income"),       color: "var(--color-pos)",        txs: [], total: 0 },
      { bucket: "gastos",       label: t("day_details.expenses"),     color: "var(--color-neg)",        txs: [], total: 0 },
      { bucket: "transfer",     label: t("day_details.transfers"),    color: "var(--color-fg-faint)",   txs: [], total: 0 },
      { bucket: "investimento", label: t("day_details.investments"),  color: "var(--color-cat-investimento)", txs: [], total: 0 },
    ];
    for (const t of day) {
      const b = bucketOf(t, categories);
      const g = out.find((x) => x.bucket === b)!;
      g.txs.push(t);
      g.total += Number(t.amount);
    }
    // Magnitude desc dentro de cada grupo.
    for (const g of out) g.txs.sort((a, b) => Math.abs(Number(b.amount)) - Math.abs(Number(a.amount)));
    return out.filter((g) => g.txs.length > 0);
  });

  let totals = $derived.by(() => {
    const find = (b: Bucket) => groups.find((g) => g.bucket === b);
    const renda = find("renda")?.total ?? 0;
    const gastos = Math.abs(find("gastos")?.total ?? 0);
    return { renda, gastos, net: renda - gastos };
  });

  function categoryName(id: number | null): string {
    if (id == null) return t("day_details.no_category");
    return categories.find((c) => c.id === id)?.name ?? t("day_details.no_category");
  }

  function categoryToken(id: number | null): string {
    const t = id == null ? null : categories.find((c) => c.id === id)?.color_token;
    return t ? `var(${t})` : "var(--color-fg-faint)";
  }

  function formatDateLong(d: string): string {
    const [y, m, day] = d.split("-").map(Number);
    const date = new Date(y, m - 1, day);
    return date.toLocaleDateString(locale.dateLocale, {
      weekday: "long",
      day: "numeric",
      month: "long",
    });
  }

  let hasAny = $derived(groups.length > 0 || dueOnSelectedDay.length > 0);
</script>

<div class="rounded-lg border border-border-subtle bg-surface flex flex-col">
  <header class="px-4 pt-3 pb-2 flex flex-col gap-0.5 border-b border-border-subtle">
    <div class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint">
      {selectedDate ? t("day_details.selected_day") : t("day_details.select_day")}
    </div>
    {#if selectedDate}
      <div class="text-[13px] font-medium text-fg">{formatDateLong(selectedDate)}</div>
    {/if}
  </header>

  {#if !selectedDate}
    <div class="px-4 py-6 text-[12px] text-fg-faint italic">
      {t("day_details.empty_select")}
    </div>
  {:else if !hasAny}
    <div class="px-4 py-6 text-[12px] text-fg-faint italic">
      {t("day_details.empty_none")}
    </div>
  {:else}
    {#if dueOnSelectedDay.length > 0}
      <!-- Contas com vencimento neste dia, com estado visual claro -->
      <section class="border-b border-border-subtle">
        <div class="px-4 py-1.5 bg-surface-2/50 text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint">
          {t("day_details.due_bills")}
        </div>
        <ul>
          {#each dueOnSelectedDay as e (e.rule_id)}
            {@const state = billState(e)}
            {@const color = billColor(state)}
            <li
              class="px-4 py-2 border-t border-border-subtle first:border-t-0 flex items-start gap-2 min-w-0"
              style={state === "paid"
                ? "background: color-mix(in oklch, var(--color-pos) 6%, transparent);"
                : state === "overdue"
                  ? "background: color-mix(in oklch, var(--color-neg) 6%, transparent);"
                  : ""}
            >
              <span
                class="w-4 h-4 rounded-full grid place-items-center text-[10px] font-bold shrink-0 border"
                style="color: {color}; border-color: {color};"
              >
                {state === "paid" ? "✓" : state === "overdue" ? "!" : "○"}
              </span>
              <div class="flex-1 min-w-0 flex flex-col gap-0.5">
                <span class="text-[12px] text-fg font-medium truncate" title={e.pattern}>
                  {e.pattern}
                </span>
                <span class="text-[10px]" style="color: {color};">
                  {billStatusText(e, state)}
                </span>
              </div>
              {#if state === "paid" && e.paid_amount}
                <span class="text-[11.5px] tabular shrink-0" style="color: {color};">
                  {formatMoney(e.paid_amount)}
                </span>
              {/if}
            </li>
          {/each}
        </ul>
      </section>
    {/if}

    {#if groups.length > 0}
    <!-- Totais REAIS (excluem transfer/investimento) — alinhado com KPIs do Dashboard -->
    <div class="px-4 py-3 grid grid-cols-3 gap-2 border-b border-border-subtle">
      <div class="flex flex-col gap-0.5">
        <span class="text-[9.5px] uppercase tracking-wider text-fg-faint">{t("day_details.income")}</span>
        <span class="text-[11.5px] tabular text-pos font-medium">
          {totals.renda > 0 ? formatMoney(String(totals.renda)) : "—"}
        </span>
      </div>
      <div class="flex flex-col gap-0.5">
        <span class="text-[9.5px] uppercase tracking-wider text-fg-faint">{t("day_details.expenses")}</span>
        <span class="text-[11.5px] tabular text-neg font-medium">
          {totals.gastos > 0 ? formatMoney(String(totals.gastos)) : "—"}
        </span>
      </div>
      <div class="flex flex-col gap-0.5">
        <span class="text-[9.5px] uppercase tracking-wider text-fg-faint">{t("day_details.balance")}</span>
        <span class="text-[11.5px] tabular font-semibold {totals.net >= 0 ? 'text-pos' : 'text-neg'}">
          {formatMoney(String(totals.net))}
        </span>
      </div>
    </div>

    <!-- Grupos: cada um com header colorido + lista de tx -->
    {#each groups as g}
      <section class="border-b border-border-subtle last:border-b-0">
        <div class="px-4 py-1.5 bg-surface-2/50 flex items-center justify-between text-[10.5px]">
          <span class="uppercase tracking-wider font-semibold flex items-center gap-1.5">
            <span class="w-1.5 h-1.5 rounded-full" style="background: {g.color};"></span>
            <span style="color: {g.color};">{g.label}</span>
            <span class="text-fg-faint">· {g.txs.length}</span>
          </span>
          <span class="tabular text-fg-faint">
            {formatMoney(String(g.total))}
          </span>
        </div>
        <ul>
          {#each g.txs as t (t.id)}
            {@const n = Number(t.amount)}
            <li class="px-4 py-2 border-t border-border-subtle first:border-t-0 flex items-start gap-2 min-w-0">
              <span
                class="w-2 h-2 rounded-full shrink-0 mt-1.5"
                style="background: {categoryToken(t.category_id)}"
                title={categoryName(t.category_id)}
              ></span>
              <div class="flex-1 min-w-0 flex flex-col gap-0.5">
                <span class="text-[12px] text-fg truncate" title={t.description}>
                  {t.description}
                </span>
                <span class="text-[10px] text-fg-faint">
                  {categoryName(t.category_id)}
                </span>
              </div>
              <span class="text-[11.5px] tabular shrink-0 font-medium {n >= 0 ? 'text-pos' : 'text-fg'}">
                {formatMoney(t.amount)}
              </span>
            </li>
          {/each}
        </ul>
      </section>
    {/each}
    {/if}
  {/if}
</div>
