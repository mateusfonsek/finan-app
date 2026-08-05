<script lang="ts">
  import { push } from "svelte-spa-router";
  import { formatMoney } from "$lib/format/money";
  import { locale } from "$lib/i18n/locale.svelte";
  import { Button } from "$lib/components/ui/button";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
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
</script>

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
          <th class="col-head text-left px-4 py-2 w-[124px]">{t("tx_table.date")}</th>
          <th class="col-head text-left px-4 py-2">{t("tx_table.description")}</th>
          <th class="col-head text-left px-4 py-2 w-[184px]">{t("tx_table.category")}</th>
          <th class="col-head text-right px-4 py-2 w-[136px]">{t("tx_table.amount")}</th>
        </tr>
      </thead>
      <tbody>
        {#each transactions as t (t.id)}
          {@const selected = selectedId === t.id}
          <!-- A linha inteira é o alvo do clique (menos a célula de categoria,
               que tem controle próprio) — alvo grande, como em listas do macOS. -->
          <tr
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
