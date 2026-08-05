<script lang="ts">
  import Icon from "$lib/components/ui/Icon.svelte";
  import { Button } from "$lib/components/ui/button";
  import ErrorNote from "$lib/components/ui/ErrorNote.svelte";
  import { locale } from "$lib/i18n/locale.svelte";
  import { scrim, sheet } from "$lib/motion";
  import PatternListEditor, {
    rowsFrom,
    valuesOf,
    type PatternRow,
  } from "./PatternListEditor.svelte";
  import type { Category, Rule } from "$lib/bindings";

  const t = locale.t;

  type Props = {
    rule: Rule;
    categories: Category[];
    onClose: () => void;
    onSave: (
      ruleId: number,
      data: {
        patterns: string[];
        categoryId: number;
        priority: number;
        dueDay: number | null;
        displayName: string | null;
      },
    ) => Promise<void>;
  };

  let { rule, categories, onClose, onSave }: Props = $props();

  let rows = $state<PatternRow[]>([]);
  let categoryId = $state<number | null>(null);
  let priority = $state(0);
  /** Svelte 5 coage <input type="number"> pra number | null. */
  let dueDayValue = $state<number | null>(null);
  let busy = $state(false);
  let error = $state<string | null>(null);

  // Recarrega o rascunho quando o painel troca de regra (clicar noutra linha
  // com o painel aberto).
  $effect(() => {
    rows = rowsFrom(rule.patterns);
    categoryId = rule.category_id;
    priority = rule.priority;
    dueDayValue = rule.due_day;
    error = null;
  });

  let category = $derived(categories.find((c) => c.id === categoryId));
  /** Só os trechos que valem alguma coisa — é o que o resumo e o save usam. */
  let filled = $derived(valuesOf(rows).map((v) => v.trim()).filter((v) => v !== ""));

  async function save() {
    error = null;
    if (filled.length === 0) {
      error = t("rule_form.pattern_required");
      return;
    }
    if (categoryId === null) {
      error = t("rule_form.category_required");
      return;
    }
    if (dueDayValue != null) {
      if (!Number.isInteger(dueDayValue) || dueDayValue < 1 || dueDayValue > 31) {
        error = t("rule_form.due_day_invalid");
        return;
      }
    }
    busy = true;
    try {
      await onSave(rule.id, {
        patterns: filled,
        categoryId,
        priority,
        dueDay: dueDayValue,
        // Preservado: o rótulo vem do import (razão social do CNPJ) e não é
        // editável aqui — omitir apagaria ele no backend.
        displayName: rule.display_name,
      });
      onClose();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      onClose();
    }
    // ⌘↩ salva — mesmo atalho do painel de transação.
    if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      void save();
    }
  }
</script>

<svelte:window {onkeydown} />

<!-- Mesmo par véu + folha do painel de transação: entra pela direita, sai pela
     direita. Véu leve porque a tarefa é focada, não bloqueante. -->
<button
  type="button"
  aria-label={t("common.close")}
  onclick={onClose}
  transition:scrim
  class="fixed inset-0 z-50 bg-black/25"
></button>

<div
  transition:sheet={{ side: "right" }}
  class="fixed right-0 top-0 bottom-0 z-60 w-[372px] bg-surface border-l border-border-subtle
         flex flex-col shadow-[var(--shadow-sheet)]"
  role="dialog"
  aria-label={t("rule_panel.title")}
>
  <header
    data-tauri-drag-region="deep"
    class="flex items-center justify-between px-4 pb-3 border-b border-border-subtle"
    style="padding-top: max(12px, var(--titlebar-h))"
  >
    <span class="section-title">{t("rule_panel.title")}</span>
    <button
      type="button"
      onclick={onClose}
      aria-label={t("common.close")}
      class="press w-6 h-6 grid place-items-center rounded-full text-fg-faint
             hover:text-fg hover:bg-hover transition-colors duration-[var(--dur-fast)]"
    >
      <Icon name="x" size={13} stroke={2} />
    </button>
  </header>

  <div class="flex-1 overflow-y-auto">
    <!-- O assunto do painel é a frase que a regra representa. Ela acompanha o
         que está sendo digitado, então dá pra ler o efeito antes de salvar. -->
    <div class="px-4 pt-4 pb-3 flex flex-col gap-1.5">
      {#if rule.display_name}
        <span class="text-callout font-semibold text-fg">{rule.display_name}</span>
      {/if}
      <span class="text-foot text-fg-subtle">
        {filled.length > 1 ? t("rule_panel.if_contains_any") : t("rule_panel.if_contains")}
      </span>
      {#if filled.length === 0}
        <span class="text-title2 font-semibold font-mono text-fg-faint">—</span>
      {:else if filled.length === 1}
        <span class="text-title2 font-semibold font-mono text-fg break-words">{filled[0]}</span>
      {:else}
        <!-- Vários trechos: lista com marcador "ou", porque a relação entre
             eles é alternativa, não sequência. -->
        <ul class="flex flex-col gap-1 mt-0.5">
          {#each filled as p, i (i)}
            <li class="flex gap-2">
              <span class="text-foot text-fg-faint w-5 shrink-0 pt-px text-right">
                {i === 0 ? "" : t("rule_panel.or")}
              </span>
              <span class="text-callout font-medium font-mono text-fg break-all">{p}</span>
            </li>
          {/each}
        </ul>
      {/if}
      <span class="text-foot text-fg-subtle mt-0.5">{t("rule_panel.then_use")}</span>
      {#if category}
        <span class="chip text-fg self-start">
          <span
            class="w-2 h-2 rounded-full"
            style="background: var({category.color_token ?? '--color-cat-outros'})"
          ></span>
          {category.name}
        </span>
      {:else}
        <span class="text-foot text-fg-faint self-start">{t("rule_panel.no_category")}</span>
      {/if}
    </div>

    <div class="hairline"></div>

    <div class="px-4 py-4 flex flex-col gap-3.5">
      <div class="flex flex-col gap-1">
        <span class="text-foot text-fg-subtle">{t("rule_form.patterns_label")}</span>
        <PatternListEditor {rows} onChange={(next) => (rows = next)} autofocus />
        <!-- A dica fica visível: antes ela só existia como tooltip, invisível
             pra quem não passa o mouse por cima. -->
        <span class="text-cap text-fg-faint leading-snug mt-0.5">
          {t("rule_panel.patterns_hint")}
        </span>
      </div>

      <label class="flex flex-col gap-1">
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
        <span class="text-foot text-fg-subtle">{t("rule_form.due_day")}</span>
        <input
          type="number"
          min="1"
          max="31"
          placeholder="—"
          bind:value={dueDayValue}
          class="field tabular w-24"
        />
        <span class="text-cap text-fg-faint leading-snug">{t("rule_panel.due_day_hint")}</span>
      </label>

      <label class="flex flex-col gap-1">
        <span class="text-foot text-fg-subtle">{t("rule_form.priority")}</span>
        <input type="number" bind:value={priority} class="field tabular w-24" />
        <span class="text-cap text-fg-faint leading-snug">{t("rule_panel.priority_hint")}</span>
      </label>

      {#if error}
        <ErrorNote message={error} />
      {/if}
    </div>

    <div class="hairline"></div>

    <p class="px-4 py-3 text-cap text-fg-faint leading-snug">
      {t("rule_panel.scope_note")}
    </p>
  </div>

  <footer class="px-4 py-3 border-t border-border-subtle flex justify-end gap-2">
    <Button variant="ghost" onclick={onClose} disabled={busy}>{t("common.cancel")}</Button>
    <Button onclick={save} disabled={busy}>
      {busy ? t("rule_form.saving") : t("rules_page.save_changes")}
    </Button>
  </footer>
</div>
