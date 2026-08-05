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
  };

  let { rules, categories, onEdit, onDelete }: Props = $props();

  function categoryName(id: number): string {
    return categories.find((c) => c.id === id)?.name ?? "?";
  }
  function categoryToken(id: number): string {
    return categories.find((c) => c.id === id)?.color_token ?? "--color-cat-outros";
  }
</script>

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
          <th class="px-4 py-2 w-[92px]"><span class="sr-only">{t("common.actions")}</span></th>
        </tr>
      </thead>
      <tbody>
        {#each rules as r (r.id)}
          <tr class="row group border-t border-border-subtle first:border-t-0">
            <td class="px-4 py-2">
              {#if r.display_name}
                <div class="text-callout text-fg font-medium">{r.display_name}</div>
                <div class="text-cap text-fg-subtle font-mono truncate">{r.pattern}</div>
              {:else}
                <div class="font-mono text-sub text-fg">{r.pattern}</div>
              {/if}
            </td>
            <td class="px-4 py-2">
              <span class="inline-flex items-center gap-1.5 text-sub text-fg">
                <span
                  class="w-2 h-2 rounded-full shrink-0"
                  style="background: var({categoryToken(r.category_id)})"
                ></span>
                {categoryName(r.category_id)}
              </span>
            </td>
            <td class="px-4 py-2 text-right text-sub tabular text-fg-muted">
              {r.due_day ? t("rules.due_day", { day: r.due_day }) : "—"}
            </td>
            <td class="px-4 py-2 text-right text-sub tabular text-fg-muted">{r.priority}</td>
            <td class="px-4 py-2">
              <div
                class="flex gap-1 justify-end opacity-0 group-hover:opacity-100 focus-within:opacity-100
                       transition-opacity duration-[var(--dur-fast)]"
              >
                <button
                  type="button"
                  onclick={() => onEdit(r)}
                  title={t("rules.edit")}
                  aria-label={`${t("rules.edit")} ${r.pattern}`}
                  class="press w-6 h-6 grid place-items-center rounded-[var(--radius-sm)] text-fg-muted
                         hover:bg-hover hover:text-fg transition-colors duration-[var(--dur-fast)]"
                >
                  <Icon name="pencil" size={12.5} />
                </button>
                <button
                  type="button"
                  onclick={() => onDelete(r)}
                  title={t("rules.delete")}
                  aria-label={`${t("rules.delete")} ${r.pattern}`}
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
