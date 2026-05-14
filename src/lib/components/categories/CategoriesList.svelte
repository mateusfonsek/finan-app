<script lang="ts">
  import type { CategoryWithCount } from "$lib/bindings";

  type Props = {
    categories: CategoryWithCount[];
    onEdit: (c: CategoryWithCount) => void;
    onDelete: (c: CategoryWithCount) => Promise<void>;
  };

  let { categories, onEdit, onDelete }: Props = $props();

  function kindLabel(k: string): string {
    if (k === "income") return "Receita";
    if (k === "transfer") return "Transferência";
    return "Despesa";
  }
</script>

<div class="rounded-lg border border-border-subtle bg-surface overflow-hidden">
  <table class="w-full text-[12px]">
    <thead class="bg-surface-2">
      <tr>
        <th class="text-left px-4 py-2 font-medium text-fg-faint uppercase tracking-wider text-[10.5px]">Categoria</th>
        <th class="text-left px-4 py-2 font-medium text-fg-faint uppercase tracking-wider text-[10.5px] w-[140px]">Tipo</th>
        <th class="text-right px-4 py-2 font-medium text-fg-faint uppercase tracking-wider text-[10.5px] w-[120px]">Transações</th>
        <th class="px-4 py-2 w-[140px]"></th>
      </tr>
    </thead>
    <tbody>
      {#each categories as c (c.id)}
        <tr class="border-t border-border-subtle hover:bg-hover">
          <td class="px-4 py-2.5">
            <span class="inline-flex items-center gap-2">
              <span class="w-2.5 h-2.5 rounded-sm shrink-0"
                    style="background: var({c.color_token ?? '--color-cat-outros'})"></span>
              <span class="font-medium">{c.name}</span>
            </span>
          </td>
          <td class="px-4 py-2.5 text-fg-muted">{kindLabel(c.kind)}</td>
          <td class="px-4 py-2.5 text-right tabular text-fg-muted">{c.transaction_count}</td>
          <td class="px-4 py-2.5 text-right flex gap-2 justify-end">
            <button
              type="button"
              onclick={() => onEdit(c)}
              class="text-[11px] text-fg-muted hover:text-fg underline-offset-2 hover:underline"
            >
              Editar
            </button>
            <button
              type="button"
              onclick={() => onDelete(c)}
              class="text-[11px] text-neg hover:underline underline-offset-2"
            >
              Apagar
            </button>
          </td>
        </tr>
      {:else}
        <tr>
          <td colspan="4" class="px-4 py-10 text-center text-fg-faint">
            Sem categorias.
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>
