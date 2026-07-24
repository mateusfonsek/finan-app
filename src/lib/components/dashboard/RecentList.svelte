<script lang="ts">
  import { locale } from "$lib/i18n/locale.svelte";

  const tr = locale.t;
  import { formatMoney } from "$lib/format/money";
  import type { ExpenseRow } from "$lib/bindings";

  type Props = {
    transactions: ExpenseRow[];
  };

  let { transactions }: Props = $props();

  /**
   * OFX brasileiro entrega descrições de Pix/TED prefixadas com cabeçalho
   * tipo "Transferência enviada pelo Pix - <beneficiário> - <cnpj> - <banco>...".
   * Pra exibir mais limpo no widget, mostramos só o que vem depois do primeiro
   * hífen (que normalmente é o nome do beneficiário em diante). Descrições sem
   * hífen passam inalteradas.
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
      <div class="flex flex-col gap-0.5 min-w-0">
        <span class="text-[12px] text-fg truncate" title={t.description}>{displayDescription(t.description)}</span>
        {#if t.category_name}
          <span
            class="self-start text-[9.5px] uppercase tracking-wider font-semibold rounded-full px-1.5 py-px border"
            style={t.category_color_token
              ? `color: var(${t.category_color_token}); border-color: color-mix(in oklch, var(${t.category_color_token}) 45%, transparent); background: color-mix(in oklch, var(${t.category_color_token}) 12%, transparent);`
              : "color: var(--color-fg-faint); border-color: var(--color-border-subtle); background: var(--color-surface-2);"}
          >
            {t.category_name}
          </span>
        {:else}
          <span class="self-start text-[9.5px] uppercase tracking-wider font-semibold text-fg-faint italic">
            {tr("dashboard.no_category")}
          </span>
        {/if}
      </div>
      <span class="text-[12px] text-right tabular font-medium {Number(t.amount) >= 0 ? 'text-pos' : 'text-fg'}">
        {formatMoney(t.amount)}
      </span>
    </li>
  {:else}
    <li class="text-fg-faint italic text-[12px] px-3 py-4">{tr("dashboard.empty_tx")}</li>
  {/each}
</ul>
