<script lang="ts">
  import type { Category, Rule } from "$lib/bindings";

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

<div class="rounded-lg border border-border-subtle bg-surface overflow-hidden">
  <table class="w-full text-[12px]">
    <thead class="bg-surface-2">
      <tr>
        <th class="text-left px-4 py-2 font-medium text-fg-faint uppercase tracking-wider text-[10.5px]">Pattern</th>
        <th class="text-left px-4 py-2 font-medium text-fg-faint uppercase tracking-wider text-[10.5px] w-[180px]">Categoria</th>
        <th class="text-right px-4 py-2 font-medium text-fg-faint uppercase tracking-wider text-[10.5px] w-[90px]">Vence</th>
        <th class="text-right px-4 py-2 font-medium text-fg-faint uppercase tracking-wider text-[10.5px] w-[90px]">Prio</th>
        <th class="px-4 py-2 w-[120px]"></th>
      </tr>
    </thead>
    <tbody>
      {#each rules as r (r.id)}
        <tr class="border-t border-border-subtle hover:bg-hover">
          <td class="px-4 py-2.5">
            {#if r.display_name}
              <div class="text-[12px] text-fg font-medium">{r.display_name}</div>
              <div class="text-[10.5px] text-fg-faint font-mono">{r.pattern}</div>
            {:else}
              <div class="font-mono text-[11.5px]">{r.pattern}</div>
            {/if}
          </td>
          <td class="px-4 py-2.5">
            <span class="inline-flex items-center gap-1.5">
              <span class="w-2 h-2 rounded-full" style="background: var({categoryToken(r.category_id)})"></span>
              {categoryName(r.category_id)}
            </span>
          </td>
          <td class="px-4 py-2.5 text-right tabular text-fg-muted">
            {r.due_day ? `dia ${r.due_day}` : "—"}
          </td>
          <td class="px-4 py-2.5 text-right tabular">{r.priority}</td>
          <td class="px-4 py-2.5 text-right flex gap-2 justify-end">
            <button
              type="button"
              onclick={() => onEdit(r)}
              class="text-[11px] text-fg-muted hover:text-fg underline-offset-2 hover:underline"
            >
              Editar
            </button>
            <button
              type="button"
              onclick={() => onDelete(r)}
              class="text-[11px] text-neg hover:underline underline-offset-2"
            >
              Apagar
            </button>
          </td>
        </tr>
      {:else}
        <tr>
          <td colspan="5" class="px-4 py-10 text-center text-fg-faint">
            Nenhuma regra ainda. Crie uma acima — ex. pattern "uber" → Transporte.
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>
