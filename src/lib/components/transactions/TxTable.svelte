<script lang="ts">
  import { push } from "svelte-spa-router";
  import { flip } from "svelte/animate";
  import { formatMoney } from "$lib/format/money";
  import { locale } from "$lib/i18n/locale.svelte";
  import { Button } from "$lib/components/ui/button";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import SortHeader from "$lib/components/ui/SortHeader.svelte";
  import { createSort } from "$lib/stores/sort.svelte";
  import { DUR, SNAP, reducedMotion } from "$lib/motion";
  import CategoryPicker from "./CategoryPicker.svelte";
  import type { Category, Transaction } from "$lib/bindings";

  const t = locale.t;

  type Props = {
    transactions: Transaction[];
    categories: Category[];
    onCategoryChange: (transactionId: number, categoryId: number | null) => Promise<void>;
    onCategoryCreate: (name: string) => Promise<Category>;
    onRowClick?: (transaction: Transaction) => void;
    selectedId?: number | null;
  };

  let {
    transactions,
    categories,
    onCategoryChange,
    onCategoryCreate,
    onRowClick,
    selectedId,
  }: Props = $props();

  /** "2026-08-14" → "14 ago 2026". Data por extenso curta lê mais rápido que
   *  ISO e ocupa quase o mesmo espaço. */
  function fmtDate(iso: string): string {
    const mo = Number(iso.slice(5, 7)) - 1;
    return `${iso.slice(8, 10)} ${(locale.monthsShort[mo] ?? "").toLowerCase()} ${iso.slice(0, 4)}`;
  }

  // ── Ordenação ────────────────────────────────────────────────────────────
  // Reordena no cliente: a lista já está toda em memória, então o clique
  // responde no mesmo quadro — nenhuma ida ao backend entre o dedo e o
  // resultado.
  type SortKey = "date" | "amount";

  /** Primeiro clique de cada coluna cai no sentido mais útil dela: data começa
   *  pela mais recente; valor começa pela maior saída (gastos são negativos,
   *  logo o mais negativo vem primeiro em ordem crescente). */
  const sort = createSort<SortKey>({ date: "desc", amount: "asc" }, { key: "date", dir: "desc" });

  let sorted = $derived.by(() => {
    const sign = sort.sign;
    // Cópia: `transactions` é prop, ordenar no lugar mutaria o pai.
    return [...transactions].sort((a, b) => {
      let d = 0;
      if (sort.key === "date") {
        d = a.date < b.date ? -1 : a.date > b.date ? 1 : 0;
      } else {
        d = Number(a.amount) - Number(b.amount);
      }
      // Desempate estável por id, no mesmo sentido — duas transações do mesmo
      // dia (ou do mesmo valor) nunca trocam de lugar sozinhas entre renders.
      return d !== 0 ? sign * d : sign * (a.id - b.id);
    });
  });

  /** Rótulo do estado que o clique VAI produzir — o title antecipa o resultado
   *  em vez de descrever o atual. */
  function sortHint(key: SortKey, label: string): string {
    const next = sort.next(key);
    if (key === "date") {
      return t(next === "asc" ? "tx_table.sort_date_asc" : "tx_table.sort_date_desc");
    }
    return t(next === "asc" ? "tx_table.sort_asc" : "tx_table.sort_desc", { col: label });
  }

  let flipParams = $derived(
    reducedMotion() ? { duration: 0 } : { duration: DUR.base, easing: SNAP },
  );
</script>

<div class="card overflow-hidden">
  {#if transactions.length === 0}
    <EmptyState icon="inbox" title={t("tx_table.empty")} description={t("tx_table.empty_desc")}>
      {#snippet action()}
        <Button variant="outline" onclick={() => push("/import")}>
          {t("tx_table.import_link")}
        </Button>
      {/snippet}
    </EmptyState>
  {:else}
    <!-- Mesmo tratamento da tabela de Regras: `table-fixed` pra uma descrição
         longa reticenciar em vez de empurrar a coluna, e a rolagem horizontal
         contida no cartão em vez de vazar pela página. -->
    <div class="overflow-x-auto">
      <table class="w-full table-fixed min-w-[560px]">
        <thead>
        <tr class="border-b border-border-subtle">
          <th class="w-[124px] p-0" aria-sort={sort.aria("date")}>
            <SortHeader
              label={t("tx_table.date")}
              active={sort.key === "date"}
              dir={sort.dir}
              hint={sortHint("date", t("tx_table.date"))}
              onclick={() => sort.toggle("date")}
            />
          </th>
          <th class="col-head text-left px-4 py-2">{t("tx_table.description")}</th>
          <th class="col-head text-left px-4 py-2 w-[184px]">{t("tx_table.category")}</th>
          <th class="w-[136px] p-0" aria-sort={sort.aria("amount")}>
            <SortHeader
              label={t("tx_table.amount")}
              align="right"
              active={sort.key === "amount"}
              dir={sort.dir}
              hint={sortHint("amount", t("tx_table.amount"))}
              onclick={() => sort.toggle("amount")}
            />
          </th>
        </tr>
      </thead>
      <tbody>
        {#each sorted as t (t.id)}
          {@const selected = selectedId === t.id}
          <!-- A linha inteira é o alvo do clique (menos a célula de categoria,
               que tem controle próprio) — alvo grande, como em listas do macOS. -->
          <tr
            animate:flip={flipParams}
            class="row border-t border-border-subtle first:border-t-0 cursor-default
                   {selected ? 'bg-accent-soft hover:bg-accent-soft' : ''}"
            aria-selected={selected}
          >
            <td class="px-4 py-2 text-sub text-fg-subtle tabular whitespace-nowrap" onclick={() => onRowClick?.(t)}>
              {fmtDate(t.date)}
            </td>
            <td class="px-4 py-2 text-callout text-fg min-w-0" onclick={() => onRowClick?.(t)}>
              <!-- Duas linhas e para: descrição de extrato pode ter 200
                   caracteres, e uma linha de tabela alta demais desalinha a
                   leitura das vizinhas. O `title` devolve o resto. -->
              <span class="line-clamp-2 break-words" title={t.description}>{t.description}</span>
            </td>
            <td class="px-4 py-2">
              <CategoryPicker
                {categories}
                currentId={t.category_id}
                onselect={(catId) => onCategoryChange(t.id, catId)}
                oncreate={onCategoryCreate}
              />
            </td>
            <td
              class="px-4 py-2 text-right text-callout tabular font-medium whitespace-nowrap
                     {Number(t.amount) >= 0 ? 'text-pos' : 'text-fg'}"
              onclick={() => onRowClick?.(t)}
            >
              {formatMoney(t.amount)}
            </td>
          </tr>
        {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>
