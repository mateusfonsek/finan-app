<script lang="ts">
  import { onMount } from "svelte";
  import { push } from "svelte-spa-router";
  import { Button } from "$lib/components/ui/button";
  import DropZone from "$lib/components/import/DropZone.svelte";
  import ImportPreview from "$lib/components/import/ImportPreview.svelte";
  import { formatMoney } from "$lib/format/money";
  import { createOrGetAccount } from "$lib/api/accounts";
  import { checkExistingFitids, insertTransactions } from "$lib/api/transactions";
  import type { ParsedOfx } from "$lib/ofx/types";
  import type { Account, NewTransaction } from "$lib/bindings";

  type PendingImport = { file: File; parsed: ParsedOfx };

  let pending = $state<PendingImport | null>(null);
  let account = $state<Account | null>(null);
  let duplicateFitids = $state<Set<string>>(new Set());
  let selected = $state<Set<string>>(new Set());
  let busy = $state(false);
  let error = $state<string | null>(null);

  onMount(() => {
    const stash = (window as unknown as { __finanPending?: PendingImport }).__finanPending;
    if (stash) {
      pending = stash;
      (window as unknown as { __finanPending?: PendingImport }).__finanPending = undefined;
      void prepareImport(stash.parsed);
    }
  });

  async function prepareImport(parsed: ParsedOfx) {
    busy = true;
    try {
      account = await createOrGetAccount({
        name: parsed.account.displayName,
        bank: parsed.account.bank === "unknown" ? null : parsed.account.bank,
        ofx_acctid: parsed.account.ofxAcctid,
      });
      const fitids = parsed.transactions
        .map((t) => t.fitid)
        .filter((f): f is string => !!f);
      const existing = await checkExistingFitids(account.id, fitids);
      duplicateFitids = new Set(existing);
      selected = new Set(fitids.filter((f) => !duplicateFitids.has(f)));
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }

  function onparsed(detail: PendingImport) {
    pending = detail;
    void prepareImport(detail.parsed);
  }
  function onerror(msg: string) {
    error = msg;
  }

  function toggle(fitid: string) {
    if (selected.has(fitid)) selected.delete(fitid);
    else selected.add(fitid);
    selected = new Set(selected);
  }

  function toggleAll(checked: boolean) {
    if (!pending) return;
    selected = checked
      ? new Set(pending.parsed.transactions.map((t) => t.fitid).filter((f): f is string => !!f))
      : new Set();
  }

  async function confirmImport() {
    if (!pending || !account) return;
    busy = true;
    error = null;
    try {
      const toInsert: NewTransaction[] = pending.parsed.transactions
        .filter((t) => t.fitid && selected.has(t.fitid))
        .map((t) => ({
          date: t.date,
          amount: t.amount,
          description: t.description,
          ofx_fitid: t.fitid,
        }));
      const result = await insertTransactions(account.id, toInsert);
      console.log("[finan] import:", result);
      push("/transactions");
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }

  function reset() {
    pending = null;
    account = null;
    selected = new Set();
    duplicateFitids = new Set();
  }
</script>

<section class="p-8 max-w-5xl mx-auto flex flex-col gap-5">
  <header class="flex items-baseline justify-between">
    <h2 class="text-xl font-semibold tracking-tight" style="font-family: var(--font-display)">
      Importar OFX
    </h2>
    {#if pending}
      <span class="text-xs text-fg-faint tabular">{pending.file.name}</span>
    {/if}
  </header>

  {#if !pending}
    <DropZone {onparsed} {onerror} />
  {:else}
    {@const p = pending.parsed}
    <div class="grid grid-cols-[1fr_280px] gap-4 items-start">
      <div class="flex flex-col gap-3">
        <div class="rounded-lg border border-border-subtle bg-surface p-4 flex items-center gap-3">
          <div class="text-[10.5px] uppercase tracking-wider font-semibold text-accent-hi bg-accent-soft border border-accent/30 rounded-full px-2 py-0.5">
            {p.account.bank === "unknown" ? "Desconhecido" : p.account.bank}
          </div>
          <div class="text-sm font-medium">{p.account.displayName}</div>
          <div class="ml-auto text-xs text-fg-faint tabular">
            {p.transactions.length} transações ·
            {p.summary.earliest ?? "?"} → {p.summary.latest ?? "?"}
          </div>
        </div>

        <ImportPreview
          transactions={p.transactions}
          {duplicateFitids}
          {selected}
          ontoggle={toggle}
          ontoggleAll={toggleAll}
        />
      </div>

      <aside class="rounded-lg border border-border-subtle bg-surface p-4 flex flex-col gap-2 text-[12px]">
        <div class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint mb-1">Resumo</div>
        <div class="flex justify-between"><span class="text-fg-muted">Entradas</span><span class="tabular text-pos">{formatMoney(p.summary.totalIn)}</span></div>
        <div class="flex justify-between"><span class="text-fg-muted">Saídas</span><span class="tabular">{formatMoney(p.summary.totalOut)}</span></div>
        <div class="flex justify-between border-t border-border-subtle pt-2 mt-1"><span class="text-fg-muted">Líquido</span><span class="tabular font-semibold">{formatMoney(p.summary.net)}</span></div>
        <div class="flex justify-between mt-2"><span class="text-fg-muted">Selecionadas</span><span class="tabular">{selected.size}</span></div>
        <div class="flex justify-between"><span class="text-fg-muted">Duplicadas</span><span class="tabular text-fg-faint">{duplicateFitids.size}</span></div>
      </aside>
    </div>

    {#if error}
      <div class="rounded-lg border border-border bg-surface p-3 text-sm text-neg">{error}</div>
    {/if}

    <div class="flex justify-end gap-2 sticky bottom-0 bg-bg pt-3 border-t border-border-subtle">
      <Button variant="ghost" onclick={reset}>Cancelar</Button>
      <Button onclick={confirmImport} disabled={busy || selected.size === 0}>
        {busy ? "Importando…" : `Importar ${selected.size} ${selected.size === 1 ? "transação" : "transações"}`}
      </Button>
    </div>
  {/if}
</section>
