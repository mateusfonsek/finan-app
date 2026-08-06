<script lang="ts">
  import Icon from "$lib/components/ui/Icon.svelte";
  import Spinner from "$lib/components/ui/Spinner.svelte";
  import { Button } from "$lib/components/ui/button";
  import { locale } from "$lib/i18n/locale.svelte";
  import { toast as toastMotion } from "$lib/motion";
  import { activity } from "$lib/stores/activity.svelte";

  const t = locale.t;

  let e = $derived(activity.enrich);

  // Fraction for the bar. The guard against a 0 total is not paranoia: with
  // enrichment off the backend emits `Started { total: 0 }`, and without it the
  // transform would get NaN — the bar would vanish with nothing in the console.
  let fraction = $derived(e.total > 0 ? Math.min(1, e.done / e.total) : 0);

  let doneLabel = $derived(
    e.report === null
      ? ""
      : e.report.created_rules.length === 0
        ? t("import.enrich_none")
        : e.report.created_rules.length === 1
          ? t("import.enrich_done_one")
          : t("import.enrich_done", { n: e.report.created_rules.length }),
  );
</script>

<div
  transition:toastMotion
  class="material-pop w-[326px] p-3.5 flex flex-col gap-2 rounded-[var(--radius-xl)]"
  role="status"
>
  <div class="flex items-start gap-2.5">
    <span
      class="w-7 h-7 shrink-0 grid place-items-center rounded-[var(--radius-md)] bg-accent-soft text-accent"
    >
      {#if activity.busy}
        <Spinner size={13} />
      {:else if e.phase === "failed"}
        <Icon name="x" size={13} stroke={2.2} />
      {:else}
        <Icon name="check" size={13} stroke={2.4} />
      {/if}
    </span>

    <span class="flex-1 min-w-0 flex flex-col gap-0.5 pt-px">
      <span class="text-callout font-semibold text-fg">
        {#if activity.busy}
          {t("import.enrich_running")}
        {:else if e.phase === "cancelled"}
          {t("import.enrich_cancelled")}
        {:else if e.phase === "failed"}
          {t("import.enrich_aborted")}
        {:else}
          {doneLabel}
        {/if}
      </span>
      <span class="text-sub text-fg-muted truncate">
        {#if activity.busy}
          <!-- `tabular` keeps the counter from dancing on every increment: with
               proportional widths, "14 de 42" and "15 de 42" have different
               sizes and the company name beside it slides. -->
          <span class="tabular">
            {t("import.enrich_progress", { done: e.done, total: e.total })}
          </span>
          {#if e.label}· {e.label}{/if}
        {:else if e.phase === "failed"}
          {e.error}
        {:else if e.failed > 0}
          {t("import.enrich_failed_some", { n: e.failed })}
        {/if}
      </span>
    </span>

    {#if !activity.busy}
      <button
        type="button"
        onclick={() => activity.clear()}
        title={t("import.enrich_dismiss")}
        aria-label={t("import.enrich_dismiss")}
        class="press w-5 h-5 grid place-items-center rounded-full text-fg-faint
               hover:text-fg hover:bg-hover transition-colors duration-[var(--dur-fast)]"
      >
        <Icon name="x" size={11} stroke={2.4} />
      </button>
    {/if}
  </div>

  {#if activity.busy}
    <!-- Determinate bar: the total is known before the first lookup, so there
         is no invented estimate here. `scaleX` rather than `width` keeps the
         animation on the compositor — the bar gets one event per tax id, and
         `width` would lay out on every one of them. -->
    <div class="h-1 rounded-full bg-surface-2 overflow-hidden">
      <div
        class="h-full origin-left rounded-full bg-accent
               transition-transform duration-[var(--dur)] ease-[var(--ease-snap)]"
        style="transform: scaleX({fraction})"
      ></div>
    </div>

    <div class="flex items-center justify-end pt-0.5">
      <Button variant="ghost" size="sm" onclick={() => activity.cancel()}>
        {t("import.enrich_stop")}
      </Button>
    </div>
  {/if}
</div>
