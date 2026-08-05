<script lang="ts">
  import { locale } from "$lib/i18n/locale.svelte";

  const t = locale.t;
  import { onMount } from "svelte";
  import { push } from "svelte-spa-router";
  import { confirm } from "@tauri-apps/plugin-dialog";
  import { Button } from "$lib/components/ui/button";
  import Page from "$lib/components/ui/Page.svelte";
  import Card from "$lib/components/ui/Card.svelte";
  import Icon from "$lib/components/ui/Icon.svelte";
  import ErrorNote from "$lib/components/ui/ErrorNote.svelte";
  import Spinner from "$lib/components/ui/Spinner.svelte";
  import { rise } from "$lib/motion";
  import DropZone from "$lib/components/import/DropZone.svelte";
  import ImportPreview from "$lib/components/import/ImportPreview.svelte";
  import { formatMoney } from "$lib/format/money";
  import { createOrGetAccount } from "$lib/api/accounts";
  import { checkExistingTxKeys, insertTransactions, txKeyString } from "$lib/api/transactions";
  import { listCategories } from "$lib/api/categories";
  import { createRule, deleteRuleWithCleanup, updateRule } from "$lib/api/rules";
  import type { ParsedOfx } from "$lib/ofx/types";
  import { takeStashed } from "$lib/ofx/open";
  import { activity } from "$lib/stores/activity.svelte";
  import { watch, type Discovery } from "$lib/stores/watch.svelte";
  import { loadOfxFromPath } from "$lib/ofx/load";
  import { detectReversalPairs, type ReversalInfo } from "$lib/ofx/reversals";
  import type {
    Account,
    Category,
    InsertResult,
    NewTransaction,
    Rule,
  } from "$lib/bindings";

  type PendingImport = { file: File; parsed: ParsedOfx };

  let pending = $state<PendingImport | null>(null);
  let account = $state<Account | null>(null);
  /** Set of composite `fitid|date|amount` keys — not bare FITIDs. */
  let duplicateKeys = $state<Set<string>>(new Set());
  let selected = $state<Set<string>>(new Set());
  let reversalMap = $state<Map<string, ReversalInfo>>(new Map());
  let busy = $state(false);
  let busyMsg = $state("");
  let error = $state<string | null>(null);

  // Post-import state
  let importResult = $state<InsertResult | null>(null);
  /** O relatório é do store, não desta tela: o enriquecimento continua rodando
   *  depois que a pessoa navega para outro lugar, e um `$state` local seria
   *  apagado no desmonte — voltar para cá mostraria uma tela vazia no meio de
   *  um trabalho em andamento. */
  let autoReport = $derived(activity.enrich.report);
  let categories = $state<Category[]>([]);
  /** category chosen per unresolved CNPJ (keyed by cnpj) */
  let chosen = $state<Record<string, number | null>>({});
  let busyKey = $state<string | null>(null);
  /** Content hash when the import came from a watched folder — used to mark the
   *  file imported at the end so it leaves the badge for good. */
  let watchHash = $state<string | undefined>(undefined);

  // The toast never interrupts someone already reviewing a statement.
  $effect(() => {
    watch.suppressToast = pending !== null;
    return () => (watch.suppressToast = false);
  });

  onMount(() => {
    void listCategories().then((c) => (categories = c));
    const stash = takeStashed();
    if (stash) {
      pending = { file: stash.file, parsed: stash.parsed };
      watchHash = stash.watchHash;
      void prepareImport(stash.parsed);
    }
  });

  // "Review" in the toast only asks; this screen opens. That covers both a
  // fresh mount (arriving from another route) and already being here — in the
  // second case `push("/import")` would remount nothing and `onMount` above
  // would never run again.
  $effect(() => {
    if (!watch.openRequest) return;
    const req = watch.takeOpenRequest();
    if (req) void openDiscovery(req);
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
      // Default: import everything EXCEPT duplicates and reversal pairs.
      // Duplicates key on the (fitid, date, amount) triple; reversals on fitid.
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
      if (watchHash) await watch.resolve(watchHash, "imported");
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
      busyMsg = "";
    }

    // Depois do `finally`, e sem `await`: a tela de resultado já pode aparecer.
    // O backend decide sozinho se há o que fazer — com o enriquecimento
    // desligado o job termina na hora com relatório vazio, o que elimina a ida
    // ao backend que este trecho fazia só para perguntar se valia a pena.
    if (!error && account) {
      activity.clear();
      void activity.start(account.id);
    }
  }

  function replaceRule(updated: Rule) {
    if (!autoReport) return;
    activity.patchReport({
      ...autoReport,
      created_rules: autoReport.created_rules.map((r) =>
        r.id === updated.id ? updated : r,
      ),
    });
  }

  async function onChangeRuleCategory(rule: Rule, newCategoryId: number) {
    if (newCategoryId === rule.category_id) return;
    busyKey = `rule:${rule.id}`;
    try {
      const updated = await updateRule(rule.id, {
        patterns: rule.patterns,
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
        patterns: rule.patterns,
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
    const label = rule.display_name ?? rule.patterns[0] ?? "";
    const ok = await confirm(t("import.delete_rule_confirm", { pattern: label }), {
      title: t("import.delete"),
      kind: "warning",
      okLabel: t("common.delete"),
      cancelLabel: t("common.cancel"),
    });
    if (!ok) return;
    busyKey = `rule:${rule.id}`;
    try {
      await deleteRuleWithCleanup(rule.id);
      activity.patchReport({
        ...autoReport,
        created_rules: autoReport.created_rules.filter((r) => r.id !== rule.id),
      });
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      busyKey = null;
    }
  }

  /** A rule created during import starts with one snippet (the tax id), but may
   *  have gained others since — the counter avoids showing half the truth. */
  function patternsLabel(r: Rule): string {
    const first = r.patterns[0] ?? "";
    return r.patterns.length > 1 ? `${first}  +${r.patterns.length - 1}` : first;
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
        patterns: [cnpj],
        category_id: categoryId,
        priority: 10,
        due_day: null,
        display_name: u?.razao_social ?? u?.nome_fantasia ?? null,
      });
      activity.patchReport({
        ...autoReport,
        created_rules: [...autoReport.created_rules, rule],
        unresolved: autoReport.unresolved.filter((x) => x.cnpj !== cnpj),
      });
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
    // Um import novo não herda o relatório do anterior.
    activity.clear();
    chosen = {};
    // Sem isso, "importar outro" com um arquivo solto (DropZone) herdaria o
    // hash do extrato anterior vindo da pasta observada.
    watchHash = undefined;
  }

  /** Loads the next discovered statement straight into the preview, reusing the
   *  screen we are already on. */
  async function openNextFromQueue() {
    const next = watch.discoveries[0];
    if (next) await openDiscovery(next);
  }

  /** Loads a specific discovery into the preview without leaving the screen. */
  async function openDiscovery(next: Discovery) {
    error = null;
    busy = true;
    try {
      // `pending` and the rest of the import state are only touched after the
      // new file has been read and parsed successfully, same as `onparsed`.
      // Clearing `pending` first (as `reset()` used to do here) left the screen
      // with no statement during the whole read, and if the load failed (file
      // moved, deleted, or an iCloud placeholder evicted after the scan) the
      // error landed with `pending` already null — the screen fell back to an
      // empty dropzone with no explanation.
      const loaded = await loadOfxFromPath(next.path);
      reset();
      pending = { file: loaded.file, parsed: loaded.parsed };
      watchHash = next.hash;
      await prepareImport(loaded.parsed);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
      // Takes the file out of the queue so it cannot block `discoveries[0]`;
      // otherwise every "next statement" click would hit it again and the ones
      // behind would never surface. *How* it leaves depends on the failure:
      // only non-OFX content becomes `invalid` (permanent); failing to read now
      // is transient and merely drops out of this round, returning on the next
      // scan.
      await watch.noteLoadFailure(next.hash, e);
    } finally {
      busy = false;
    }
  }
</script>

<Page title={t("nav.import")}>
  {#snippet toolbar()}
    {#if pending}
      <span class="chip text-fg-muted max-w-[280px]">
        <Icon name="fileText" size={12} />
        <span class="truncate">{pending.file.name}</span>
      </span>
    {/if}
  {/snippet}

  {#if !pending}
    <!-- A target has the size of a target: stretched across the full page it
         stops looking like an object and becomes an empty band. -->
    <div class="w-full max-w-xl mx-auto flex flex-col gap-4 pt-4">
      {#if error}
        <ErrorNote message={error} />
      {/if}
      <DropZone {onparsed} {onerror} />
    </div>
  {:else if importResult}
    <!-- ============ After the import ============ -->
    <!-- Explicit conclusion: what went in, what was skipped, what the app
         categorized on its own. Confirming the result is part of the job. -->
    <div
      class="card p-4 flex items-start gap-3"
      style="border-color: color-mix(in oklch, var(--color-pos) 30%, var(--color-border-subtle))"
      in:rise
    >
      <span
        class="w-8 h-8 shrink-0 grid place-items-center rounded-full"
        style="color: var(--color-pos); background: color-mix(in oklch, var(--color-pos) 15%, transparent);"
      >
        <Icon name="check" size={16} stroke={2.6} />
      </span>
      <div class="flex flex-col gap-1 min-w-0">
        <div class="text-title3 font-semibold text-fg">
          {importResult.inserted}
          {importResult.inserted === 1 ? t("import.imported_one") : t("import.imported_many")}
        </div>
        <div class="text-sub text-fg-muted">
          <!-- O relatório chega depois do import: até ele existir, esta linha
               mostra só o que já é verdade. -->
          {#if autoReport}
            <span class="text-fg font-medium">{autoReport.txs_classified}</span>
            {autoReport.txs_classified === 1
              ? t("import.classified_one")
              : t("import.classified_many")}
          {/if}
          {#if importResult.skipped_duplicates > 0}
            <span class="text-fg-subtle">
              · {t("import.skipped_dups", { n: importResult.skipped_duplicates })}
            </span>
          {/if}
        </div>
      </div>
    </div>

    {#if autoReport && autoReport.created_rules.length > 0}
      <Card
        title={t("import.rules_created_title", { n: autoReport.created_rules.length })}
        note={t("import.rules_created_hint")}
      >
        <div class="flex flex-col">
          {#each autoReport.created_rules as r (r.id)}
            <div
              class="grid grid-cols-[1fr_178px_auto] gap-2.5 items-center py-2
                     border-b border-border-subtle last:border-b-0"
            >
              <div class="flex flex-col gap-0.5 min-w-0">
                <!-- A field that only looks like one when touched: the list
                      stays calm, but everything remains editable in place. -->
                <input
                  type="text"
                  value={r.display_name ?? ""}
                  placeholder={r.patterns[0] ?? ""}
                  onblur={(e) => {
                    void onChangeRuleName(r, (e.currentTarget as HTMLInputElement).value);
                  }}
                  onkeydown={(e) => {
                    if (e.key === "Enter") (e.currentTarget as HTMLInputElement).blur();
                  }}
                  class="field !bg-transparent !border-transparent hover:!border-border
                         focus:!bg-surface-2 focus:!border-accent font-medium w-full"
                  title={t("import.rule_name_title")}
                />
                <div class="text-cap text-fg-subtle font-mono px-2 truncate">{patternsLabel(r)}</div>
              </div>
              <select
                value={String(r.category_id)}
                onchange={(e) => {
                  const v = Number((e.currentTarget as HTMLSelectElement).value);
                  void onChangeRuleCategory(r, v);
                }}
                disabled={busyKey === `rule:${r.id}`}
                class="field"
                title={t("import.rule_category_title")}
              >
                {#each categories as c}
                  <option value={String(c.id)}>{c.name}</option>
                {/each}
              </select>
              <Button
                variant="ghost"
                size="icon"
                onclick={() => onDeleteRule(r)}
                disabled={busyKey === `rule:${r.id}`}
                title={t("import.delete")}
                aria-label={`${t("import.delete")} ${r.display_name ?? r.patterns[0] ?? ""}`}
                class="hover:text-neg"
              >
                {#if busyKey === `rule:${r.id}`}
                  <Spinner size={12} />
                {:else}
                  <Icon name="trash2" size={13} />
                {/if}
              </Button>
            </div>
          {/each}
        </div>
      </Card>
    {/if}

    {#if autoReport && autoReport.unresolved.length > 0}
      <Card
        title={t("import.unresolved_title", { n: autoReport.unresolved.length })}
        note={t("import.unresolved_hint")}
      >
        <div class="flex flex-col">
          {#each autoReport.unresolved as u}
            <div
              class="grid grid-cols-[1fr_178px_auto] gap-2.5 items-center py-2
                     border-b border-border-subtle last:border-b-0"
            >
              <div class="flex flex-col min-w-0">
                <div class="text-callout text-fg font-medium truncate">
                  {u.razao_social ?? u.nome_fantasia ?? u.cnpj}
                </div>
                <div class="text-cap text-fg-subtle tabular truncate">
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
                aria-label={t("import.category_placeholder")}
                class="field"
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
      </Card>
    {/if}

    {#if error}
      <ErrorNote message={error} />
    {/if}

    <!-- Fixed action bar: translucent material, content passes underneath. -->
    <div
      class="material-chrome sticky bottom-0 -mx-8 px-8 py-3 mt-1 flex justify-end gap-2
             border-t border-border-subtle"
    >
      <Button variant="ghost" onclick={reset}>{t("import.import_another")}</Button>
      {#if watch.pendingCount > 0}
        <Button variant="outline" onclick={openNextFromQueue} disabled={busy}>
          {busy ? t("import.reading") : t("watch.queue_next", { n: watch.pendingCount })}
        </Button>
      {/if}
      <Button onclick={() => push("/transactions")}>{t("import.view_transactions")}</Button>
    </div>
  {:else}
    <!-- ============ Review before importing ============ -->
    {@const p = pending.parsed}
    <div class="grid grid-cols-[1fr_280px] gap-4 items-start">
      <div class="flex flex-col gap-3">
        <div class="card p-3.5 flex items-center gap-2.5 flex-wrap">
          <span
            class="w-8 h-8 shrink-0 grid place-items-center rounded-[var(--radius-md)] bg-accent-soft text-accent"
          >
            <Icon name={p.account.type === "credit_card" ? "creditCard" : "landmark"} size={15} />
          </span>
          <div class="flex flex-col min-w-0">
            <span class="text-callout font-medium text-fg truncate">{p.account.displayName}</span>
            <span class="text-cap text-fg-subtle truncate">
              {p.account.bank === "unknown" ? t("import.bank_unknown") : p.account.bank}
              ·
              <span
                title={p.account.type === "credit_card"
                  ? t("import.credit_card_title")
                  : t("import.checking_title")}
              >
                {p.account.type === "credit_card" ? t("import.credit_card") : t("import.checking")}
              </span>
            </span>
          </div>
          <div class="ml-auto text-sub text-fg-subtle tabular text-right">
            {t("import.tx_count", { n: p.transactions.length })}
            <div class="text-cap text-fg-subtle">
              {p.summary.earliest ?? "?"} → {p.summary.latest ?? "?"}
            </div>
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

      <aside class="card p-4 flex flex-col gap-2 text-sub sticky top-[calc(var(--titlebar-h)+64px)]">
        <div class="section-title mb-1">{t("import.summary")}</div>
        <div class="flex justify-between">
          <span class="text-fg-muted">{t("import.sum_inflows")}</span>
          <span class="tabular text-pos">{formatMoney(p.summary.totalIn)}</span>
        </div>
        <div class="flex justify-between">
          <span class="text-fg-muted">{t("import.sum_outflows")}</span>
          <span class="tabular text-fg">{formatMoney(p.summary.totalOut)}</span>
        </div>
        <div class="flex justify-between border-t border-border-subtle pt-2 mt-1">
          <span class="text-fg-muted">{t("import.sum_net")}</span>
          <span class="tabular font-semibold text-fg">{formatMoney(p.summary.net)}</span>
        </div>
        <div class="flex justify-between mt-2">
          <span class="text-fg-muted">{t("import.sum_selected")}</span>
          <span class="tabular font-medium text-accent">{selected.size}</span>
        </div>
        <div class="flex justify-between">
          <span class="text-fg-muted">{t("import.sum_duplicates")}</span>
          <span class="tabular text-fg-subtle">{duplicateKeys.size}</span>
        </div>
        {#if reversalMap.size > 0}
          <div class="flex justify-between" title={t("import.neutralized_pairs_title")}>
            <span class="text-fg-muted">{t("import.neutralized_pairs")}</span>
            <span class="tabular text-fg-subtle">{reversalMap.size / 2}</span>
          </div>
        {/if}
      </aside>
    </div>

    {#if error}
      <ErrorNote message={error} />
    {/if}

    <div
      class="material-chrome sticky bottom-0 -mx-8 px-8 py-3 mt-1 flex justify-end gap-2
             border-t border-border-subtle"
    >
      <Button variant="ghost" onclick={reset}>{t("common.cancel")}</Button>
      <Button onclick={confirmImport} disabled={busy || selected.size === 0}>
        {#if busy}
          <Spinner size={12} />
          {busyMsg || t("import.importing")}
        {:else}
          {selected.size === 1
            ? t("import.import_n_one", { n: selected.size })
            : t("import.import_n_many", { n: selected.size })}
        {/if}
      </Button>
    </div>
  {/if}
</Page>
