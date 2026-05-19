<script lang="ts">
  import { onMount } from "svelte";
  import { push } from "svelte-spa-router";
  import { Button } from "$lib/components/ui/button";
  import DropZone from "$lib/components/import/DropZone.svelte";
  import ImportPreview from "$lib/components/import/ImportPreview.svelte";
  import { formatMoney } from "$lib/format/money";
  import { createOrGetAccount } from "$lib/api/accounts";
  import { checkExistingFitids, insertTransactions } from "$lib/api/transactions";
  import { listCategories } from "$lib/api/categories";
  import { createRule, deleteRuleWithCleanup, updateRule } from "$lib/api/rules";
  import { autoClassifyWithCnpj } from "$lib/api/suggestions";
  import type { ParsedOfx } from "$lib/ofx/types";
  import { detectReversalPairs, type ReversalInfo } from "$lib/ofx/reversals";
  import type {
    Account,
    AutoClassifyReport,
    Category,
    InsertResult,
    NewTransaction,
    Rule,
  } from "$lib/bindings";

  type PendingImport = { file: File; parsed: ParsedOfx };

  let pending = $state<PendingImport | null>(null);
  let account = $state<Account | null>(null);
  let duplicateFitids = $state<Set<string>>(new Set());
  let selected = $state<Set<string>>(new Set());
  let reversalMap = $state<Map<string, ReversalInfo>>(new Map());
  let busy = $state(false);
  let busyMsg = $state("");
  let error = $state<string | null>(null);

  // Post-import state
  let importResult = $state<InsertResult | null>(null);
  let autoReport = $state<AutoClassifyReport | null>(null);
  let categories = $state<Category[]>([]);
  /** category chosen per unresolved CNPJ (keyed by cnpj) */
  let chosen = $state<Record<string, number | null>>({});
  let busyKey = $state<string | null>(null);

  onMount(() => {
    void listCategories().then((c) => (categories = c));
    const stash = (window as unknown as { __finanPending?: PendingImport }).__finanPending;
    if (stash) {
      pending = stash;
      (window as unknown as { __finanPending?: PendingImport }).__finanPending = undefined;
      void prepareImport(stash.parsed);
    }
  });

  async function prepareImport(parsed: ParsedOfx) {
    busy = true;
    busyMsg = "Preparando…";
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
      reversalMap = detectReversalPairs(parsed.transactions);
      // Default: importar tudo MENOS duplicadas e MENOS estornos+revertidos.
      selected = new Set(
        fitids.filter(
          (f) => !duplicateFitids.has(f) && !reversalMap.has(f),
        ),
      );
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
      busyMsg = "";
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
      busyMsg = "Importando…";
      importResult = await insertTransactions(account.id, toInsert);
      busyMsg = "Resolvendo CNPJs novos…";
      autoReport = await autoClassifyWithCnpj(account.id);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
      busyMsg = "";
    }
  }

  function replaceRule(updated: Rule) {
    if (!autoReport) return;
    autoReport = {
      ...autoReport,
      created_rules: autoReport.created_rules.map((r) =>
        r.id === updated.id ? updated : r,
      ),
    };
  }

  async function onChangeRuleCategory(rule: Rule, newCategoryId: number) {
    if (newCategoryId === rule.category_id) return;
    busyKey = `rule:${rule.id}`;
    try {
      const updated = await updateRule(rule.id, {
        pattern: rule.pattern,
        category_id: newCategoryId,
        priority: rule.priority,
        due_day: rule.due_day,
        display_name: rule.display_name,
      });
      replaceRule(updated);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      busyKey = null;
    }
  }

  async function onChangeRuleName(rule: Rule, newName: string) {
    const trimmed = newName.trim();
    const next = trimmed.length === 0 ? null : trimmed;
    if (next === (rule.display_name ?? null)) return;
    busyKey = `rule:${rule.id}`;
    try {
      const updated = await updateRule(rule.id, {
        pattern: rule.pattern,
        category_id: rule.category_id,
        priority: rule.priority,
        due_day: rule.due_day,
        display_name: next,
      });
      replaceRule(updated);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      busyKey = null;
    }
  }

  async function onDeleteRule(rule: Rule) {
    if (!autoReport) return;
    const ok = confirm(
      `Apagar regra "${rule.pattern}"? As transações categorizadas por ela voltarão a ficar sem categoria.`,
    );
    if (!ok) return;
    busyKey = `rule:${rule.id}`;
    try {
      await deleteRuleWithCleanup(rule.id);
      autoReport = {
        ...autoReport,
        created_rules: autoReport.created_rules.filter((r) => r.id !== rule.id),
      };
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      busyKey = null;
    }
  }

  async function onCreateRuleForCnpj(cnpj: string) {
    if (!autoReport) return;
    const categoryId = chosen[cnpj];
    if (categoryId == null) {
      error = "Escolha uma categoria antes de criar a regra.";
      return;
    }
    const u = autoReport.unresolved.find((x) => x.cnpj === cnpj);
    busyKey = `cnpj:${cnpj}`;
    error = null;
    try {
      const rule = await createRule({
        pattern: cnpj,
        category_id: categoryId,
        priority: 10,
        due_day: null,
        display_name: u?.razao_social ?? u?.nome_fantasia ?? null,
      });
      autoReport = {
        ...autoReport,
        created_rules: [...autoReport.created_rules, rule],
        unresolved: autoReport.unresolved.filter((x) => x.cnpj !== cnpj),
      };
      delete chosen[cnpj];
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      busyKey = null;
    }
  }

  function reset() {
    pending = null;
    account = null;
    selected = new Set();
    duplicateFitids = new Set();
    reversalMap = new Map();
    importResult = null;
    autoReport = null;
    chosen = {};
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
  {:else if importResult && autoReport}
    <!-- ============ Post-import view ============ -->
    <div class="rounded-lg border border-border-subtle bg-surface p-4 flex flex-col gap-1.5">
      <div class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint">
        Resultado do import
      </div>
      <div class="text-[13px] text-fg">
        <span class="font-medium">{importResult.inserted}</span>
        {importResult.inserted === 1 ? "transação importada" : "transações importadas"}
        {#if importResult.skipped_duplicates > 0}
          · <span class="text-fg-faint">{importResult.skipped_duplicates} duplicadas ignoradas</span>
        {/if}
      </div>
      <div class="text-[12px] text-fg-muted">
        <span class="text-fg font-medium">{autoReport.txs_classified}</span>
        categorizada{autoReport.txs_classified === 1 ? "" : "s"} automaticamente
        via regras existentes + CNPJ novo.
      </div>
    </div>

    {#if autoReport.created_rules.length > 0}
      <div class="rounded-xl bg-surface border border-border-subtle p-4 flex flex-col gap-3">
        <div class="flex items-baseline justify-between">
          <div class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint">
            Regras criadas via CNPJ ({autoReport.created_rules.length})
          </div>
          <div class="text-[10.5px] text-fg-faint">
            Apagar uma regra desfaz a categorização das transações casadas por ela.
          </div>
        </div>
        <div class="flex flex-col gap-2">
          {#each autoReport.created_rules as r (r.id)}
            <div class="grid grid-cols-[1fr_180px_auto] gap-2 items-center border-b border-border-subtle pb-2 last:border-b-0 last:pb-0">
              <div class="flex flex-col gap-0.5 min-w-0">
                <input
                  type="text"
                  value={r.display_name ?? ""}
                  placeholder={r.pattern}
                  onblur={(e) => {
                    void onChangeRuleName(r, (e.currentTarget as HTMLInputElement).value);
                  }}
                  onkeydown={(e) => {
                    if (e.key === "Enter") (e.currentTarget as HTMLInputElement).blur();
                  }}
                  class="rounded-md border border-transparent hover:border-border focus:border-accent bg-transparent focus:bg-surface-2 px-2 py-1 text-[12.5px] text-fg font-medium focus:outline-none w-full"
                  title="Nome amigável da regra"
                />
                <div class="text-[10.5px] text-fg-faint tabular px-2">{r.pattern}</div>
              </div>
              <select
                value={String(r.category_id)}
                onchange={(e) => {
                  const v = Number((e.currentTarget as HTMLSelectElement).value);
                  void onChangeRuleCategory(r, v);
                }}
                disabled={busyKey === `rule:${r.id}`}
                class="rounded-md border border-border bg-surface-2 px-2 py-1 text-[12px] text-fg"
                title="Categoria sugerida pela CNAE — você pode alterar"
              >
                {#each categories as c}
                  <option value={String(c.id)}>{c.name}</option>
                {/each}
              </select>
              <Button
                variant="ghost"
                onclick={() => onDeleteRule(r)}
                disabled={busyKey === `rule:${r.id}`}
              >
                {busyKey === `rule:${r.id}` ? "…" : "Apagar"}
              </Button>
            </div>
          {/each}
        </div>
      </div>
    {/if}

    {#if autoReport.unresolved.length > 0}
      <div class="rounded-xl bg-surface border border-border-subtle p-4 flex flex-col gap-3">
        <div class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint">
          CNPJs sem mapeamento CNAE ({autoReport.unresolved.length})
        </div>
        <div class="text-[10.5px] text-fg-faint">
          A BrasilAPI retornou dados, mas a atividade (CNAE) não está mapeada
          pra uma categoria. Escolha manualmente.
        </div>
        <div class="flex flex-col gap-2">
          {#each autoReport.unresolved as u}
            <div class="grid grid-cols-[1fr_180px_auto] gap-2 items-center border-b border-border-subtle pb-2 last:border-b-0 last:pb-0">
              <div class="flex flex-col">
                <div class="text-[12.5px] text-fg font-medium">
                  {u.razao_social ?? u.nome_fantasia ?? u.cnpj}
                </div>
                <div class="text-[10.5px] text-fg-faint tabular">
                  {u.cnpj}
                  {#if u.cnae_fiscal_descricao}
                    · CNAE {u.cnae_fiscal ?? "—"} · {u.cnae_fiscal_descricao}
                  {/if}
                </div>
              </div>
              <select
                value={chosen[u.cnpj] == null ? "" : String(chosen[u.cnpj])}
                onchange={(e) => {
                  const v = (e.currentTarget as HTMLSelectElement).value;
                  chosen[u.cnpj] = v === "" ? null : Number(v);
                }}
                class="rounded-md border border-border bg-surface-2 px-2 py-1 text-[12px] text-fg"
              >
                <option value="">— categoria —</option>
                {#each categories as c}
                  <option value={String(c.id)}>{c.name}</option>
                {/each}
              </select>
              <Button
                onclick={() => onCreateRuleForCnpj(u.cnpj)}
                disabled={busyKey === `cnpj:${u.cnpj}` || chosen[u.cnpj] == null}
              >
                {busyKey === `cnpj:${u.cnpj}` ? "Criando…" : "Criar regra"}
              </Button>
            </div>
          {/each}
        </div>
      </div>
    {/if}

    {#if error}
      <div class="rounded-lg border border-border bg-surface p-3 text-sm text-neg">{error}</div>
    {/if}

    <div class="flex justify-end gap-2 sticky bottom-0 bg-bg pt-3 border-t border-border-subtle">
      <Button variant="ghost" onclick={reset}>Importar outro arquivo</Button>
      <Button onclick={() => push("/transactions")}>Ver transações</Button>
    </div>
  {:else}
    <!-- ============ Preview view ============ -->
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
          {reversalMap}
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
        {#if reversalMap.size > 0}
          <div class="flex justify-between" title="Pares estorno/reembolso ↔ transação revertida (desmarcados por padrão)">
            <span class="text-fg-muted">Pares neutralizados</span>
            <span class="tabular text-fg-faint">{reversalMap.size / 2}</span>
          </div>
        {/if}
      </aside>
    </div>

    {#if error}
      <div class="rounded-lg border border-border bg-surface p-3 text-sm text-neg">{error}</div>
    {/if}

    <div class="flex justify-end gap-2 sticky bottom-0 bg-bg pt-3 border-t border-border-subtle">
      <Button variant="ghost" onclick={reset}>Cancelar</Button>
      <Button onclick={confirmImport} disabled={busy || selected.size === 0}>
        {busy ? busyMsg || "Importando…" : `Importar ${selected.size} ${selected.size === 1 ? "transação" : "transações"}`}
      </Button>
    </div>
  {/if}
</section>
