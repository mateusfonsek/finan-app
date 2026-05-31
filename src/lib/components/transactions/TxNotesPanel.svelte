<script lang="ts">
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
      ruleError = "Pattern não pode ser vazio.";
      return;
    }
    if (ruleCategoryId === null) {
      ruleError = "Selecione uma categoria.";
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
  aria-label="Fechar"
  onclick={onClose}
  class="fixed inset-0 z-20 bg-black/30"
></button>

<aside
  class="fixed right-0 top-0 bottom-0 z-30 w-[360px] bg-surface border-l border-border-subtle flex flex-col"
  style="box-shadow: -12px 0 32px -8px rgba(0,0,0,.55)"
>
  <header class="flex items-center justify-between px-4 py-3 border-b border-border-subtle">
    <span class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint">Detalhe</span>
    <button type="button" onclick={onClose} class="text-fg-muted hover:text-fg" aria-label="Fechar">
      ✕
    </button>
  </header>

  <div class="p-4 flex flex-col gap-3 text-[12px]">
    <div class="flex justify-between">
      <span class="text-fg-muted">Data</span>
      <span class="tabular">{transaction.date}</span>
    </div>
    <div class="flex justify-between">
      <span class="text-fg-muted">Valor</span>
      <span class="tabular font-semibold {Number(transaction.amount) >= 0 ? 'text-pos' : 'text-fg'}">
        {formatMoney(transaction.amount)}
      </span>
    </div>
    <div class="flex flex-col gap-1">
      <span class="text-fg-muted">Descrição</span>
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
        + Criar regra desta transação
      </Button>
    {:else}
      <div class="flex flex-col gap-2 rounded-md border border-border-subtle bg-surface-2/50 p-3">
        <span class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint">Nova regra</span>
        <label class="flex flex-col gap-1">
          <span class="text-[11px] text-fg-muted">Pattern (descrição contém)</span>
          <input
            bind:value={rulePattern}
            class="rounded-md border border-border bg-surface px-2 py-1 text-[12px] text-fg font-mono focus:outline-none focus:border-accent"
            title="LIKE substring que será gravado na regra"
          />
        </label>
        <label class="flex flex-col gap-1">
          <span class="text-[11px] text-fg-muted">Categoria</span>
          <select
            value={ruleCategoryId === null ? "" : String(ruleCategoryId)}
            onchange={(e) => {
              const v = (e.currentTarget as HTMLSelectElement).value;
              ruleCategoryId = v === "" ? null : Number(v);
            }}
            class="rounded-md border border-border bg-surface px-2 py-1 text-[12px] text-fg"
          >
            <option value="">— selecione —</option>
            {#each categories as c}
              <option value={String(c.id)}>{c.name}</option>
            {/each}
          </select>
        </label>
        {#if ruleError}
          <div class="text-[11px] text-neg">{ruleError}</div>
        {/if}
        <div class="flex justify-end gap-2">
          <Button variant="ghost" onclick={() => (ruleOpen = false)} disabled={ruleBusy}>Cancelar</Button>
          <Button onclick={createRule} disabled={ruleBusy}>
            {ruleBusy ? "Criando…" : "Criar e aplicar"}
          </Button>
        </div>
      </div>
    {/if}
  </div>

  <div class="px-4 pb-2 pt-3">
    <label class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint mb-1 block" for="tx-notes">
      Notes
    </label>
    <textarea
      id="tx-notes"
      bind:this={textarea}
      bind:value={draft}
      placeholder="Anotações sobre essa transação…"
      rows="6"
      class="w-full rounded-md border border-border bg-surface-2 p-2 text-[12px] text-fg resize-none focus:outline-none focus:border-accent focus:bg-bg"
    ></textarea>
  </div>

  <footer class="mt-auto px-4 py-3 border-t border-border-subtle flex justify-end gap-2">
    <Button variant="ghost" onclick={onClose}>Cancelar</Button>
    <Button onclick={save} disabled={busy}>{busy ? "Salvando…" : "Salvar"}</Button>
  </footer>
</aside>
