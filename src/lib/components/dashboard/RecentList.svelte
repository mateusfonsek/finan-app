<script lang="ts">
  import { formatMoney } from "$lib/format/money";
  import type { Transaction } from "$lib/bindings";

  type Props = {
    transactions: Transaction[];
  };

  let { transactions }: Props = $props();
</script>

<ul class="flex flex-col">
  {#each transactions as t (t.id)}
    <li class="grid grid-cols-[68px_1fr_100px] gap-3 items-center px-3 py-2 border-b border-border-subtle last:border-b-0">
      <span class="text-[11px] text-fg-muted tabular">{t.date}</span>
      <span class="text-[12px] text-fg truncate">{t.description}</span>
      <span class="text-[12px] text-right tabular font-medium {Number(t.amount) >= 0 ? 'text-pos' : 'text-fg'}">
        {formatMoney(t.amount)}
      </span>
    </li>
  {:else}
    <li class="text-fg-faint italic text-[12px] px-3 py-4">Nenhuma transação ainda.</li>
  {/each}
</ul>
