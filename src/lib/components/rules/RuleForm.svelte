<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import Icon from "$lib/components/ui/Icon.svelte";
  import ErrorNote from "$lib/components/ui/ErrorNote.svelte";
  import { locale } from "$lib/i18n/locale.svelte";
  import PatternListEditor, {
    rowsFrom,
    valuesOf,
    type PatternRow,
  } from "./PatternListEditor.svelte";
  import type { Category } from "$lib/bindings";

  const t = locale.t;

  /** Só cria. Editar acontece no painel lateral (`RulePanel`), aberto pela
   *  linha da lista — um formulário que troca de identidade no meio do uso
   *  esconde de qual regra ele está falando. */
  type Props = {
    categories: Category[];
    onSave: (data: {
      patterns: string[];
      categoryId: number;
      priority: number;
      dueDay: number | null;
    }) => Promise<void>;
  };

  let { categories, onSave }: Props = $props();

  let rows = $state<PatternRow[]>(rowsFrom([]));
  let categoryId = $state<number | null>(null);
  let priority = $state(0);
  /** Svelte 5 coage <input type="number"> pra number | null; mantemos compatível. */
  let dueDayValue = $state<number | null>(null);
  let busy = $state(false);
  let error = $state<string | null>(null);

  let filled = $derived(valuesOf(rows).map((v) => v.trim()).filter((v) => v !== ""));

  async function submit(e: Event) {
    e.preventDefault();
    error = null;
    if (filled.length === 0) {
      error = t("rule_form.pattern_required");
      return;
    }
    if (categoryId === null) {
      error = t("rule_form.category_required");
      return;
    }
    let dueDay: number | null = null;
    if (dueDayValue != null) {
      if (!Number.isInteger(dueDayValue) || dueDayValue < 1 || dueDayValue > 31) {
        error = t("rule_form.due_day_invalid");
        return;
      }
      dueDay = dueDayValue;
    }
    busy = true;
    try {
      await onSave({ patterns: filled, categoryId, priority, dueDay });
      rows = rowsFrom([]);
      categoryId = null;
      priority = 0;
      dueDayValue = null;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }
</script>

<form onsubmit={submit} class="card p-4 flex flex-col gap-3.5">
  <div class="flex items-center gap-2">
    <Icon name="plus" size={13} class="text-fg-subtle" />
    <span class="section-title">{t("rule_form.form_new")}</span>
  </div>

  <!-- A lista de trechos cresce pra baixo, então ganha a própria faixa: num
       grid de uma linha só ela desalinharia os outros campos a cada trecho
       adicionado. -->
  <div class="flex flex-col gap-1">
    <span class="text-foot text-fg-subtle">{t("rule_form.patterns_label")}</span>
    <PatternListEditor {rows} onChange={(next) => (rows = next)} />
  </div>

  <div class="grid grid-cols-[1fr_84px_96px_auto] gap-2.5 items-end">
    <label class="flex flex-col gap-1 min-w-0">
      <span class="text-foot text-fg-subtle">{t("rule_form.category")}</span>
      <select
        value={categoryId === null ? "" : String(categoryId)}
        onchange={(e) => {
          const v = (e.currentTarget as HTMLSelectElement).value;
          categoryId = v === "" ? null : Number(v);
        }}
        class="field"
      >
        <option value="">{t("rule_form.select_placeholder")}</option>
        {#each categories as c}
          <option value={String(c.id)}>{c.name}</option>
        {/each}
      </select>
    </label>

    <label class="flex flex-col gap-1">
      <span class="text-foot text-fg-subtle">{t("rule_form.priority")}</span>
      <input type="number" bind:value={priority} class="field tabular" />
    </label>

    <label class="flex flex-col gap-1">
      <span class="text-foot text-fg-subtle" title={t("rule_form.due_day_title")}>
        {t("rule_form.due_day")}
      </span>
      <input
        type="number"
        min="1"
        max="31"
        placeholder="—"
        bind:value={dueDayValue}
        class="field tabular"
      />
    </label>

    <Button type="submit" disabled={busy}>
      {busy ? t("rule_form.saving") : t("rule_form.add")}
    </Button>
  </div>

  {#if error}
    <ErrorNote message={error} />
  {/if}
</form>
