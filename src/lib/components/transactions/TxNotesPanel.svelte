<script lang="ts">
  import { tick } from "svelte";
  import Icon from "$lib/components/ui/Icon.svelte";
  import { Button } from "$lib/components/ui/button";
  import { locale } from "$lib/i18n/locale.svelte";
  import { formatMoney } from "$lib/format/money";
  import { rise, scrim, sheet } from "$lib/motion";
  import { suggestPatternFor } from "$lib/api/suggestions";
  import type { Category, Transaction } from "$lib/bindings";

  const t = locale.t;

  type Props = {
    transaction: Transaction;
    categories: Category[];
    onClose: () => void;
    onSave: (transactionId: number, notes: string | null) => Promise<void>;
    onCreateRule: (data: { pattern: string; categoryId: number }) => Promise<void>;
  };

  let { transaction, categories, onClose, onSave, onCreateRule }: Props = $props();

  let draft = $state("");
  let busy = $state(false);
  let textarea: HTMLTextAreaElement | undefined = $state();

  // Mini-form de criação de regra (expansível).
  let ruleOpen = $state(false);
  let rulePattern = $state("");
  let ruleCategoryId = $state<number | null>(null);
  let ruleBusy = $state(false);
  let ruleError = $state<string | null>(null);

  let category = $derived(categories.find((c) => c.id === transaction.category_id));

  function fmtDate(iso: string): string {
    const [y, m, d] = iso.split("-").map(Number);
    return new Date(y, m - 1, d).toLocaleDateString(locale.dateLocale, {
      day: "numeric",
      month: "long",
      year: "numeric",
    });
  }

  async function openRuleForm() {
    ruleError = null;
    ruleCategoryId = transaction.category_id ?? null;
    rulePattern = await suggestPatternFor(transaction.description);
    ruleOpen = true;
  }

  async function createRule() {
    ruleError = null;
    const pattern = rulePattern.trim();
    if (!pattern) {
      ruleError = t("rule_form.pattern_required");
      return;
    }
    if (ruleCategoryId === null) {
      ruleError = t("rule_form.category_required");
      return;
    }
    ruleBusy = true;
    try {
      await onCreateRule({ pattern, categoryId: ruleCategoryId });
      onClose();
    } catch (e) {
      ruleError = e instanceof Error ? e.message : String(e);
    } finally {
      ruleBusy = false;
    }
  }

  $effect(() => {
    draft = transaction.notes ?? "";
  });

  async function save() {
    busy = true;
    try {
      const value = draft.trim();
      await onSave(transaction.id, value === "" ? null : value);
      onClose();
    } finally {
      busy = false;
    }
  }

  async function focusTextarea() {
    await tick();
    textarea?.focus();
  }

  $effect(() => {
    void focusTextarea();
  });

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      if (ruleOpen) ruleOpen = false;
      else onClose();
    }
    // ⌘↩ salva — atalho de confirmação padrão do macOS em painéis de edição.
    if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      void save();
    }
  }
</script>

<svelte:window {onkeydown} />

<!-- Painel lateral: entra pela direita e sai pela direita, sempre. O véu é
     leve porque a tarefa é focada, não bloqueante. -->
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
  aria-label={t("tx_notes.detail")}
>
  <header
    data-tauri-drag-region
    class="flex items-center justify-between px-4 pb-3 border-b border-border-subtle"
    style="padding-top: max(12px, var(--titlebar-h))"
  >
    <span class="section-title">{t("tx_notes.detail")}</span>
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
    <!-- O valor é o assunto do painel: ele abre a leitura, o resto contextualiza. -->
    <div class="px-4 pt-4 pb-3 flex flex-col gap-1">
      <span
        class="text-display font-semibold tabular {Number(transaction.amount) >= 0
          ? 'text-pos'
          : 'text-fg'}"
      >
        {formatMoney(transaction.amount)}
      </span>
      <span class="text-sub text-fg-subtle tabular">{fmtDate(transaction.date)}</span>
    </div>

    <div class="px-4 pb-4 flex flex-col gap-3">
      <div class="flex flex-col gap-1">
        <span class="text-foot text-fg-subtle">{t("tx_table.description")}</span>
        <span class="text-callout text-fg leading-relaxed selectable">{transaction.description}</span>
      </div>

      <div class="flex items-center gap-2">
        <span class="text-foot text-fg-subtle">{t("tx_table.category")}</span>
        {#if category}
          <span class="chip text-fg">
            <span
              class="w-2 h-2 rounded-full"
              style="background: var({category.color_token ?? '--color-cat-outros'})"
            ></span>
            {category.name}
          </span>
        {:else}
          <span class="text-foot text-fg-subtle">{t("category_picker.no_category")}</span>
        {/if}
      </div>

      {#if transaction.ofx_fitid}
        <div class="flex items-baseline justify-between gap-3">
          <span class="text-foot text-fg-subtle">FITID</span>
          <span class="font-mono text-cap text-fg-subtle truncate selectable">
            {transaction.ofx_fitid}
          </span>
        </div>
      {/if}
    </div>

    <div class="hairline"></div>

    <div class="px-4 py-3">
      {#if !ruleOpen}
        <Button variant="outline" onclick={openRuleForm} class="w-full justify-center">
          <Icon name="wandSparkles" size={13} />
          {t("tx_notes.create_rule_cta")}
        </Button>
      {:else}
        <div class="card-inset p-3 flex flex-col gap-2.5" in:rise>
          <span class="section-title">{t("tx_notes.new_rule")}</span>
          <label class="flex flex-col gap-1">
            <span class="text-foot text-fg-subtle">{t("rule_form.pattern_label")}</span>
            <input
              bind:value={rulePattern}
              class="field font-mono"
              title={t("tx_notes.pattern_title")}
            />
          </label>
          <label class="flex flex-col gap-1">
            <span class="text-foot text-fg-subtle">{t("rule_form.category")}</span>
            <select
              value={ruleCategoryId === null ? "" : String(ruleCategoryId)}
              onchange={(e) => {
                const v = (e.currentTarget as HTMLSelectElement).value;
                ruleCategoryId = v === "" ? null : Number(v);
              }}
              class="field"
            >
              <option value="">{t("rule_form.select_placeholder")}</option>
              {#each categories as c}
                <option value={String(c.id)}>{c.name}</option>
              {/each}
            </select>
          </label>
          {#if ruleError}
            <div class="text-foot text-neg flex items-start gap-1.5">
              <Icon name="circleAlert" size={12} stroke={2} class="mt-px" />
              <span>{ruleError}</span>
            </div>
          {/if}
          <div class="flex justify-end gap-2">
            <Button variant="ghost" onclick={() => (ruleOpen = false)} disabled={ruleBusy}>
              {t("common.cancel")}
            </Button>
            <Button onclick={createRule} disabled={ruleBusy}>
              {ruleBusy ? t("tx_notes.creating") : t("tx_notes.create_apply")}
            </Button>
          </div>
        </div>
      {/if}
    </div>

    <div class="hairline"></div>

    <div class="px-4 pt-3 pb-4">
      <label class="section-title mb-1.5 block" for="tx-notes">
        {t("tx_notes.notes")}
      </label>
      <textarea
        id="tx-notes"
        bind:this={textarea}
        bind:value={draft}
        placeholder={t("tx_notes.notes_placeholder")}
        rows="6"
        class="field w-full resize-none leading-relaxed"
      ></textarea>
    </div>
  </div>

  <footer class="px-4 py-3 border-t border-border-subtle flex justify-end gap-2">
    <Button variant="ghost" onclick={onClose}>{t("common.cancel")}</Button>
    <Button onclick={save} disabled={busy}>
      {busy ? t("categories.saving") : t("categories.save")}
    </Button>
  </footer>
</div>
