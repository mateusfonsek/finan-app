<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import type { Category, Rule } from "$lib/bindings";

  type Props = {
    categories: Category[];
    initial?: Rule | null;
    onSave: (data: { pattern: string; categoryId: number; priority: number }) => Promise<void>;
    onCancel?: () => void;
    submitLabel?: string;
  };

  let { categories, initial = null, onSave, onCancel, submitLabel }: Props = $props();

  let pattern = $state("");
  let categoryId = $state<number | null>(null);
  let priority = $state(0);
  let busy = $state(false);
  let error = $state<string | null>(null);

  $effect(() => {
    pattern = initial?.pattern ?? "";
    categoryId = initial?.category_id ?? null;
    priority = initial?.priority ?? 0;
  });

  async function submit(e: Event) {
    e.preventDefault();
    error = null;
    if (pattern.trim().length === 0) {
      error = "Pattern não pode ser vazio.";
      return;
    }
    if (categoryId === null) {
      error = "Selecione uma categoria.";
      return;
    }
    busy = true;
    try {
      await onSave({ pattern: pattern.trim(), categoryId, priority });
      if (!initial) {
        pattern = "";
        categoryId = null;
        priority = 0;
      }
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }
</script>

<form onsubmit={submit} class="rounded-lg border border-border-subtle bg-surface p-4 flex flex-col gap-3">
  <div class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint">
    {initial ? "Editar regra" : "Nova regra"}
  </div>

  <div class="grid grid-cols-[1fr_180px_100px_auto] gap-2 items-end">
    <label class="flex flex-col gap-1">
      <span class="text-[11px] text-fg-muted">Pattern (descrição contém)</span>
      <input
        bind:value={pattern}
        placeholder="ex: uber"
        class="rounded-md border border-border bg-surface-2 px-2 py-1 text-[12px] text-fg focus:outline-none focus:border-accent"
      />
    </label>

    <label class="flex flex-col gap-1">
      <span class="text-[11px] text-fg-muted">Categoria</span>
      <select
        value={categoryId === null ? "" : String(categoryId)}
        onchange={(e) => {
          const v = (e.currentTarget as HTMLSelectElement).value;
          categoryId = v === "" ? null : Number(v);
        }}
        class="rounded-md border border-border bg-surface-2 px-2 py-1 text-[12px] text-fg"
      >
        <option value="">— selecione —</option>
        {#each categories as c}
          <option value={String(c.id)}>{c.name}</option>
        {/each}
      </select>
    </label>

    <label class="flex flex-col gap-1">
      <span class="text-[11px] text-fg-muted">Prioridade</span>
      <input
        type="number"
        bind:value={priority}
        class="rounded-md border border-border bg-surface-2 px-2 py-1 text-[12px] text-fg tabular focus:outline-none focus:border-accent"
      />
    </label>

    <div class="flex gap-2">
      {#if onCancel}
        <Button variant="ghost" onclick={onCancel} type="button">Cancelar</Button>
      {/if}
      <Button type="submit" disabled={busy}>
        {busy ? "Salvando…" : (submitLabel ?? (initial ? "Salvar" : "Adicionar"))}
      </Button>
    </div>
  </div>

  {#if error}
    <div class="text-[11px] text-neg">{error}</div>
  {/if}
</form>
