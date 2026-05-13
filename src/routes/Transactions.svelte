<script lang="ts">
  import { onMount } from "svelte";
  import TxTable from "$lib/components/transactions/TxTable.svelte";
  import TxFilterBar from "$lib/components/transactions/TxFilterBar.svelte";
  import TxNotesPanel from "$lib/components/transactions/TxNotesPanel.svelte";
  import { filters } from "$lib/stores/filters.svelte";
  import { listCategories, createCategory } from "$lib/api/categories";
  import {
    listTransactions,
    updateTransactionCategory,
    updateTransactionNotes,
  } from "$lib/api/transactions";
  import type { Category, Transaction } from "$lib/bindings";

  let transactions = $state<Transaction[]>([]);
  let categories = $state<Category[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let selectedTx = $state<Transaction | null>(null);

  let searchInputRef = $state<HTMLInputElement | null>(null);

  async function refresh() {
    try {
      transactions = await listTransactions({
        account_id: null,
        month: filters.month,
        category_id: filters.categoryId,
        q: filters.q === "" ? null : filters.q,
        limit: null,
      });
      if (selectedTx) {
        const fresh = transactions.find((t) => t.id === selectedTx?.id);
        selectedTx = fresh ?? null;
      }
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  onMount(async () => {
    try {
      categories = await listCategories();
      await refresh();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  });

  async function onCategoryChange(transactionId: number, categoryId: number | null) {
    await updateTransactionCategory(transactionId, categoryId);
    transactions = transactions.map((t) =>
      t.id === transactionId ? { ...t, category_id: categoryId } : t,
    );
    if (filters.categoryId !== null && filters.categoryId !== categoryId) {
      await refresh();
    }
  }

  async function onCategoryCreate(name: string): Promise<Category> {
    const created = await createCategory({
      name,
      color_token: "--color-cat-outros",
      kind: "expense",
    });
    categories = [...categories, created];
    return created;
  }

  async function onMonthChange(m: string | null) {
    filters.month = m;
    await refresh();
  }
  async function onCategoryFilterChange(id: number | null) {
    filters.categoryId = id;
    await refresh();
  }

  async function onQueryChange(v: string) {
    filters.q = v;
    await refresh();
  }

  function onRowClick(t: Transaction) {
    selectedTx = t;
  }
  function closePanel() {
    selectedTx = null;
  }
  async function onSaveNotes(transactionId: number, notes: string | null) {
    await updateTransactionNotes(transactionId, notes);
    transactions = transactions.map((t) =>
      t.id === transactionId ? { ...t, notes } : t,
    );
  }
</script>

<section class="p-8 max-w-5xl mx-auto flex flex-col gap-5">
  <header class="flex items-baseline justify-between gap-4 flex-wrap">
    <h2 class="text-xl font-semibold tracking-tight" style="font-family: var(--font-display)">
      Transações
    </h2>
    <span class="text-xs text-fg-faint tabular">
      {transactions.length} {transactions.length === 1 ? "transação" : "transações"}
    </span>
  </header>

  <TxFilterBar
    {categories}
    month={filters.month}
    categoryId={filters.categoryId}
    q={filters.q}
    {onMonthChange}
    onCategoryChange={onCategoryFilterChange}
    {onQueryChange}
    bind:searchInputRef
  />

  {#if loading}
    <div class="text-fg-faint text-sm">Carregando…</div>
  {:else if error}
    <div class="rounded-lg border border-border bg-surface p-3 text-sm text-neg">{error}</div>
  {:else}
    <TxTable
      {transactions}
      {categories}
      {onCategoryChange}
      {onCategoryCreate}
      {onRowClick}
      selectedId={selectedTx?.id ?? null}
    />
  {/if}
</section>

{#if selectedTx}
  <TxNotesPanel transaction={selectedTx} onClose={closePanel} onSave={onSaveNotes} />
{/if}
