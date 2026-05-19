<script lang="ts">
  import { formatMoney } from "$lib/format/money";
  import type { ParsedTransaction } from "$lib/ofx/types";
  import type { ReversalInfo, ReversalRole } from "$lib/ofx/reversals";

  type Props = {
    transactions: ParsedTransaction[];
    duplicateFitids: Set<string>;
    reversalMap?: Map<string, ReversalInfo>;
    selected: Set<string>;
    ontoggle: (fitid: string) => void;
    ontoggleAll: (checked: boolean) => void;
  };

  let {
    transactions,
    duplicateFitids,
    reversalMap = new Map(),
    selected,
    ontoggle,
    ontoggleAll,
  }: Props = $props();

  let allChecked = $derived(
    transactions.length > 0 && transactions.every((t) => !t.fitid || selected.has(t.fitid)),
  );

  function reversalLabel(role: ReversalRole): string {
    return role;
  }

  function reversalTooltip(role: ReversalRole): string {
    switch (role) {
      case "estorno":
        return "Esta transação é um estorno — reverte outra. Somadas dão zero. Desmarcada por padrão pra não inflar gastos/renda.";
      case "estornada":
        return "Esta transação foi estornada (revertida pelo banco). Somadas com o estorno dão zero. Desmarcada por padrão.";
      case "reembolso":
        return "Esta transação é um reembolso — devolução de um Pix enviado. Somadas dão zero. Desmarcada por padrão.";
      case "reembolsada":
        return "Esta transação foi reembolsada. Somadas com o reembolso dão zero. Desmarcada por padrão.";
    }
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
        <th class="text-left px-3 py-2 font-medium text-fg-faint uppercase tracking-wider text-[10.5px]">Data</th>
        <th class="text-left px-3 py-2 font-medium text-fg-faint uppercase tracking-wider text-[10.5px]">Descrição</th>
        <th class="text-right px-3 py-2 font-medium text-fg-faint uppercase tracking-wider text-[10.5px]">Valor</th>
      </tr>
    </thead>
    <tbody>
      {#each transactions as t (t.fitid ?? `${t.date}-${t.amount}-${t.description}`)}
        {@const isDup = !!(t.fitid && duplicateFitids.has(t.fitid))}
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

  {#if reversalMap.size > 0}
    <div class="px-3 py-2 border-t border-border-subtle bg-surface-2 text-[11px] text-fg-muted flex items-center gap-2">
      <span class="inline-block w-2 h-2 rounded" style="background: var(--color-cat-amarelo);"></span>
      <span>
        <strong class="text-fg">{reversalMap.size / 2}</strong>
        par{reversalMap.size / 2 === 1 ? "" : "es"} estorno/reembolso ↔ original detectado{reversalMap.size / 2 === 1 ? "" : "s"} —
        desmarcado{reversalMap.size / 2 === 1 ? "" : "s"} por padrão porque a soma é zero (não é gasto nem renda real). Marque
        manualmente se quiser importar mesmo assim.
      </span>
    </div>
  {/if}
</div>
