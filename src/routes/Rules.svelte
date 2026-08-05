<script lang="ts">
  import { locale } from "$lib/i18n/locale.svelte";

  const t = locale.t;
  import { onMount } from "svelte";
  import { confirm } from "@tauri-apps/plugin-dialog";
  import { Button } from "$lib/components/ui/button";
  import Page from "$lib/components/ui/Page.svelte";
  import Loading from "$lib/components/ui/Loading.svelte";
  import ErrorNote from "$lib/components/ui/ErrorNote.svelte";
  import Icon from "$lib/components/ui/Icon.svelte";
  import RuleForm from "$lib/components/rules/RuleForm.svelte";
  import RulePanel from "$lib/components/rules/RulePanel.svelte";
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

  async function onCreate(data: {
    patterns: string[];
    categoryId: number;
    priority: number;
    dueDay: number | null;
  }) {
    await createRule({
      patterns: data.patterns,
      category_id: data.categoryId,
      priority: data.priority,
      due_day: data.dueDay,
    });
    await refresh();
  }

  async function onUpdate(
    ruleId: number,
    data: {
      patterns: string[];
      categoryId: number;
      priority: number;
      dueDay: number | null;
      displayName: string | null;
    },
  ) {
    await updateRule(ruleId, {
      patterns: data.patterns,
      category_id: data.categoryId,
      priority: data.priority,
      due_day: data.dueDay,
      // O backend faz `SET display_name = ?` sempre; sem reenviar, o rótulo
      // vindo do import seria apagado a cada edição.
      display_name: data.displayName,
    });
    await refresh();
  }

  /** Alerta NATIVO do macOS — apagar uma regra descategoriza transações. */
  async function onDelete(rule: Rule) {
    const label = rule.display_name ?? rule.patterns[0] ?? "";
    const ok = await confirm(t("rules_page.delete_confirm", { pattern: label }), {
      title: t("rules.delete"),
      kind: "warning",
      okLabel: t("common.delete"),
      cancelLabel: t("common.cancel"),
    });
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
        ? t("rules_page.applied_none")
        : (n === 1
            ? t("rules_page.applied_one", { n })
            : t("rules_page.applied_many", { n }));
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      applying = false;
    }
  }
</script>

<Page title={t("nav.rules")} subtitle={t("rules_page.desc")}>
  {#snippet toolbar()}
    <Button variant="outline" onclick={onApply} disabled={applying || rules.length === 0}>
      <Icon name="rotateCw" size={12.5} class={applying ? "animate-spin" : ""} />
      {applying ? t("rules_page.applying") : t("rules_page.apply")}
    </Button>
  {/snippet}

  {#if loading}
    <Loading />
  {:else}
    {#if error}
      <ErrorNote message={error} />
    {/if}

    <!-- Resultado de "aplicar" fica junto do botão que o causou, não numa
         faixa genérica no topo. -->
    {#if applyMsg}
      <ErrorNote message={applyMsg} tone="success" />
    {/if}

    <!-- O formulário da página cria; editar acontece no painel lateral. Assim o
         "novo" nunca muda de identidade no meio do caminho. -->
    <RuleForm {categories} onSave={onCreate} />

    <RulesList
      {rules}
      {categories}
      onEdit={(r) => (editing = r)}
      {onDelete}
      selectedId={editing?.id ?? null}
    />
  {/if}
</Page>

{#if editing}
  <RulePanel
    rule={editing}
    {categories}
    onClose={() => (editing = null)}
    onSave={onUpdate}
  />
{/if}
