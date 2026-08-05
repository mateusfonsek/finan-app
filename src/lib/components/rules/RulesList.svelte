<script lang="ts">
  import { flip } from "svelte/animate";
  import Icon from "$lib/components/ui/Icon.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import SortHeader from "$lib/components/ui/SortHeader.svelte";
  import type { IconName } from "$lib/components/ui/icons";
  import { createSort, compareText, nullsLast } from "$lib/stores/sort.svelte";
  import { DUR, SNAP, reducedMotion } from "$lib/motion";
  import { locale } from "$lib/i18n/locale.svelte";
  import type { Category, RuleWithCount } from "$lib/bindings";

  const t = locale.t;

  type Props = {
    rules: RuleWithCount[];
    categories: Category[];
    onEdit: (rule: RuleWithCount) => void;
    onDelete: (rule: RuleWithCount) => Promise<void>;
    selectedId?: number | null;
  };

  let { rules, categories, onEdit, onDelete, selectedId = null }: Props = $props();

  function categoryName(id: number): string {
    return categories.find((c) => c.id === id)?.name ?? "?";
  }
  function categoryToken(id: number): string {
    return categories.find((c) => c.id === id)?.color_token ?? "--color-cat-outros";
  }

  /** O que a primeira coluna mostra — e portanto por onde ela ordena. Ordenar
   *  por um valor invisível faria a tabela parecer embaralhada. */
  function label(r: RuleWithCount): string {
    return r.display_name ?? r.patterns[0] ?? "";
  }

  // ── Ordenação ────────────────────────────────────────────────────────────
  type SortKey = "pattern" | "category" | "due" | "priority" | "count";

  /** Primeiro clique de cada coluna no sentido mais útil dela: texto começa em
   *  A→Z, vencimento pelo dia 1, e as duas colunas numéricas começam pelo maior
   *  (a prioridade que vence e as regras que mais pegam). */
  const sort = createSort<SortKey>(
    { pattern: "asc", category: "asc", due: "asc", priority: "desc", count: "desc" },
    // Espelha o `ORDER BY priority DESC, created_at DESC` do backend: a tabela
    // abre exatamente na ordem em que os dados chegaram.
    { key: "priority", dir: "desc" },
  );

  let sorted = $derived.by(() => {
    const sign = sort.sign;
    const code = locale.code;
    return [...rules].sort((a, b) => {
      let d = 0;
      switch (sort.key) {
        case "pattern":
          d = compareText(label(a), label(b), code);
          break;
        case "category":
          d = compareText(categoryName(a.category_id), categoryName(b.category_id), code);
          break;
        case "due": {
          // Regra sem vencimento vai pro fim nos dois sentidos. Se as DUAS não
          // têm, `nullsLast` devolve 0 e a decisão cai no desempate por id lá
          // embaixo, em vez de sair daqui com uma ordem instável.
          const empty = nullsLast(a.due_day, b.due_day);
          if (empty !== null && empty !== 0) return empty;
          d = empty === 0 ? 0 : (a.due_day ?? 0) - (b.due_day ?? 0);
          break;
        }
        case "priority":
          d = a.priority - b.priority;
          break;
        case "count":
          d = a.transaction_count - b.transaction_count;
          break;
      }
      // Desempate estável por id — duas regras empatadas nunca trocam de lugar
      // sozinhas entre um render e outro.
      return d !== 0 ? sign * d : sign * (a.id - b.id);
    });
  });

  function hint(key: SortKey, col: string): string {
    return t(sort.next(key) === "asc" ? "rules.sort_asc" : "rules.sort_desc", { col });
  }

  let flipParams = $derived(
    reducedMotion() ? { duration: 0 } : { duration: DUR.base, easing: SNAP },
  );

  /**
   * Larguras das colunas.
   *
   * A tabela é `table-fixed`. Sem isso o navegador dimensiona pelo conteúdo, e
   * um trecho longo — uma linha inteira de extrato, que é exatamente o caso que
   * a tela precisa suportar — empurra a primeira coluna sem limite até estourar
   * o cartão. Com `table-fixed` estas larguras valem de verdade e o que sobra
   * vira reticência.
   *
   * Larguras FOLGADAS de propósito: com `table-fixed` esses valores valem mesmo
   * numa janela larga (toda a sobra vai pro trecho), então apertá-los pro
   * mínimo do rótulo deixa o cabeçalho reticenciado o tempo todo, não só quando
   * falta espaço. O custo de folgar é só a coluna do trecho, que tem sobra.
   *
   * Fixas somam 476px. Com o `min-w` da tabela, a coluna do trecho nunca cai
   * abaixo de ~180px, que é onde ele ainda diz alguma coisa.
   */
  const COLUMNS: Array<{
    key: SortKey;
    labelKey: string;
    align: "left" | "right";
    width: string;
    dense?: boolean;
    symbol?: IconName;
  }> = [
    { key: "pattern", labelKey: "rules.col_pattern", align: "left", width: "" },
    { key: "category", labelKey: "rules.col_category", align: "left", width: "w-[156px]" },
    { key: "count", labelKey: "rules.col_transactions", align: "right", width: "w-[124px]", dense: true },
    // "Vence" e "Prioridade" viram símbolo: são as duas colunas mais estreitas
    // e as que menos mudam de linha pra linha, então o nome por extenso custava
    // largura que a coluna do trecho aproveita melhor.
    {
      key: "due",
      labelKey: "rules.col_due",
      align: "right",
      width: "w-[76px]",
      dense: true,
      symbol: "calendar",
    },
    {
      key: "priority",
      labelKey: "rules.col_priority",
      align: "right",
      width: "w-[76px]",
      dense: true,
      symbol: "arrowUpNarrowWide",
    },
  ];
</script>

<!-- A linha mostra o primeiro trecho; os outros viram um contador. Enfileirar
     todos faria a coluna crescer sem dizer mais nada — quem quer ver abre. -->
{#snippet extra(total: number)}
  {#if total > 1}
    <span
      class="ml-1.5 text-cap2 text-fg-faint font-sans tabular"
      title={t("rules.more_patterns_title", { n: total - 1 })}
    >
      {t("rules.more_patterns", { n: total - 1 })}
    </span>
  {/if}
{/snippet}

<div class="card overflow-hidden">
  {#if rules.length === 0}
    <EmptyState icon="wandSparkles" title={t("rules.empty_title")} description={t("rules.empty")} />
  {:else}
    <!-- Se a janela ficar estreita demais pras seis colunas, quem rola é a
         tabela dentro do cartão — nunca a página, e nunca cortando conteúdo no
         `overflow-hidden` que arredonda os cantos. -->
    <div class="overflow-x-auto">
      <table class="w-full table-fixed min-w-[660px]">
        <thead>
        <tr class="border-b border-border-subtle">
          {#each COLUMNS as c (c.key)}
            <th class="{c.width} p-0" aria-sort={sort.aria(c.key)}>
              <SortHeader
                label={t(c.labelKey)}
                align={c.align}
                dense={c.dense}
                symbol={c.symbol}
                active={sort.key === c.key}
                dir={sort.dir}
                hint={hint(c.key, t(c.labelKey))}
                onclick={() => sort.toggle(c.key)}
              />
            </th>
          {/each}
          <th class="px-3 py-2 w-[44px]"><span class="sr-only">{t("common.actions")}</span></th>
        </tr>
      </thead>
      <tbody>
        {#each sorted as r (r.id)}
          {@const selected = selectedId === r.id}
          <!-- A linha inteira abre o painel de edição — mesmo gesto da tabela de
               transações. Só a coluna de ações escapa do clique. -->
          <tr
            animate:flip={flipParams}
            class="row group border-t border-border-subtle first:border-t-0 cursor-default
                   {selected ? 'bg-accent-soft hover:bg-accent-soft' : ''}"
            aria-selected={selected}
          >
            <!-- `min-w-0` é o que faz `truncate` valer dentro de uma célula de
                 tabela: sem ele a caixa cresce com o texto e nada reticencia.
                 O `title` devolve o que a reticência escondeu. -->
            <td class="px-4 py-2 min-w-0" onclick={() => onEdit(r)}>
              {#if r.display_name}
                <div class="text-callout text-fg font-medium truncate" title={r.display_name}>
                  {r.display_name}
                </div>
                <div class="text-cap text-fg-subtle font-mono truncate" title={r.patterns.join(" · ")}>
                  {r.patterns[0] ?? ""}{@render extra(r.patterns.length)}
                </div>
              {:else}
                <div class="font-mono text-sub text-fg truncate" title={r.patterns.join(" · ")}>
                  {r.patterns[0] ?? ""}{@render extra(r.patterns.length)}
                </div>
              {/if}
            </td>
            <td class="px-4 py-2 min-w-0" onclick={() => onEdit(r)}>
              <span
                class="flex items-center gap-1.5 text-sub text-fg min-w-0"
                title={categoryName(r.category_id)}
              >
                <span
                  class="w-2 h-2 rounded-full shrink-0"
                  style="background: var({categoryToken(r.category_id)})"
                ></span>
                <span class="truncate">{categoryName(r.category_id)}</span>
              </span>
            </td>
            <!-- Zero é informação, não ausência: a regra existe e não pega
                 nada. Por isso ele aparece apagado em vez de virar "—".
                 O recuo acompanha o do cabeçalho (`dense`), senão o número sai
                 do prumo do rótulo. -->
            <td
              class="px-3 py-2 text-right text-sub tabular
                     {r.transaction_count === 0 ? 'text-fg-faint' : 'text-fg-muted'}"
              onclick={() => onEdit(r)}
              title={r.transaction_count === 0 ? t("rules.reach_none") : t("rules.reach_title")}
            >
              {r.transaction_count}
            </td>
            <td
              class="px-3 py-2 text-right text-sub tabular text-fg-muted"
              onclick={() => onEdit(r)}
            >
              {r.due_day ? t("rules.due_day", { day: r.due_day }) : "—"}
            </td>
            <td
              class="px-3 py-2 text-right text-sub tabular text-fg-muted"
              onclick={() => onEdit(r)}
            >
              {r.priority}
            </td>
            <td class="px-3 py-2">
              <div
                class="flex gap-1 justify-end opacity-0 group-hover:opacity-100 focus-within:opacity-100
                       transition-opacity duration-[var(--dur-fast)]"
              >
                <button
                  type="button"
                  onclick={() => onDelete(r)}
                  title={t("rules.delete")}
                  aria-label={`${t("rules.delete")} ${label(r)}`}
                  class="press w-6 h-6 grid place-items-center rounded-[var(--radius-sm)] text-fg-muted
                         hover:bg-neg/12 hover:text-neg transition-colors duration-[var(--dur-fast)]"
                >
                  <Icon name="trash2" size={12.5} />
                </button>
              </div>
            </td>
          </tr>
        {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>
