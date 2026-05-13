<script lang="ts">
  import { formatMoney } from "$lib/format/money";
  import type { Transaction } from "$lib/bindings";

  let { transactions }: { transactions: Transaction[] } = $props();
</script>

<div class="rounded-lg border border-border-subtle bg-surface overflow-hidden">
  <table class="w-full text-[12px]">
    <thead class="bg-surface-2">
      <tr>
        <th class="text-left px-4 py-2 font-medium text-fg-faint uppercase tracking-wider text-[10.5px] w-[100px]">Data</th>
        <th class="text-left px-4 py-2 font-medium text-fg-faint uppercase tracking-wider text-[10.5px]">Descrição</th>
        <th class="text-right px-4 py-2 font-medium text-fg-faint uppercase tracking-wider text-[10.5px] w-[140px]">Valor</th>
      </tr>
    </thead>
    <tbody>
      {#each transactions as t (t.id)}
        <tr class="border-t border-border-subtle hover:bg-hover">
          <td class="px-4 py-2.5 text-fg-muted tabular">{t.date}</td>
          <td class="px-4 py-2.5">{t.description}</td>
          <td class="px-4 py-2.5 text-right tabular font-medium {Number(t.amount) >= 0 ? 'text-pos' : 'text-fg'}">
            {formatMoney(t.amount)}
          </td>
        </tr>
      {:else}
        <tr>
          <td colspan="3" class="px-4 py-10 text-center text-fg-faint">
            Nenhuma transação ainda. <a href="#/import" class="text-accent hover:underline">Importar um OFX</a>?
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>
