<script lang="ts">
  import Icon from "$lib/components/ui/Icon.svelte";
  import Spinner from "$lib/components/ui/Spinner.svelte";
  import { Button } from "$lib/components/ui/button";
  import { locale } from "$lib/i18n/locale.svelte";
  import { toast as toastMotion } from "$lib/motion";
  import { activity } from "$lib/stores/activity.svelte";

  const t = locale.t;

  let e = $derived(activity.enrich);

  // Fração para a barra. A guarda contra total 0 não é paranoia: com o
  // enriquecimento desligado o backend emite `Started { total: 0 }`, e sem ela
  // o transform receberia NaN — a barra sumiria sem erro nenhum no console.
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
          <!-- `tabular` para o contador não dançar a cada incremento: com
               larguras proporcionais, "14 de 42" e "15 de 42" têm tamanhos
               diferentes e o nome da empresa ao lado escorrega. -->
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
    <!-- Barra determinada: o total é conhecido antes da primeira consulta, então
         não há estimativa inventada aqui. `scaleX` em vez de `width` para a
         animação ficar no compositor — a barra recebe um evento por CNPJ, e
         `width` faria layout a cada um deles. -->
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
