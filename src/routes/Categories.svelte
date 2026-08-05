<script lang="ts">
  import { locale } from "$lib/i18n/locale.svelte";

  const t = locale.t;
  import { onMount } from "svelte";
  import { confirm } from "@tauri-apps/plugin-dialog";
  import Page from "$lib/components/ui/Page.svelte";
  import Loading from "$lib/components/ui/Loading.svelte";
  import ErrorNote from "$lib/components/ui/ErrorNote.svelte";
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

  /** Apagar é irreversível: pede confirmação num alerta NATIVO do macOS (o
   *  `confirm()` do webview parece web, e num app de desktop isso destoa). */
  async function onDelete(c: CategoryWithCount) {
    const msg = c.transaction_count > 0
      ? (c.transaction_count === 1
          ? t("categories_page.delete_confirm_one", { name: c.name, n: c.transaction_count })
          : t("categories_page.delete_confirm_many", { name: c.name, n: c.transaction_count }))
      : t("categories_page.delete_confirm", { name: c.name });
    const ok = await confirm(msg, {
      title: t("categories.delete"),
      kind: "warning",
      okLabel: t("common.delete"),
      cancelLabel: t("common.cancel"),
    });
    if (!ok) return;
    try {
      await deleteCategory(c.id);
      await refresh();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }
</script>

<Page title={t("nav.categories")} subtitle={t("categories_page.desc")}>
  {#if loading}
    <Loading />
  {:else}
    {#if error}
      <ErrorNote message={error} />
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
</Page>
