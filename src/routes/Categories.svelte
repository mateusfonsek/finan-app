<script lang="ts">
  import { onMount } from "svelte";
  import CategoryForm from "$lib/components/categories/CategoryForm.svelte";
  import CategoriesList from "$lib/components/categories/CategoriesList.svelte";
  import {
    listCategoriesWithCount,
    createCategory,
    updateCategory,
    deleteCategory,
  } from "$lib/api/categories";
  import type { CategoryWithCount } from "$lib/bindings";

  let categories = $state<CategoryWithCount[]>([]);
  let editing = $state<CategoryWithCount | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);

  async function refresh() {
    categories = await listCategoriesWithCount();
  }

  onMount(async () => {
    try {
      await refresh();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  });

  async function onCreate(data: { name: string; colorToken: string; kind: string }) {
    await createCategory({ name: data.name, color_token: data.colorToken, kind: data.kind });
    await refresh();
  }

  async function onUpdate(data: { name: string; colorToken: string; kind: string }) {
    if (!editing) return;
    await updateCategory(editing.id, {
      name: data.name,
      color_token: data.colorToken,
      kind: data.kind,
    });
    editing = null;
    await refresh();
  }

  async function onDelete(c: CategoryWithCount) {
    const msg = c.transaction_count > 0
      ? `Apagar "${c.name}"? ${c.transaction_count} ${c.transaction_count === 1 ? "transação ficará" : "transações ficarão"} sem categoria.`
      : `Apagar "${c.name}"?`;
    if (!confirm(msg)) return;
    try {
      await deleteCategory(c.id);
      await refresh();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }
</script>

<section class="p-8 max-w-4xl mx-auto flex flex-col gap-5">
  <header>
    <h2 class="text-xl font-semibold tracking-tight" style="font-family: var(--font-display)">
      Categorias
    </h2>
    <p class="text-xs text-fg-faint max-w-xl mt-1">
      Crie, edite ou apague suas categorias. As 9 padrão (marcadas como "padrão")
      podem ser editadas mas não apagadas — pra preservar o estado inicial do app.
    </p>
  </header>

  {#if loading}
    <div class="text-fg-faint text-sm">Carregando…</div>
  {:else}
    {#if error}
      <div class="rounded-lg border border-border bg-surface p-3 text-sm text-neg">{error}</div>
    {/if}

    {#if editing}
      <CategoryForm
        initial={editing}
        onSave={onUpdate}
        onCancel={() => (editing = null)}
      />
    {:else}
      <CategoryForm onSave={onCreate} />
    {/if}

    <CategoriesList
      {categories}
      onEdit={(c) => (editing = c)}
      {onDelete}
    />
  {/if}
</section>
