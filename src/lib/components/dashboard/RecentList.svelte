<script lang="ts">
  import { formatMoney } from "$lib/format/money";
  import type { Transaction } from "$lib/bindings";

  type Props = {
    transactions: Transaction[];
  };

  let { transactions }: Props = $props();

  /**
   * OFX brasileiro entrega descrições de Pix/TED prefixadas com cabeçalho
   * tipo "Transferência enviada pelo Pix - <beneficiário> - <cnpj> - <banco>...".
   * Pra exibir mais limpo no widget de últimas transações, mostramos só o que
   * vem depois do primeiro hífen (que normalmente é o nome do beneficiário em
   * diante). Descrições sem hífen passam inalteradas.
   */
  function displayDescription(raw: string): string {
    const idx = raw.indexOf("-");
    if (idx === -1) return raw;
    const after = raw.slice(idx + 1).trim();
    return after || raw;
  }
</script>

<ul class="flex flex-col">
  {#each transactions as t (t.id)}
    <li class="grid grid-cols-[68px_1fr_100px] gap-3 items-center px-3 py-2 border-b border-border-subtle last:border-b-0">
      <span class="text-[11px] text-fg-muted tabular">{t.date}</span>
      <span class="text-[12px] text-fg truncate" title={t.description}>{displayDescription(t.description)}</span>
      <span class="text-[12px] text-right tabular font-medium {Number(t.amount) >= 0 ? 'text-pos' : 'text-fg'}">
        {formatMoney(t.amount)}
      </span>
    </li>
  {:else}
    <li class="text-fg-faint italic text-[12px] px-3 py-4">Nenhuma transação ainda.</li>
  {/each}
</ul>
