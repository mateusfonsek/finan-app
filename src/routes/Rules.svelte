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
  import RuleApplyDialog from "$lib/components/rules/RuleApplyDialog.svelte";
  import RuleMatchesDialog from "$lib/components/rules/RuleMatchesDialog.svelte";
  import RuleForm from "$lib/components/rules/RuleForm.svelte";
  import RulePanel from "$lib/components/rules/RulePanel.svelte";
  import RulesList from "$lib/components/rules/RulesList.svelte";
  import { listCategories } from "$lib/api/categories";
  import {
    listRulesWithCount,
    createRule,
    updateRule,
    deleteRule,
    previewRuleApplication,
    applyRuleChoices,
  } from "$lib/api/rules";
  import type { Category, RuleChoice, RulePreviewRow, RuleWithCount } from "$lib/bindings";

  let rules = $state<RuleWithCount[]>([]);
  let categories = $state<Category[]>([]);
  let editing = $state<RuleWithCount | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let applyMsg = $state<string | null>(null);
  let applying = $state(false);
  let previewRows = $state<RulePreviewRow[]>([]);
  let previewOpen = $state(false);
  /** Regra cuja lista de transações está aberta. O painel de edição continua
   *  montado atrás — fechar aqui devolve você pra regra que estava editando. */
  let matchesFor = $state<RuleWithCount | null>(null);

  async function refresh() {
    rules = await listRulesWithCount();
  }

  onMount(async () => {
    try {
      [categories, rules] = await Promise.all([listCategories(), listRulesWithCount()]);
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
  async function onDelete(rule: RuleWithCount) {
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

  /**
   * "Revisar e aplicar" não grava nada: ele consulta o que MUDARIA e abre a
   * revisão. Quem grava é a escolha do usuário lá dentro.
   *
   * Quando não há nada a mudar, não abre janela nenhuma — um modal só pra dizer
   * "nada a fazer" é uma cerimônia que cobra um clique de volta sem entregar
   * nada. A resposta vai na mesma faixa de sempre, junto do botão que a causou.
   */
  async function onApply() {
    applying = true;
    applyMsg = null;
    error = null;
    try {
      const rows = await previewRuleApplication(null);
      if (rows.length === 0) {
        applyMsg = t("rule_apply.nothing_to_do");
        return;
      }
      previewRows = rows;
      previewOpen = true;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      applying = false;
    }
  }

  async function onConfirmApply(choices: RuleChoice[]) {
    const n = await applyRuleChoices(choices);
    previewOpen = false;
    previewRows = [];
    applyMsg =
      n === 1 ? t("rule_apply.applied_one") : t("rule_apply.applied_many", { n });
    // As contagens de alcance da tabela não mudam com isso (alcance independe
    // de categoria), mas a lista é barata e assim nada fica velho na tela.
    await refresh();
  }
</script>

<!-- `wide`: seis colunas não cabem na coluna padrão sem espremer o trecho da
     descrição, que é a informação principal da tela. -->
<Page title={t("nav.rules")} subtitle={t("rules_page.desc")} width="wide">
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

{#if previewOpen}
  <RuleApplyDialog
    rows={previewRows}
    {categories}
    onClose={() => (previewOpen = false)}
    onApply={onConfirmApply}
  />
{/if}

{#if editing}
  <RulePanel
    rule={editing}
    {categories}
    onClose={() => (editing = null)}
    onSave={onUpdate}
    onViewMatches={() => (matchesFor = editing)}
    blocked={matchesFor !== null}
  />
{/if}

{#if matchesFor}
  <RuleMatchesDialog
    rule={matchesFor}
    {categories}
    onClose={() => (matchesFor = null)}
  />
{/if}
