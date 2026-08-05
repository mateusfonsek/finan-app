<script lang="ts">
  import { locale } from "$lib/i18n/locale.svelte";
  import { formatMoney } from "$lib/format/money";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import type { ExpenseRow } from "$lib/bindings";

  const tr = locale.t;

  type Props = { transactions: ExpenseRow[] };
  let { transactions }: Props = $props();

  /**
   * Brazilian OFX prefixes Pix/TED descriptions with a header like
   * "Transferencia enviada pelo Pix - <payee> - <tax id> - <bank>...". For a
   * cleaner widget, only what follows the first hyphen is shown (usually the
   * payee onward). Descriptions without a hyphen pass through unchanged.
   */
  function displayDescription(raw: string): string {
    const idx = raw.indexOf("-");
    if (idx === -1) return raw;
    const after = raw.slice(idx + 1).trim();
    return after || raw;
  }

  /** "2026-08-14" becomes "14 ago" — the full date neither fits nor is needed
   *  in a widget already scoped to one month. */
  function shortDate(iso: string): string {
    const mo = Number(iso.slice(5, 7)) - 1;
    return `${iso.slice(8, 10)} ${(locale.monthsShort[mo] ?? "").toLowerCase()}`;
  }
</script>

{#if transactions.length === 0}
  <EmptyState icon="inbox" title={tr("dashboard.empty_tx")} compact />
{:else}
  <ul class="flex flex-col">
    {#each transactions as t (t.id)}
      <li
        class="row grid grid-cols-[52px_1fr_auto] gap-3 items-center px-4 py-2
               border-b border-border-subtle last:border-b-0"
      >
        <span class="text-foot text-fg-subtle tabular">{shortDate(t.date)}</span>
        <div class="flex flex-col gap-0.5 min-w-0">
          <span class="text-sub text-fg truncate" title={t.description}>
            {displayDescription(t.description)}
          </span>
          {#if t.category_name}
            <span
              class="self-start text-cap2 font-medium rounded-full px-1.5 py-px"
              style={t.category_color_token
                ? `color: var(${t.category_color_token}); background: color-mix(in oklch, var(${t.category_color_token}) 14%, transparent);`
                : "color: var(--color-fg-faint); background: var(--color-surface-2);"}
            >
              {t.category_name}
            </span>
          {:else}
            <span class="self-start text-cap2 text-fg-subtle">
              {tr("dashboard.no_category")}
            </span>
          {/if}
        </div>
        <span
          class="text-sub text-right tabular font-medium {Number(t.amount) >= 0
            ? 'text-pos'
            : 'text-fg'}"
        >
          {formatMoney(t.amount)}
        </span>
      </li>
    {/each}
  </ul>
{/if}
