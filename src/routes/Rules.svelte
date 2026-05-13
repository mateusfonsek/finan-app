<script lang="ts">
  import { onMount } from "svelte";
  import { Button } from "$lib/components/ui/button";
  import RuleForm from "$lib/components/rules/RuleForm.svelte";
  import RulesList from "$lib/components/rules/RulesList.svelte";
  import { listCategories } from "$lib/api/categories";
  import {
    listRules,
    createRule,
    updateRule,
    deleteRule,
    applyRulesToUncategorized,
  } from "$lib/api/rules";
  import type { Category, Rule } from "$lib/bindings";

  let rules = $state<Rule[]>([]);
  let categories = $state<Category[]>([]);
  let editing = $state<Rule | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let applyMsg = $state<string | null>(null);
  let applying = $state(false);

  async function refresh() {
    rules = await listRules();
  }

  onMount(async () => {
    try {
      [categories, rules] = await Promise.all([listCategories(), listRules()]);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  });

  async function onCreate(data: { pattern: string; categoryId: number; priority: number }) {
    await createRule({ pattern: data.pattern, category_id: data.categoryId, priority: data.priority });
    await refresh();
  }

  async function onUpdate(data: { pattern: string; categoryId: number; priority: number }) {
    if (!editing) return;
    await updateRule(editing.id, {
      pattern: data.pattern,
      category_id: data.categoryId,
      priority: data.priority,
    });
    editing = null;
    await refresh();
  }

  async function onDelete(rule: Rule) {
    const ok = confirm(`Apagar regra "${rule.pattern}"?`);
    if (!ok) return;
    await deleteRule(rule.id);
    await refresh();
  }

  async function onApply() {
    applying = true;
    applyMsg = null;
    try {
      const n = await applyRulesToUncategorized(null);
      applyMsg = n === 0
        ? "Nenhuma transação foi categorizada (nenhum pattern bateu)."
        : `${n} ${n === 1 ? "transação" : "transações"} categorizada${n === 1 ? "" : "s"} automaticamente.`;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      applying = false;
    }
  }
</script>

<section class="p-8 max-w-5xl mx-auto flex flex-col gap-5">
  <header class="flex items-baseline justify-between gap-4 flex-wrap">
    <div class="flex flex-col gap-1">
      <h2 class="text-xl font-semibold tracking-tight" style="font-family: var(--font-display)">
        Regras
      </h2>
      <p class="text-xs text-fg-faint max-w-xl">
        Regras "se descrição contém X, categoria Y" são aplicadas automaticamente em cada import.
        Categorização manual nunca é sobrescrita.
      </p>
    </div>
    <div class="flex items-center gap-2">
      {#if applyMsg}
        <span class="text-[11px] text-fg-faint">{applyMsg}</span>
      {/if}
      <Button onclick={onApply} disabled={applying || rules.length === 0}>
        {applying ? "Aplicando…" : "Aplicar a existentes"}
      </Button>
    </div>
  </header>

  {#if loading}
    <div class="text-fg-faint text-sm">Carregando…</div>
  {:else}
    {#if error}
      <div class="rounded-lg border border-border bg-surface p-3 text-sm text-neg">{error}</div>
    {/if}

    {#if editing}
      <RuleForm
        {categories}
        initial={editing}
        onSave={onUpdate}
        onCancel={() => (editing = null)}
        submitLabel="Salvar alterações"
      />
    {:else}
      <RuleForm {categories} onSave={onCreate} />
    {/if}

    <RulesList
      {rules}
      {categories}
      onEdit={(r) => (editing = r)}
      {onDelete}
    />
  {/if}
</section>
