<script lang="ts">
  import { onMount } from "svelte";
  import { Button } from "$lib/components/ui/button";
  import Spinner from "$lib/components/ui/Spinner.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import { locale } from "$lib/i18n/locale.svelte";
  import { dialog, scrim } from "$lib/motion";
  import { listRulesWithCount, updateRule } from "$lib/api/rules";
  import type { RuleWithCount } from "$lib/bindings";

  const t = locale.t;

  type Props = {
    /** Day of month the chosen rule will fall due on, from now on. `null` when
     *  opened from the month strip, where no day was clicked — then the dialog
     *  asks for one instead of inventing it. */
    day: number | null;
    onClose: () => void;
    onChanged: () => void;
  };

  let { day, onClose, onChanged }: Props = $props();

  let typedDay = $state<number | null>(null);
  let effectiveDay = $derived(day ?? typedDay);
  let dayIsValid = $derived(
    effectiveDay != null && Number.isInteger(effectiveDay) && effectiveDay >= 1 && effectiveDay <= 31,
  );

  let rules = $state<RuleWithCount[] | null>(null);
  let q = $state("");
  let busyId = $state<number | null>(null);
  let error = $state<string | null>(null);
  let closeEl: HTMLButtonElement | undefined = $state();

  onMount(() => {
    void listRulesWithCount()
      .then((r) => (rules = r))
      .catch((e) => (error = e instanceof Error ? e.message : String(e)));
    queueMicrotask(() => closeEl?.focus());
  });

  let shown = $derived.by(() => {
    const list = rules ?? [];
    const term = q.trim().toLowerCase();
    if (term === "") return list;
    return list.filter(
      (r) =>
        (r.display_name ?? "").toLowerCase().includes(term) ||
        r.patterns.some((p) => p.toLowerCase().includes(term)),
    );
  });

  function labelOf(r: RuleWithCount): string {
    return r.display_name ?? r.patterns[0] ?? "";
  }

  /** Editing through `updateRule` keeps one write path for a rule: the command
   *  re-applies the rules afterwards, which a direct column update would skip. */
  async function assign(r: RuleWithCount) {
    if (!dayIsValid) return;
    busyId = r.id;
    error = null;
    try {
      await updateRule(r.id, {
        patterns: r.patterns,
        category_id: r.category_id,
        priority: r.priority,
        due_day: effectiveDay,
        display_name: r.display_name,
        pay_lead_months: r.pay_lead_months,
      });
      onChanged();
      onClose();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      busyId = null;
    }
  }
</script>

<div class="fixed inset-0 z-40 bg-black/40" transition:scrim onclick={onClose} aria-hidden="true"></div>

<div
  class="fixed left-1/2 top-1/2 z-50 w-[420px] max-h-[70vh] -translate-x-1/2 -translate-y-1/2
         card flex flex-col overflow-hidden"
  transition:dialog
  role="dialog"
  aria-modal="true"
  aria-label={day == null ? t("rule_due_picker.title_any") : t("rule_due_picker.title", { day })}
>
  <header class="px-4 pt-3.5 pb-3 border-b border-border-subtle flex items-start justify-between gap-3">
    <div class="flex flex-col gap-0.5 min-w-0">
      <h2 class="text-title3 font-semibold text-fg">
        {day == null ? t("rule_due_picker.title_any") : t("rule_due_picker.title", { day })}
      </h2>
      <p class="text-foot text-fg-subtle leading-relaxed">
        {dayIsValid
          ? t("rule_due_picker.desc", { day: effectiveDay! })
          : t("rule_due_picker.desc_any")}
      </p>
    </div>
    <button
      bind:this={closeEl}
      type="button"
      onclick={onClose}
      class="press text-fg-muted hover:text-fg shrink-0 text-callout"
      aria-label={t("common.close")}
    >
      ✕
    </button>
  </header>

  <div class="px-4 py-2 border-b border-border-subtle flex items-center gap-2">
    {#if day == null}
      <label class="flex items-center gap-1.5 shrink-0">
        <span class="text-foot text-fg-subtle">{t("rule_due_picker.day_label")}</span>
        <input class="field w-[56px] tabular" type="number" min="1" max="31" bind:value={typedDay} />
      </label>
    {/if}
    <input class="field flex-1" bind:value={q} placeholder={t("rule_due_picker.search")} />
  </div>

  <div class="overflow-y-auto">
    {#if rules === null}
      <div class="py-8 grid place-items-center"><Spinner size={16} /></div>
    {:else if shown.length === 0}
      <EmptyState icon="wandSparkles" title={t("rule_due_picker.empty")} compact />
    {:else}
      <ul class="divide-y divide-border-subtle">
        {#each shown as r (r.id)}
          <li class="px-4 py-2 flex items-center gap-3">
            <span class="text-sub text-fg truncate flex-1">{labelOf(r)}</span>
            {#if r.due_day != null}
              <span class="text-foot text-fg-subtle shrink-0">
                {t("rule_due_picker.currently", { day: r.due_day })}
              </span>
            {/if}
            <Button
              size="sm"
              variant="outline"
              onclick={() => assign(r)}
              disabled={busyId !== null || !dayIsValid}
            >
              {r.due_day === effectiveDay
                ? t("rule_due_picker.already", { day: effectiveDay! })
                : t("rule_due_picker.assign", { day: effectiveDay! })}
            </Button>
          </li>
        {/each}
      </ul>
    {/if}
  </div>

  {#if error}
    <div class="px-4 py-2 border-t border-border-subtle text-foot text-neg">{error}</div>
  {/if}
</div>
