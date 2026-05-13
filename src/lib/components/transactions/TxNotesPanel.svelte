<script lang="ts">
  import { tick } from "svelte";
  import { Button } from "$lib/components/ui/button";
  import { formatMoney } from "$lib/format/money";
  import type { Transaction } from "$lib/bindings";

  type Props = {
    transaction: Transaction;
    onClose: () => void;
    onSave: (transactionId: number, notes: string | null) => Promise<void>;
  };

  let { transaction, onClose, onSave }: Props = $props();

  let draft = $state(transaction.notes ?? "");
  let busy = $state(false);
  let textarea: HTMLTextAreaElement | undefined = $state();

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

  <div class="px-4 pb-2">
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
