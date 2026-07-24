<script lang="ts">
  import { locale } from "$lib/i18n/locale.svelte";

  const t = locale.t;
  import { onMount } from "svelte";
  import { push } from "svelte-spa-router";
  import { Button } from "$lib/components/ui/button";
  import DropZone from "$lib/components/import/DropZone.svelte";
  import ImportPreview from "$lib/components/import/ImportPreview.svelte";
  import { formatMoney } from "$lib/format/money";
  import { createOrGetAccount } from "$lib/api/accounts";
  import { checkExistingTxKeys, insertTransactions, txKeyString } from "$lib/api/transactions";
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
  /** Set de chaves compostas `fitid|date|amount` — não confundir com FITID puro. */
  let duplicateKeys = $state<Set<string>>(new Set());
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
    busyMsg = t("import.preparing");
    try {
      account = await createOrGetAccount({
        name: parsed.account.displayName,
        bank: parsed.account.bank === "unknown" ? null : parsed.account.bank,
        ofx_acctid: parsed.account.ofxAcctid,
        kind: parsed.account.type,
      });
      const candidates = parsed.transactions
        .filter((t): t is typeof t & { fitid: string } => !!t.fitid)
        .map((t) => ({ ofx_fitid: t.fitid, date: t.date, amount: t.amount }));
      duplicateKeys = await checkExistingTxKeys(account.id, candidates);
      reversalMap = detectReversalPairs(parsed.transactions);
      // Default: importar tudo MENOS duplicadas e MENOS estornos+revertidos.
      // Duplicata é por tripla composta (fitid, date, amount); reversal é por fitid.
      selected = new Set(
        parsed.transactions
          .filter((t) => {
            if (!t.fitid) return false;
            const key = txKeyString({ ofx_fitid: t.fitid, date: t.date, amount: t.amount });
            return !duplicateKeys.has(key) && !reversalMap.has(t.fitid);
          })
          .map((t) => t.fitid as string),
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
      busyMsg = t("import.importing");
      importResult = await insertTransactions(account.id, toInsert);
      busyMsg = t("import.resolving_cnpj");
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
    const ok = confirm(t("import.delete_rule_confirm", { pattern: rule.pattern }));
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
      error = t("import.choose_category_first");
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
    duplicateKeys = new Set();
    reversalMap = new Map();
    importResult = null;
    autoReport = null;
    chosen = {};
  }
</script>

<section class="p-8 max-w-5xl mx-auto flex flex-col gap-5">
  <header class="flex items-baseline justify-between">
    <h2 class="text-xl font-semibold tracking-tight" style="font-family: var(--font-display)">
      {t("nav.import")}
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
        {t("import.result_title")}
      </div>
      <div class="text-[13px] text-fg">
        <span class="font-medium">{importResult.inserted}</span>
        {importResult.inserted === 1 ? t("import.imported_one") : t("import.imported_many")}
        {#if importResult.skipped_duplicates > 0}
          · <span class="text-fg-faint">{t("import.skipped_dups", { n: importResult.skipped_duplicates })}</span>
        {/if}
      </div>
      <div class="text-[12px] text-fg-muted">
        <span class="text-fg font-medium">{autoReport.txs_classified}</span>
        {autoReport.txs_classified === 1 ? t("import.classified_one") : t("import.classified_many")}
      </div>
    </div>

    {#if autoReport.created_rules.length > 0}
      <div class="rounded-xl bg-surface border border-border-subtle p-4 flex flex-col gap-3">
        <div class="flex items-baseline justify-between">
          <div class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint">
            {t("import.rules_created_title", { n: autoReport.created_rules.length })}
          </div>
          <div class="text-[10.5px] text-fg-faint">
            {t("import.rules_created_hint")}
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
                  title={t("import.rule_name_title")}
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
                title={t("import.rule_category_title")}
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
                {busyKey === `rule:${r.id}` ? "…" : t("import.delete")}
              </Button>
            </div>
          {/each}
        </div>
      </div>
    {/if}

    {#if autoReport.unresolved.length > 0}
      <div class="rounded-xl bg-surface border border-border-subtle p-4 flex flex-col gap-3">
        <div class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint">
          {t("import.unresolved_title", { n: autoReport.unresolved.length })}
        </div>
        <div class="text-[10.5px] text-fg-faint">
          {t("import.unresolved_hint")}
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
                    {t("import.cnae_label", { code: u.cnae_fiscal ?? "—", desc: u.cnae_fiscal_descricao })}
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
                <option value="">{t("import.category_placeholder")}</option>
                {#each categories as c}
                  <option value={String(c.id)}>{c.name}</option>
                {/each}
              </select>
              <Button
                onclick={() => onCreateRuleForCnpj(u.cnpj)}
                disabled={busyKey === `cnpj:${u.cnpj}` || chosen[u.cnpj] == null}
              >
                {busyKey === `cnpj:${u.cnpj}` ? t("import.creating") : t("import.create_rule")}
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
      <Button variant="ghost" onclick={reset}>{t("import.import_another")}</Button>
      <Button onclick={() => push("/transactions")}>{t("import.view_transactions")}</Button>
    </div>
  {:else}
    <!-- ============ Preview view ============ -->
    {@const p = pending.parsed}
    <div class="grid grid-cols-[1fr_280px] gap-4 items-start">
      <div class="flex flex-col gap-3">
        <div class="rounded-lg border border-border-subtle bg-surface p-4 flex items-center gap-3">
          <div class="text-[10.5px] uppercase tracking-wider font-semibold text-accent-hi bg-accent-soft border border-accent/30 rounded-full px-2 py-0.5">
            {p.account.bank === "unknown" ? t("import.bank_unknown") : p.account.bank}
          </div>
          <div
            class="text-[10.5px] uppercase tracking-wider font-semibold rounded-full px-2 py-0.5 border"
            style={p.account.type === "credit_card"
              ? "color: var(--color-cat-amarelo); border-color: var(--color-cat-amarelo); background: color-mix(in oklch, var(--color-cat-amarelo) 14%, transparent);"
              : "color: var(--color-fg-muted); border-color: var(--color-border); background: var(--color-surface-2);"}
            title={p.account.type === "credit_card"
              ? t("import.credit_card_title")
              : t("import.checking_title")}
          >
            {p.account.type === "credit_card" ? t("import.credit_card") : t("import.checking")}
          </div>
          <div class="text-sm font-medium">{p.account.displayName}</div>
          <div class="ml-auto text-xs text-fg-faint tabular">
            {t("import.tx_count", { n: p.transactions.length })} ·
            {p.summary.earliest ?? "?"} → {p.summary.latest ?? "?"}
          </div>
        </div>

        <ImportPreview
          transactions={p.transactions}
          {duplicateKeys}
          {reversalMap}
          {selected}
          ontoggle={toggle}
          ontoggleAll={toggleAll}
        />
      </div>

      <aside class="rounded-lg border border-border-subtle bg-surface p-4 flex flex-col gap-2 text-[12px]">
        <div class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint mb-1">{t("import.summary")}</div>
        <div class="flex justify-between"><span class="text-fg-muted">{t("import.sum_inflows")}</span><span class="tabular text-pos">{formatMoney(p.summary.totalIn)}</span></div>
        <div class="flex justify-between"><span class="text-fg-muted">{t("import.sum_outflows")}</span><span class="tabular">{formatMoney(p.summary.totalOut)}</span></div>
        <div class="flex justify-between border-t border-border-subtle pt-2 mt-1"><span class="text-fg-muted">{t("import.sum_net")}</span><span class="tabular font-semibold">{formatMoney(p.summary.net)}</span></div>
        <div class="flex justify-between mt-2"><span class="text-fg-muted">{t("import.sum_selected")}</span><span class="tabular">{selected.size}</span></div>
        <div class="flex justify-between"><span class="text-fg-muted">{t("import.sum_duplicates")}</span><span class="tabular text-fg-faint">{duplicateKeys.size}</span></div>
        {#if reversalMap.size > 0}
          <div class="flex justify-between" title={t("import.neutralized_pairs_title")}>
            <span class="text-fg-muted">{t("import.neutralized_pairs")}</span>
            <span class="tabular text-fg-faint">{reversalMap.size / 2}</span>
          </div>
        {/if}
      </aside>
    </div>

    {#if error}
      <div class="rounded-lg border border-border bg-surface p-3 text-sm text-neg">{error}</div>
    {/if}

    <div class="flex justify-end gap-2 sticky bottom-0 bg-bg pt-3 border-t border-border-subtle">
      <Button variant="ghost" onclick={reset}>{t("common.cancel")}</Button>
      <Button onclick={confirmImport} disabled={busy || selected.size === 0}>
        {busy ? (busyMsg || t("import.importing")) : (selected.size === 1 ? t("import.import_n_one", { n: selected.size }) : t("import.import_n_many", { n: selected.size }))}
      </Button>
    </div>
  {/if}
</section>
