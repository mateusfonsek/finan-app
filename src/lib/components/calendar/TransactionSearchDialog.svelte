<script lang="ts">
  import { onMount } from "svelte";
  import Icon from "$lib/components/ui/Icon.svelte";
  import { Button } from "$lib/components/ui/button";
  import Spinner from "$lib/components/ui/Spinner.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import { locale } from "$lib/i18n/locale.svelte";
  import { formatMoney } from "$lib/format/money";
  import { dialog, scrim } from "$lib/motion";
  import { transactionsMatchingRule } from "$lib/api/rules";
  import { listTransactions } from "$lib/api/transactions";
  import { billLinks } from "$lib/api/bills";
  import type { BillLink, Transaction } from "$lib/bindings";

  const t = locale.t;

  type Props = {
    ruleId: number;
    /** Label of the bill being settled, for the header. */
    billLabel: string;
    /** The occurrence being settled, so its own link never blocks itself. */
    dueMonth: string;
    onPick: (transaction: Transaction) => void;
    onClose: () => void;
  };

  let { ruleId, billLabel, dueMonth, onPick, onClose }: Props = $props();

  /** How many of the newest transactions to offer before the user types. Enough
   *  to cover a few months of statements without loading a lifetime of them. */
  const RECENT_LIMIT = 60;

  /** Long enough that a typed word is one query, short enough to feel immediate. */
  const DEBOUNCE_MS = 180;

  let q = $state("");
  /** `q` settled. The query runs off this, so typing a word costs one request. */
  let query = $state("");
  let suggested = $state<Transaction[] | null>(null);
  let others = $state<Transaction[] | null>(null);
  /** A query is in flight over results already on screen. Distinct from the
   *  first load, which has nothing to show yet. */
  let refreshing = $state(false);
  let links = $state<BillLink[]>([]);
  let chosen = $state<number | null>(null);
  let error = $state<string | null>(null);
  let closeEl: HTMLButtonElement | undefined = $state();

  /** Only the newest query may write its results. Responses can land out of
   *  order, and a stale one overwriting a fresh one shows results for a term
   *  the user already moved past — with the field saying something else. */
  let issued = 0;

  onMount(() => {
    void Promise.all([transactionsMatchingRule(ruleId), billLinks()])
      .then(([matches, found]) => {
        suggested = matches.transactions;
        links = found;
      })
      .catch((e) => {
        suggested = [];
        error = e instanceof Error ? e.message : String(e);
      });
    queueMicrotask(() => closeEl?.focus());
  });

  $effect(() => {
    const typed = q;
    const handle = setTimeout(() => (query = typed.trim()), DEBOUNCE_MS);
    return () => clearTimeout(handle);
  });

  // Refetches as the settled term changes. `listTransactions` searches
  // description and notes server-side, which is what reaches transactions this
  // rule's snippets never match — the whole reason this dialog exists.
  //
  // `others` is deliberately NOT cleared here. Clearing it would collapse the
  // list to a spinner between every query, and with a content-sized dialog that
  // means the whole panel shrinks and grows under the cursor as you type.
  $effect(() => {
    const term = query;
    const mine = ++issued;
    refreshing = true;
    void listTransactions({
      account_id: null,
      month: null,
      category_id: null,
      q: term === "" ? null : term,
      limit: term === "" ? RECENT_LIMIT : null,
    })
      .then((r) => {
        if (mine === issued) others = r;
      })
      .catch(() => {
        if (mine === issued) others = [];
      })
      .finally(() => {
        if (mine === issued) refreshing = false;
      });
  });

  let suggestedIds = $derived(new Set((suggested ?? []).map((t) => t.id)));

  /** A suggestion is still a suggestion while it matches what was typed; the
   *  section is a shortcut, not a separate search. */
  let shownSuggested = $derived.by(() => {
    // Filters on the settled term, not the raw one, so both sections change in
    // the same frame. Filtering this one instantly is free, but it would make
    // the list update in two stages — the small version of the same jitter.
    const term = query.toLowerCase();
    const list = suggested ?? [];
    if (term === "") return list;
    return list.filter((tx) => tx.description.toLowerCase().includes(term));
  });

  /** Never repeat a suggestion down in "others". */
  let shownOthers = $derived((others ?? []).filter((tx) => !suggestedIds.has(tx.id)));

  let linkByTx = $derived.by(() => {
    const map = new Map<number, BillLink>();
    for (const l of links) {
      // The occurrence being settled must not block itself: re-opening a bill
      // that already points at a transaction should show it as choosable.
      if (l.due_month === dueMonth && l.rule_label === billLabel) continue;
      map.set(l.transaction_id, l);
    }
    return map;
  });

  let loading = $derived(suggested === null || others === null);
  let nothing = $derived(!loading && shownSuggested.length === 0 && shownOthers.length === 0);

  function shortDate(iso: string): string {
    const [y, m, d] = iso.split("-").map(Number);
    return new Date(y, m - 1, d).toLocaleDateString(locale.dateLocale, {
      day: "numeric",
      month: "short",
      year: y === new Date().getFullYear() ? undefined : "numeric",
    });
  }

  function monthLabel(month: string): string {
    const [y, m] = month.split("-").map(Number);
    return new Date(y, m - 1, 1).toLocaleDateString(locale.dateLocale, {
      month: "long",
      year: "numeric",
    });
  }

  function confirm() {
    const all = [...shownSuggested, ...shownOthers];
    const picked = all.find((tx) => tx.id === chosen);
    if (picked) onPick(picked);
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") onClose();
  }
</script>

<svelte:window onkeydown={onKeydown} />

<button
  type="button"
  class="fixed inset-0 z-70 bg-black/40"
  transition:scrim
  onclick={onClose}
  aria-label={t("common.close")}
></button>

<div
  class="fixed left-1/2 top-1/2 z-80 w-[560px] -translate-x-1/2 -translate-y-1/2
         card flex flex-col overflow-hidden"
  transition:dialog
  role="dialog"
  aria-modal="true"
  aria-label={t("tx_search.title")}
>
  <header class="px-4 pt-3.5 pb-3 border-b border-border-subtle flex items-start justify-between gap-3">
    <div class="flex flex-col gap-0.5 min-w-0">
      <h2 class="text-title3 font-semibold text-fg">{t("tx_search.title")}</h2>
      <p class="text-foot text-fg-subtle leading-relaxed">
        {t("tx_search.desc", { bill: billLabel, month: monthLabel(dueMonth) })}
      </p>
    </div>
    <button
      bind:this={closeEl}
      type="button"
      onclick={onClose}
      class="press shrink-0 text-fg-muted hover:text-fg"
      aria-label={t("common.close")}
    >
      <Icon name="x" size={13} stroke={2} />
    </button>
  </header>

  <div class="px-4 py-2 border-b border-border-subtle">
    <!-- One field, deliberately: the term is the filter. A date range would
         need a backend that does not have one. -->
    <div class="relative">
      <input class="field w-full pr-7" bind:value={q} placeholder={t("tx_search.search")} />
      <!-- Absolutely placed so an in-flight query never moves the field, the
           list, or the panel around it. -->
      {#if refreshing && !loading}
        <span class="absolute right-2 top-1/2 -translate-y-1/2 text-fg-faint">
          <Spinner size={12} />
        </span>
      {/if}
    </div>
  </div>

  <!-- A fixed height, not a content-sized one: a search panel whose frame
       resizes with the result count jumps under the cursor on every keystroke.
       The other dialogs in this app size to their content because their content
       does not change while open. -->
  <div class="overflow-y-auto h-[46vh]">
    {#if loading}
      <div class="py-10 grid place-items-center"><Spinner size={16} /></div>
    {:else if nothing}
      <EmptyState icon="search" title={t("tx_search.empty")} description={t("tx_search.empty_desc")} compact />
    {:else}
      {#each [["suggested", shownSuggested], ["others", shownOthers]] as [key, list]}
        {#if (list as Transaction[]).length > 0}
          <div class="px-4 py-1.5 bg-surface-2/60 text-cap font-semibold text-fg-subtle sticky top-0">
            {key === "suggested" ? t("tx_search.by_rule") : t("tx_search.others")}
          </div>
          <ul class="divide-y divide-border-subtle">
            {#each list as Transaction[] as tx (tx.id)}
              {@const taken = linkByTx.get(tx.id)}
              <li>
                <label
                  class="flex items-center gap-2.5 px-4 py-2 {taken
                    ? 'opacity-55'
                    : 'cursor-default hover:bg-hover'}"
                >
                  <input
                    type="radio"
                    name="tx"
                    disabled={taken !== undefined}
                    checked={chosen === tx.id}
                    onchange={() => (chosen = tx.id)}
                  />
                  <span class="text-foot tabular text-fg-subtle shrink-0 w-[62px]">
                    {shortDate(tx.date)}
                  </span>
                  <span class="flex flex-col min-w-0 flex-1">
                    <span class="text-sub text-fg truncate">{tx.description}</span>
                    {#if taken}
                      <span class="text-cap text-fg-faint">
                        {t("tx_search.already_pays", {
                          bill: taken.rule_label,
                          month: monthLabel(taken.due_month),
                        })}
                      </span>
                    {/if}
                  </span>
                  <span class="text-sub tabular shrink-0 {Number(tx.amount) < 0 ? 'text-fg' : 'text-pos'}">
                    {formatMoney(tx.amount)}
                  </span>
                </label>
              </li>
            {/each}
          </ul>
        {/if}
      {/each}
    {/if}
  </div>

  {#if error}
    <div class="px-4 py-2 border-t border-border-subtle text-foot text-neg">{error}</div>
  {/if}

  <footer class="px-4 py-2.5 border-t border-border-subtle flex items-center justify-end gap-2">
    <Button size="sm" variant="ghost" onclick={onClose}>{t("common.cancel")}</Button>
    <Button size="sm" onclick={confirm} disabled={chosen === null}>{t("tx_search.use")}</Button>
  </footer>
</div>
