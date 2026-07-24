<script lang="ts">
  import { formatMoney } from "$lib/format/money";
  import { locale } from "$lib/i18n/locale.svelte";
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
</script>

<div class="rounded-lg border border-border-subtle bg-surface overflow-hidden">
  <table class="w-full text-[12px]">
    <thead class="bg-surface-2">
      <tr>
        <th class="text-left px-4 py-2 font-medium text-fg-faint uppercase tracking-wider text-[10.5px] w-[100px]">{t("tx_table.date")}</th>
        <th class="text-left px-4 py-2 font-medium text-fg-faint uppercase tracking-wider text-[10.5px]">{t("tx_table.description")}</th>
        <th class="text-left px-4 py-2 font-medium text-fg-faint uppercase tracking-wider text-[10.5px] w-[180px]">{t("tx_table.category")}</th>
        <th class="text-right px-4 py-2 font-medium text-fg-faint uppercase tracking-wider text-[10.5px] w-[140px]">{t("tx_table.amount")}</th>
      </tr>
    </thead>
    <tbody>
      {#each transactions as t (t.id)}
        <tr
          class="border-t border-border-subtle hover:bg-hover {selectedId === t.id ? 'bg-accent-soft' : ''}"
        >
          <td class="px-4 py-2.5 text-fg-muted tabular" onclick={() => onRowClick?.(t)}>
            {t.date}
          </td>
          <td class="px-4 py-2.5" onclick={() => onRowClick?.(t)}>
            {t.description}
          </td>
          <td class="px-4 py-2.5">
            <CategoryPicker
              categories={categories}
              currentId={t.category_id}
              onselect={(catId) => onCategoryChange(t.id, catId)}
              oncreate={onCategoryCreate}
            />
          </td>
          <td
            class="px-4 py-2.5 text-right tabular font-medium {Number(t.amount) >= 0 ? 'text-pos' : 'text-fg'}"
            onclick={() => onRowClick?.(t)}
          >
            {formatMoney(t.amount)}
          </td>
        </tr>
      {:else}
        <tr>
          <td colspan="4" class="px-4 py-10 text-center text-fg-faint">
            {t("tx_table.empty")} <a href="#/import" class="text-accent hover:underline">{t("tx_table.import_link")}</a>?
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>
