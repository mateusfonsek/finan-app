<script lang="ts">
  import { locale } from "$lib/i18n/locale.svelte";

  const t = locale.t;
  import { tick } from "svelte";
  import { Button } from "$lib/components/ui/button";
  import { formatMoney } from "$lib/format/money";
  import { suggestPatternFor } from "$lib/api/suggestions";
  import type { Category, Transaction } from "$lib/bindings";

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
</script>

<button
  type="button"
  aria-label={t("common.close")}
  onclick={onClose}
  class="fixed inset-0 z-20 bg-black/30"
></button>

<aside
  class="fixed right-0 top-0 bottom-0 z-30 w-[360px] bg-surface border-l border-border-subtle flex flex-col"
  style="box-shadow: -12px 0 32px -8px rgba(0,0,0,.55)"
>
  <header class="flex items-center justify-between px-4 py-3 border-b border-border-subtle">
    <span class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint">{t("tx_notes.detail")}</span>
    <button type="button" onclick={onClose} class="text-fg-muted hover:text-fg" aria-label={t("common.close")}>
      ✕
    </button>
  </header>

  <div class="p-4 flex flex-col gap-3 text-[12px]">
    <div class="flex justify-between">
      <span class="text-fg-muted">{t("tx_table.date")}</span>
      <span class="tabular">{transaction.date}</span>
    </div>
    <div class="flex justify-between">
      <span class="text-fg-muted">{t("tx_table.amount")}</span>
      <span class="tabular font-semibold {Number(transaction.amount) >= 0 ? 'text-pos' : 'text-fg'}">
        {formatMoney(transaction.amount)}
      </span>
    </div>
    <div class="flex flex-col gap-1">
      <span class="text-fg-muted">{t("tx_table.description")}</span>
      <span class="text-fg">{transaction.description}</span>
    </div>
    {#if transaction.ofx_fitid}
      <div class="flex justify-between">
        <span class="text-fg-muted">FITID</span>
        <span class="font-mono text-[11px] text-fg-faint">{transaction.ofx_fitid}</span>
      </div>
    {/if}
  </div>

  <div class="px-4 pb-3 border-b border-border-subtle">
    {#if !ruleOpen}
      <Button variant="ghost" onclick={openRuleForm} class="w-full justify-center">
        {t("tx_notes.create_rule_cta")}
      </Button>
    {:else}
      <div class="flex flex-col gap-2 rounded-md border border-border-subtle bg-surface-2/50 p-3">
        <span class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint">{t("tx_notes.new_rule")}</span>
        <label class="flex flex-col gap-1">
          <span class="text-[11px] text-fg-muted">{t("rule_form.pattern_label")}</span>
          <input
            bind:value={rulePattern}
            class="rounded-md border border-border bg-surface px-2 py-1 text-[12px] text-fg font-mono focus:outline-none focus:border-accent"
            title={t("tx_notes.pattern_title")}
          />
        </label>
        <label class="flex flex-col gap-1">
          <span class="text-[11px] text-fg-muted">{t("rule_form.category")}</span>
          <select
            value={ruleCategoryId === null ? "" : String(ruleCategoryId)}
            onchange={(e) => {
              const v = (e.currentTarget as HTMLSelectElement).value;
              ruleCategoryId = v === "" ? null : Number(v);
            }}
            class="rounded-md border border-border bg-surface px-2 py-1 text-[12px] text-fg"
          >
            <option value="">{t("rule_form.select_placeholder")}</option>
            {#each categories as c}
              <option value={String(c.id)}>{c.name}</option>
            {/each}
          </select>
        </label>
        {#if ruleError}
          <div class="text-[11px] text-neg">{ruleError}</div>
        {/if}
        <div class="flex justify-end gap-2">
          <Button variant="ghost" onclick={() => (ruleOpen = false)} disabled={ruleBusy}>{t("common.cancel")}</Button>
          <Button onclick={createRule} disabled={ruleBusy}>
            {ruleBusy ? t("tx_notes.creating") : t("tx_notes.create_apply")}
          </Button>
        </div>
      </div>
    {/if}
  </div>

  <div class="px-4 pb-2 pt-3">
    <label class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint mb-1 block" for="tx-notes">
      {t("tx_notes.notes")}
    </label>
    <textarea
      id="tx-notes"
      bind:this={textarea}
      bind:value={draft}
      placeholder={t("tx_notes.notes_placeholder")}
      rows="6"
      class="w-full rounded-md border border-border bg-surface-2 p-2 text-[12px] text-fg resize-none focus:outline-none focus:border-accent focus:bg-bg"
    ></textarea>
  </div>

  <footer class="mt-auto px-4 py-3 border-t border-border-subtle flex justify-end gap-2">
    <Button variant="ghost" onclick={onClose}>{t("common.cancel")}</Button>
    <Button onclick={save} disabled={busy}>{busy ? t("categories.saving") : t("categories.save")}</Button>
  </footer>
</aside>
