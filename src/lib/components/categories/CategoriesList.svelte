<script lang="ts">
  import Icon from "$lib/components/ui/Icon.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import { locale } from "$lib/i18n/locale.svelte";
  import type { CategoryWithCount } from "$lib/bindings";

  const t = locale.t;

  type Props = {
    categories: CategoryWithCount[];
    onEdit: (c: CategoryWithCount) => void;
    onDelete: (c: CategoryWithCount) => Promise<void>;
    selectedId?: number | null;
  };

  let { categories, onEdit, onDelete, selectedId = null }: Props = $props();

  function kindLabel(k: string): string {
    return t("kind." + k);
  }
</script>

<div class="card overflow-hidden">
  {#if categories.length === 0}
    <EmptyState icon="tags" title={t("categories.empty")} description={t("categories.empty_desc")} />
  {:else}
    <table class="w-full">
      <thead>
        <tr class="border-b border-border-subtle">
          <th class="col-head text-left px-4 py-2">{t("categories.col_category")}</th>
          <th class="col-head text-left px-4 py-2 w-[140px]">{t("categories.col_type")}</th>
          <th class="col-head text-right px-4 py-2 w-[120px]">{t("categories.col_transactions")}</th>
          <th class="px-4 py-2 w-[60px]"><span class="sr-only">{t("common.actions")}</span></th>
        </tr>
      </thead>
      <tbody>
        {#each categories as c (c.id)}
          {@const selected = selectedId === c.id}
          <!-- The whole row opens the edit panel, same gesture as the rules and
               transactions tables. Only the actions column escapes the click,
               holding what the row does not do: delete. -->
          <tr
            class="row group border-t border-border-subtle first:border-t-0 cursor-default
                   {selected ? 'bg-accent-soft hover:bg-accent-soft' : ''}"
            aria-selected={selected}
          >
            <td class="px-4 py-2" onclick={() => onEdit(c)}>
              <span class="inline-flex items-center gap-2 text-callout">
                <span
                  class="w-2.5 h-2.5 rounded-[3px] shrink-0"
                  style="background: var({c.color_token ?? '--color-cat-outros'})"
                ></span>
                <span class="font-medium text-fg">{c.name}</span>
              </span>
            </td>
            <td class="px-4 py-2 text-sub text-fg-muted" onclick={() => onEdit(c)}>
              {kindLabel(c.kind)}
            </td>
            <td
              class="px-4 py-2 text-right text-sub tabular text-fg-muted"
              onclick={() => onEdit(c)}
            >
              {c.transaction_count}
            </td>
            <td class="px-4 py-2">
              <div
                class="flex gap-1 justify-end opacity-0 group-hover:opacity-100 focus-within:opacity-100
                       transition-opacity duration-[var(--dur-fast)]"
              >
                <button
                  type="button"
                  onclick={() => onDelete(c)}
                  title={t("categories.delete")}
                  aria-label={`${t("categories.delete")} ${c.name}`}
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
