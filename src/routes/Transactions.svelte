<script lang="ts">
  import { onMount } from "svelte";
  import TxTable from "$lib/components/transactions/TxTable.svelte";
  import { listTransactions } from "$lib/api/transactions";
  import type { Transaction } from "$lib/bindings";

  let transactions = $state<Transaction[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      transactions = await listTransactions();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  });
</script>

<section class="p-8 max-w-5xl mx-auto flex flex-col gap-5">
  <header class="flex items-baseline justify-between">
    <h2 class="text-xl font-semibold tracking-tight" style="font-family: var(--font-display)">
      Transações
    </h2>
    <span class="text-xs text-fg-faint tabular">
      {transactions.length} {transactions.length === 1 ? "transação" : "transações"}
    </span>
  </header>

  {#if loading}
    <div class="text-fg-faint text-sm">Carregando…</div>
  {:else if error}
    <div class="rounded-lg border border-border bg-surface p-3 text-sm text-neg">{error}</div>
  {:else}
    <TxTable {transactions} />
  {/if}
</section>
