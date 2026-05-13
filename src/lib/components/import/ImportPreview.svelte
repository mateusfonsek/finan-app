<script lang="ts">
  import { formatMoney } from "$lib/format/money";
  import type { ParsedTransaction } from "$lib/ofx/types";

  type Props = {
    transactions: ParsedTransaction[];
    duplicateFitids: Set<string>;
    selected: Set<string>;
    ontoggle: (fitid: string) => void;
    ontoggleAll: (checked: boolean) => void;
  };

  let { transactions, duplicateFitids, selected, ontoggle, ontoggleAll }: Props = $props();

  let allChecked = $derived(
    transactions.length > 0 && transactions.every((t) => !t.fitid || selected.has(t.fitid)),
  );
</script>

<div class="rounded-lg border border-border-subtle bg-surface overflow-hidden">
  <table class="w-full text-[12px]">
    <thead class="bg-surface-2">
      <tr>
        <th class="text-left px-3 py-2 w-8">
          <input
            type="checkbox"
            checked={allChecked}
            onchange={(e) => ontoggleAll((e.currentTarget as HTMLInputElement).checked)}
          />
        </th>
        <th class="text-left px-3 py-2 font-medium text-fg-faint uppercase tracking-wider text-[10.5px]">Data</th>
        <th class="text-left px-3 py-2 font-medium text-fg-faint uppercase tracking-wider text-[10.5px]">Descrição</th>
        <th class="text-right px-3 py-2 font-medium text-fg-faint uppercase tracking-wider text-[10.5px]">Valor</th>
      </tr>
    </thead>
    <tbody>
      {#each transactions as t (t.fitid ?? `${t.date}-${t.amount}-${t.description}`)}
        {@const isDup = !!(t.fitid && duplicateFitids.has(t.fitid))}
        {@const isSel = !!(t.fitid && selected.has(t.fitid))}
        <tr class="border-t border-border-subtle {isDup ? 'opacity-60' : ''}">
          <td class="px-3 py-2">
            <input
              type="checkbox"
              checked={isSel}
              disabled={!t.fitid}
              onchange={() => t.fitid && ontoggle(t.fitid)}
            />
          </td>
          <td class="px-3 py-2 text-fg-muted tabular">{t.date}</td>
          <td class="px-3 py-2">
            {t.description}
            {#if isDup}
              <span class="ml-2 text-[10px] text-fg-faint uppercase tracking-wider">duplicada</span>
            {/if}
          </td>
          <td class="px-3 py-2 text-right tabular font-medium {Number(t.amount) >= 0 ? 'text-pos' : 'text-fg'}">
            {formatMoney(t.amount)}
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>
