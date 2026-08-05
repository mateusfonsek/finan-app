<script lang="ts">
  import { push } from "svelte-spa-router";
  import { flip } from "svelte/animate";
  import { formatMoney } from "$lib/format/money";
  import { locale } from "$lib/i18n/locale.svelte";
  import { Button } from "$lib/components/ui/button";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import Icon from "$lib/components/ui/Icon.svelte";
  import { DUR, SNAP, reducedMotion } from "$lib/motion";
  import CategoryPicker from "./CategoryPicker.svelte";
  import type { Category, Transaction } from "$lib/bindings";

  const t = locale.t;

  type Props = {
    transactions: Transaction[];
    categories: Category[];
    onCategoryChange: (transactionId: number, categoryId: number | null) => Promise<void>;
    onCategoryCreate: (name: string) => Promise<Category>;
    onRowClick?: (transaction: Transaction) => void;
    selectedId?: number | null;
  };

  let {
    transactions,
    categories,
    onCategoryChange,
    onCategoryCreate,
    onRowClick,
    selectedId,
  }: Props = $props();

  /** "2026-08-14" → "14 ago 2026". Data por extenso curta lê mais rápido que
   *  ISO e ocupa quase o mesmo espaço. */
  function fmtDate(iso: string): string {
    const mo = Number(iso.slice(5, 7)) - 1;
    return `${iso.slice(8, 10)} ${(locale.monthsShort[mo] ?? "").toLowerCase()} ${iso.slice(0, 4)}`;
  }

  // ── Ordenação ────────────────────────────────────────────────────────────
  // Reordena no cliente: a lista já está toda em memória, então o clique
  // responde no mesmo quadro — nenhuma ida ao backend entre o dedo e o
  // resultado.
  type SortKey = "date" | "amount";
  type SortDir = "asc" | "desc";

  /** O padrão espelha a ordem que o backend já devolve (`date DESC, id DESC`),
   *  então a tabela abre exatamente como abria antes. */
  let sortKey = $state<SortKey>("date");
  let sortDir = $state<SortDir>("desc");

  /** Primeiro clique de cada coluna cai no sentido mais útil dela: data começa
   *  pela mais recente; valor começa pela maior saída (gastos são negativos,
   *  logo o mais negativo vem primeiro em ordem crescente). */
  const FIRST_DIR: Record<SortKey, SortDir> = { date: "desc", amount: "asc" };

  function toggleSort(key: SortKey) {
    if (sortKey === key) {
      sortDir = sortDir === "asc" ? "desc" : "asc";
    } else {
      sortKey = key;
      sortDir = FIRST_DIR[key];
    }
  }

  let sorted = $derived.by(() => {
    const sign = sortDir === "asc" ? 1 : -1;
    // Cópia: `transactions` é prop, ordenar no lugar mutaria o pai.
    return [...transactions].sort((a, b) => {
      let d = 0;
      if (sortKey === "date") {
        d = a.date < b.date ? -1 : a.date > b.date ? 1 : 0;
      } else {
        d = Number(a.amount) - Number(b.amount);
      }
      // Desempate estável por id, no mesmo sentido — duas transações do mesmo
      // dia (ou do mesmo valor) nunca trocam de lugar sozinhas entre renders.
      return d !== 0 ? sign * d : sign * (a.id - b.id);
    });
  });

  function ariaSort(key: SortKey): "ascending" | "descending" | "none" {
    if (sortKey !== key) return "none";
    return sortDir === "asc" ? "ascending" : "descending";
  }

  /** Rótulo do estado que o clique VAI produzir — o title antecipa o resultado
   *  em vez de descrever o atual. */
  function sortHint(key: SortKey, label: string): string {
    const next = sortKey === key ? (sortDir === "asc" ? "desc" : "asc") : FIRST_DIR[key];
    if (key === "date") {
      return t(next === "asc" ? "tx_table.sort_date_asc" : "tx_table.sort_date_desc");
    }
    return t(next === "asc" ? "tx_table.sort_asc" : "tx_table.sort_desc", { col: label });
  }

  let flipParams = $derived(
    reducedMotion() ? { duration: 0 } : { duration: DUR.base, easing: SNAP },
  );
</script>

<!-- Cabeçalho clicável. O indicador some quando a coluna não está ativa, mas o
     espaço dele fica reservado — assim nada empurra o texto ao aparecer. -->
{#snippet sortHead(key: SortKey, label: string, align: "left" | "right")}
  {@const active = sortKey === key}
  {@const icon = !active ? "chevronsUpDown" : sortDir === "asc" ? "chevronUp" : "chevronDown"}
  <button
    type="button"
    onclick={() => toggleSort(key)}
    title={sortHint(key, label)}
    class="col-head press-sm group flex w-full items-center gap-1 px-4 py-2 select-none
           rounded-[5px] transition-colors duration-[var(--dur-fast)] ease-[var(--ease-snap)]
           hover:text-fg {active ? 'text-fg' : ''}
           {align === 'right' ? 'flex-row-reverse justify-start text-right' : 'text-left'}"
  >
    <span>{label}</span>
    <Icon
      name={icon}
      size={12}
      stroke={2.2}
      class="transition-opacity duration-[var(--dur-fast)] ease-[var(--ease-snap)]
             {active ? 'opacity-100' : 'opacity-0 group-hover:opacity-45 group-focus-visible:opacity-45'}"
    />
  </button>
{/snippet}

<div class="card overflow-hidden">
  {#if transactions.length === 0}
    <EmptyState icon="inbox" title={t("tx_table.empty")} description={t("tx_table.empty_desc")}>
      {#snippet action()}
        <Button variant="outline" onclick={() => push("/import")}>
          {t("tx_table.import_link")}
        </Button>
      {/snippet}
    </EmptyState>
  {:else}
    <table class="w-full">
      <thead>
        <tr class="border-b border-border-subtle">
          <th class="w-[124px] p-0" aria-sort={ariaSort("date")}>
            {@render sortHead("date", t("tx_table.date"), "left")}
          </th>
          <th class="col-head text-left px-4 py-2">{t("tx_table.description")}</th>
          <th class="col-head text-left px-4 py-2 w-[184px]">{t("tx_table.category")}</th>
          <th class="w-[136px] p-0" aria-sort={ariaSort("amount")}>
            {@render sortHead("amount", t("tx_table.amount"), "right")}
          </th>
        </tr>
      </thead>
      <tbody>
        {#each sorted as t (t.id)}
          {@const selected = selectedId === t.id}
          <!-- A linha inteira é o alvo do clique (menos a célula de categoria,
               que tem controle próprio) — alvo grande, como em listas do macOS. -->
          <tr
            animate:flip={flipParams}
            class="row border-t border-border-subtle first:border-t-0 cursor-default
                   {selected ? 'bg-accent-soft hover:bg-accent-soft' : ''}"
            aria-selected={selected}
          >
            <td class="px-4 py-2 text-sub text-fg-subtle tabular whitespace-nowrap" onclick={() => onRowClick?.(t)}>
              {fmtDate(t.date)}
            </td>
            <td class="px-4 py-2 text-callout text-fg" onclick={() => onRowClick?.(t)}>
              <span class="line-clamp-2">{t.description}</span>
            </td>
            <td class="px-4 py-2">
              <CategoryPicker
                {categories}
                currentId={t.category_id}
                onselect={(catId) => onCategoryChange(t.id, catId)}
                oncreate={onCategoryCreate}
              />
            </td>
            <td
              class="px-4 py-2 text-right text-callout tabular font-medium whitespace-nowrap
                     {Number(t.amount) >= 0 ? 'text-pos' : 'text-fg'}"
              onclick={() => onRowClick?.(t)}
            >
              {formatMoney(t.amount)}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>
