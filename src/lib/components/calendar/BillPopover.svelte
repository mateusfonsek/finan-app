<script lang="ts">
  import { onMount } from "svelte";
  import { push } from "svelte-spa-router";
  import { Button } from "$lib/components/ui/button";
  import Spinner from "$lib/components/ui/Spinner.svelte";
  import { locale } from "$lib/i18n/locale.svelte";
  import { formatMoney } from "$lib/format/money";
  import { popover } from "$lib/motion";
  import { portal } from "$lib/actions/portal";
  import { transactionsMatchingRule } from "$lib/api/rules";
  import { settleBill, unsettleBill } from "$lib/api/bills";
  import TransactionSearchDialog from "./TransactionSearchDialog.svelte";
  import { billState, daysOverdue, dueDateOf } from "./bill";
  import type { CalendarEvent, Transaction } from "$lib/bindings";

  const t = locale.t;

  type Props = {
    event: CalendarEvent;
    /** The occurrence's month, `YYYY-MM` — the calendar's current month. */
    dueMonth: string;
    /** The chip that was clicked. The popover grows from it. */
    anchor: HTMLElement;
    today: string;
    onClose: () => void;
    onChanged: () => void;
  };

  let { event, dueMonth, anchor, today, onClose, onChanged }: Props = $props();

  const WIDTH = 320;
  const MARGIN = 8;

  let panelEl: HTMLElement | undefined = $state();
  let candidates = $state<Transaction[] | null>(null);
  let chosen = $state<number | null>(null);
  let busy = $state(false);
  let error = $state<string | null>(null);
  let searching = $state(false);
  /** A transaction chosen through the search dialog, which by definition is not
   *  in `candidates` — without holding it here the user would pick something and
   *  then not see what they picked. */
  let picked = $state<Transaction | null>(null);

  let billStatus = $derived(billState(event, dueMonth, today));
  let due = $derived(dueDateOf(event, dueMonth));

  let style = $derived.by(() => {
    const r = anchor.getBoundingClientRect();
    const left = Math.min(Math.max(MARGIN, r.left), window.innerWidth - WIDTH - MARGIN);
    return `position: fixed; top: ${r.bottom + 6}px; left: ${left}px; width: ${WIDTH}px;`;
  });

  $effect(() => {
    if (billStatus === "paid") return;
    const id = event.rule_id;
    void transactionsMatchingRule(id)
      .then((r) => (candidates = r.transactions))
      .catch(() => (candidates = []));
  });

  onMount(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.stopPropagation();
        close();
      }
    };
    const onDown = (e: MouseEvent) => {
      const target = e.target as Node;
      if (!panelEl?.contains(target) && !anchor.contains(target)) close();
    };
    // The calendar lives in a scrolling content pane, not the window itself,
    // so a `fixed` popover positioned once from `anchor` would float away
    // from its chip on scroll. Close instead of re-tracking the anchor —
    // that is what native macOS menus do.
    // A scroll INSIDE the panel is the user reading the candidate list, not
    // leaving the popover. Only a scroll of what lies behind detaches it from
    // its chip, and only that should close it.
    const onScroll = (e: Event) => {
      if (panelEl?.contains(e.target as Node)) return;
      close();
    };
    window.addEventListener("keydown", onKey, true);
    window.addEventListener("mousedown", onDown);
    window.addEventListener("scroll", onScroll, true);
    queueMicrotask(() => panelEl?.querySelector<HTMLElement>("[data-autofocus]")?.focus());
    return () => {
      window.removeEventListener("keydown", onKey, true);
      window.removeEventListener("mousedown", onDown);
      window.removeEventListener("scroll", onScroll, true);
    };
  });

  /** Focus goes back where it came from: the chip is what the reader was on. */
  function close() {
    anchor.focus();
    onClose();
  }

  async function confirm() {
    busy = true;
    error = null;
    try {
      await settleBill(event.rule_id, dueMonth, chosen);
      onChanged();
      close();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }

  async function undo() {
    busy = true;
    error = null;
    try {
      await unsettleBill(event.rule_id, dueMonth);
      onChanged();
      close();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }

  function shortDate(iso: string): string {
    const [y, m, d] = iso.split("-").map(Number);
    return new Date(y, m - 1, d).toLocaleDateString(locale.dateLocale, {
      day: "numeric",
      month: "short",
    });
  }
</script>

<div
  bind:this={panelEl}
  use:portal
  transition:popover={{ origin: "top left" }}
  style={style}
  class="material-pop z-50 p-3 flex flex-col gap-2.5"
  role="dialog"
  aria-label={event.pattern}
>
  <div class="flex flex-col gap-0.5">
    <span class="text-callout font-semibold text-fg truncate">{event.pattern}</span>
    <span class="text-foot text-fg-subtle">
      {#if billStatus === "paid"}
        <!-- Date and amount come from the same transaction row: either both
             are there, or the payment never passed through the statement. -->
        {event.paid_date
          ? t("bill_popover.paid_on", {
              date: shortDate(event.paid_date),
              amount: formatMoney(event.paid_amount!),
            })
          : t("bill_popover.paid_outside")}
      {:else if billStatus === "overdue" && due}
        {daysOverdue(due, today) === 1
          ? t("calendar.bill_state_overdue_one")
          : t("calendar.bill_state_overdue_days", { n: daysOverdue(due, today) })}
      {:else if due}
        {t("bill_popover.due_on", { date: shortDate(due) })}
      {/if}
    </span>
  </div>

  <div class="hairline"></div>

  {#if error}
    <p class="text-foot text-neg">{error}</p>
  {/if}

  {#if billStatus === "paid"}
    <div class="flex items-center justify-between gap-2">
      {#if event.manually_settled}
        <span class="text-foot text-fg-subtle">{t("bill_popover.settled_by_you")}</span>
        <Button size="sm" variant="outline" onclick={undo} disabled={busy} data-autofocus>
          {t("bill_popover.undo")}
        </Button>
      {:else}
        <span class="text-foot text-fg-subtle">{t("bill_popover.from_statement")}</span>
      {/if}
    </div>
  {:else}
    <label class="flex items-start gap-2 cursor-default">
      <input type="radio" name="settle" checked={chosen === null}
             onchange={() => (chosen = null)} class="mt-0.5" data-autofocus />
      <span class="flex flex-col gap-0.5 min-w-0">
        <span class="text-sub text-fg font-medium">{t("bill_popover.outside_title")}</span>
        <span class="text-foot text-fg-subtle leading-relaxed">{t("bill_popover.outside_desc")}</span>
      </span>
    </label>

    <div class="flex flex-col gap-1">
      <span class="text-foot text-fg-subtle">{t("bill_popover.pick_transaction")}</span>
      {#if candidates === null}
        <div class="py-2 grid place-items-center"><Spinner size={14} /></div>
      {:else if candidates.length === 0}
        <p class="text-foot text-fg-faint leading-relaxed">{t("bill_popover.no_candidates")}</p>
      {:else}
        <ul class="max-h-[196px] overflow-y-auto card-inset divide-y divide-border-subtle">
          {#each candidates as tx (tx.id)}
            <li>
              <label class="flex items-center gap-2 px-2 py-1.5 cursor-default hover:bg-hover">
                <input type="radio" name="settle" checked={chosen === tx.id}
                       onchange={() => (chosen = tx.id)} />
                <span class="text-foot tabular text-fg-subtle shrink-0">{shortDate(tx.date)}</span>
                <span class="text-foot text-fg truncate flex-1">{tx.description}</span>
                <span class="text-foot tabular shrink-0">{formatMoney(tx.amount)}</span>
              </label>
            </li>
          {/each}
        </ul>
      {/if}

      {#if picked}
        <!-- Outside the candidate list because it is not in it: the search
             exists for payments this rule's snippets never match. -->
        <label class="flex items-center gap-2 px-2 py-1.5 card-inset cursor-default">
          <input type="radio" name="settle" checked={chosen === picked.id}
                 onchange={() => (chosen = picked!.id)} />
          <span class="text-foot tabular text-fg-subtle shrink-0">{shortDate(picked.date)}</span>
          <span class="text-foot text-fg truncate flex-1">{picked.description}</span>
          <span class="text-foot tabular shrink-0">{formatMoney(picked.amount)}</span>
        </label>
      {/if}

      <button
        type="button"
        onclick={() => (searching = true)}
        class="text-foot text-fg-subtle hover:text-fg self-start
               transition-colors duration-[var(--dur-fast)]"
      >
        {t("bill_popover.search_other")}
      </button>

    </div>

    <div class="flex items-center justify-end gap-2 pt-0.5">
      <!-- Where the due day and the lead actually live. The popover settles one
           occurrence; changing the bill itself is a different place. -->
      <button
        type="button"
        onclick={() => { close(); void push("/rules"); }}
        class="text-foot text-fg-subtle hover:text-fg mr-auto transition-colors duration-[var(--dur-fast)]"
      >
        {t("bill_popover.open_rule")}
      </button>
      <Button size="sm" variant="ghost" onclick={close} disabled={busy}>
        {t("common.cancel")}
      </Button>
      <Button size="sm" onclick={confirm} disabled={busy}>
        {busy ? t("bill_popover.settling") : t("bill_popover.settle")}
      </Button>
    </div>
  {/if}
</div>

{#if searching}
  <TransactionSearchDialog
    ruleId={event.rule_id}
    billLabel={event.pattern}
    {dueMonth}
    onPick={(tx) => {
      picked = tx;
      chosen = tx.id;
      searching = false;
    }}
    onClose={() => (searching = false)}
  />
{/if}
