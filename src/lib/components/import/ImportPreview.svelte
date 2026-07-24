<script lang="ts">
  import { formatMoney } from "$lib/format/money";
  import { txKeyString } from "$lib/api/transactions";
  import { locale } from "$lib/i18n/locale.svelte";
  import type { ParsedTransaction } from "$lib/ofx/types";
  import type { ReversalInfo, ReversalRole } from "$lib/ofx/reversals";

  const tr = locale.t;

  type Props = {
    transactions: ParsedTransaction[];
    /** Chaves compostas `fitid|date|amount` que já existem na DB. */
    duplicateKeys: Set<string>;
    reversalMap?: Map<string, ReversalInfo>;
    selected: Set<string>;
    ontoggle: (fitid: string) => void;
    ontoggleAll: (checked: boolean) => void;
  };

  let {
    transactions,
    duplicateKeys,
    reversalMap = new Map(),
    selected,
    ontoggle,
    ontoggleAll,
  }: Props = $props();

  let allChecked = $derived(
    transactions.length > 0 && transactions.every((t) => !t.fitid || selected.has(t.fitid)),
  );

  function reversalLabel(role: ReversalRole): string {
    return tr("import.role_" + role);
  }

  function reversalTooltip(role: ReversalRole): string {
    return tr("import.reversal_" + role);
  }
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
        <th class="text-left px-3 py-2 font-medium text-fg-faint uppercase tracking-wider text-[10.5px]">{tr("tx_table.date")}</th>
        <th class="text-left px-3 py-2 font-medium text-fg-faint uppercase tracking-wider text-[10.5px]">{tr("tx_table.description")}</th>
        <th class="text-right px-3 py-2 font-medium text-fg-faint uppercase tracking-wider text-[10.5px]">{tr("tx_table.amount")}</th>
      </tr>
    </thead>
    <tbody>
      {#each transactions as t (t.fitid ?? `${t.date}-${t.amount}-${t.description}`)}
        {@const isDup = !!(t.fitid && duplicateKeys.has(txKeyString({ ofx_fitid: t.fitid, date: t.date, amount: t.amount })))}
        {@const isSel = !!(t.fitid && selected.has(t.fitid))}
        {@const rev = t.fitid ? reversalMap.get(t.fitid) : undefined}
        <tr
          class="border-t border-border-subtle {isDup ? 'opacity-60' : ''} {rev ? 'bg-amber-50/40 dark:bg-amber-950/20' : ''}"
          style={rev ? 'box-shadow: inset 3px 0 0 var(--color-cat-amarelo);' : ''}
          title={rev ? reversalTooltip(rev.role) : undefined}
        >
          <td class="px-3 py-2 pl-4">
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
            {#if rev}
              <span
                class="ml-2 text-[10px] uppercase tracking-wider font-semibold px-1.5 py-0.5 rounded"
                style="color: var(--color-cat-amarelo); border: 1px solid var(--color-cat-amarelo); opacity: 0.85;"
              >
                {reversalLabel(rev.role)}
              </span>
            {/if}
            {#if isDup}
              <span class="ml-2 text-[10px] text-fg-faint uppercase tracking-wider">{tr("import.duplicate")}</span>
            {/if}
          </td>
          <td class="px-3 py-2 text-right tabular font-medium {Number(t.amount) >= 0 ? 'text-pos' : 'text-fg'}">
            {formatMoney(t.amount)}
          </td>
        </tr>
      {/each}
    </tbody>
  </table>

  {#if reversalMap.size > 0}
    <div class="px-3 py-2 border-t border-border-subtle bg-surface-2 text-[11px] text-fg-muted flex items-center gap-2">
      <span class="inline-block w-2 h-2 rounded" style="background: var(--color-cat-amarelo);"></span>
      <span>
        {reversalMap.size / 2 === 1
          ? tr("import.reversal_legend_one", { n: reversalMap.size / 2 })
          : tr("import.reversal_legend_many", { n: reversalMap.size / 2 })}
      </span>
    </div>
  {/if}
</div>
