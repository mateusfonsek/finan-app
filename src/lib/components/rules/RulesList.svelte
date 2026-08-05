<script lang="ts">
  import Icon from "$lib/components/ui/Icon.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import { locale } from "$lib/i18n/locale.svelte";
  import type { Category, Rule } from "$lib/bindings";

  const t = locale.t;

  type Props = {
    rules: Rule[];
    categories: Category[];
    onEdit: (rule: Rule) => void;
    onDelete: (rule: Rule) => Promise<void>;
    selectedId?: number | null;
  };

  let { rules, categories, onEdit, onDelete, selectedId = null }: Props = $props();

  function categoryName(id: number): string {
    return categories.find((c) => c.id === id)?.name ?? "?";
  }
  function categoryToken(id: number): string {
    return categories.find((c) => c.id === id)?.color_token ?? "--color-cat-outros";
  }
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
    <table class="w-full">
      <thead>
        <tr class="border-b border-border-subtle">
          <th class="col-head text-left px-4 py-2">{t("rules.col_pattern")}</th>
          <th class="col-head text-left px-4 py-2 w-[172px]">{t("rules.col_category")}</th>
          <th class="col-head text-right px-4 py-2 w-[88px]">{t("rules.col_due")}</th>
          <th class="col-head text-right px-4 py-2 w-[86px]">{t("rules.col_priority")}</th>
          <th class="px-4 py-2 w-[60px]"><span class="sr-only">{t("common.actions")}</span></th>
        </tr>
      </thead>
      <tbody>
        {#each rules as r (r.id)}
          {@const selected = selectedId === r.id}
          <!-- A linha inteira abre o painel de edição — mesmo gesto da tabela de
               transações. Só a coluna de ações escapa do clique. -->
          <tr
            class="row group border-t border-border-subtle first:border-t-0 cursor-default
                   {selected ? 'bg-accent-soft hover:bg-accent-soft' : ''}"
            aria-selected={selected}
          >
            <td class="px-4 py-2" onclick={() => onEdit(r)}>
              {#if r.display_name}
                <div class="text-callout text-fg font-medium">{r.display_name}</div>
                <div class="text-cap text-fg-subtle font-mono truncate">
                  {r.patterns[0] ?? ""}{@render extra(r.patterns.length)}
                </div>
              {:else}
                <div class="font-mono text-sub text-fg truncate">
                  {r.patterns[0] ?? ""}{@render extra(r.patterns.length)}
                </div>
              {/if}
            </td>
            <td class="px-4 py-2" onclick={() => onEdit(r)}>
              <span class="inline-flex items-center gap-1.5 text-sub text-fg">
                <span
                  class="w-2 h-2 rounded-full shrink-0"
                  style="background: var({categoryToken(r.category_id)})"
                ></span>
                {categoryName(r.category_id)}
              </span>
            </td>
            <td
              class="px-4 py-2 text-right text-sub tabular text-fg-muted"
              onclick={() => onEdit(r)}
            >
              {r.due_day ? t("rules.due_day", { day: r.due_day }) : "—"}
            </td>
            <td
              class="px-4 py-2 text-right text-sub tabular text-fg-muted"
              onclick={() => onEdit(r)}
            >
              {r.priority}
            </td>
            <td class="px-4 py-2">
              <div
                class="flex gap-1 justify-end opacity-0 group-hover:opacity-100 focus-within:opacity-100
                       transition-opacity duration-[var(--dur-fast)]"
              >
                <button
                  type="button"
                  onclick={() => onDelete(r)}
                  title={t("rules.delete")}
                  aria-label={`${t("rules.delete")} ${r.display_name ?? r.patterns[0] ?? ""}`}
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
  {/if}
</div>
