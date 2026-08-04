<script lang="ts">
  import { push, router } from "svelte-spa-router";
  import { locale } from "$lib/i18n/locale.svelte";
  import { watch } from "$lib/stores/watch.svelte";
  import {
    autoCollapse,
    autoCollapseArmed,
    collapse,
    expand,
    initialToastState,
    phaseOf,
    syncHash,
  } from "./toastState";

  const t = locale.t;

  /** Tempo até encolher sozinha. Ela NÃO some depois disso — vira pastilha. */
  const AUTO_COLLAPSE_MS = 6000;

  // Não chamar de `state`: `$state` seria lido como auto-subscribe da store
  // `state` (a sintaxe `$` do Svelte), e o compilador reclama de uso antes
  // da declaração.
  let toast = $state(initialToastState);
  let hovering = $state(false);
  let timer: ReturnType<typeof setTimeout> | undefined;

  let current = $derived(watch.discoveries[0] ?? null);

  // Alinhar com a descoberta em foco é efeito, não derivação: `syncHash`
  // preserva o estado quando o hash não mudou, então isto é no-op na maioria
  // dos renders e só reseta quando chega outro arquivo.
  $effect(() => {
    toast = syncHash(toast, current?.hash ?? null);
  });

  let phase = $derived(phaseOf(toast, watch.suppressToast));

  $effect(() => {
    clearTimeout(timer);
    if (!autoCollapseArmed(toast, watch.suppressToast, hovering)) return;
    timer = setTimeout(() => (toast = autoCollapse(toast)), AUTO_COLLAPSE_MS);
    return () => clearTimeout(timer);
  });

  const IMPORT_ROUTE = "/import";

  function review() {
    if (!current) return;
    // Quem carrega o arquivo é a tela de Importar, via sinal na store: daqui
    // não dá pra navegar pra `/import` estando já em `/import` (o `push` não
    // dispara `hashchange`, o Import não remonta, e o clique viraria nada).
    watch.requestOpen(current);
    if (router.location !== IMPORT_ROUTE) void push(IMPORT_ROUTE);
    // Sem esconder nada aqui: resolver a descoberta é o que a tira da store, e
    // é a store que decide o que esta notificação mostra. Se a abertura falhar
    // lá no Import, ela continua pendente — que é o correto.
  }

  async function ignore() {
    if (!current) return;
    await watch.resolve(current.hash, "ignored");
  }
</script>

{#if current && phase !== "hidden"}
  <div
    class="fixed bottom-5 right-5 z-50"
    onmouseenter={() => (hovering = true)}
    onmouseleave={() => (hovering = false)}
    role="presentation"
  >
    {#if phase === "expanded"}
      {@const meta = {
        n: current.txCount,
        from: current.earliest ?? "?",
        to: current.latest ?? "?",
      }}
      <div
        class="w-[320px] rounded-xl border border-border bg-surface shadow-xl p-4 flex flex-col gap-2"
        role="status"
      >
        <div class="flex items-start gap-2">
          <span class="text-[12.5px] font-semibold text-fg flex items-center gap-2 flex-1">
            <span>📄</span>
            <span>
              {watch.pendingCount > 1
                ? t("watch.toast_title_many", { n: watch.pendingCount })
                : t("watch.toast_title")}
            </span>
          </span>
          <!-- Encolher na hora, sem esperar o timer. -->
          <button
            type="button"
            onclick={() => (toast = collapse(toast))}
            title={t("watch.toast_collapse")}
            aria-label={t("watch.toast_collapse")}
            class="text-fg-faint hover:text-fg transition-colors leading-none px-1 -mt-0.5 text-[15px]"
          >
            −
          </button>
        </div>
        <div class="text-[12px] text-fg truncate" title={current.fileName}>{current.fileName}</div>
        <div class="text-[11px] text-fg-faint tabular">
          {current.txCount === 1
            ? t("watch.toast_meta_one", meta)
            : t("watch.toast_meta_many", meta)}
        </div>
        <div class="flex items-center justify-end gap-3 pt-1">
          <button
            type="button"
            onclick={ignore}
            class="text-[11.5px] text-fg-muted hover:text-fg transition-colors"
          >
            {t("watch.toast_ignore")}
          </button>
          <button
            type="button"
            onclick={review}
            class="text-[12px] font-medium px-3 py-1.5 rounded-md bg-accent-soft text-fg border border-accent/30 hover:bg-hover transition-colors"
          >
            {t("watch.toast_review")}
          </button>
        </div>
      </div>
    {:else}
      {@const pending =
        watch.pendingCount === 1
          ? t("watch.badge_pending_one", { n: watch.pendingCount })
          : t("watch.badge_pending_many", { n: watch.pendingCount })}
      <button
        type="button"
        onclick={() => (toast = expand(toast))}
        title={t("watch.toast_expand")}
        class="flex items-center gap-2 rounded-full border border-border bg-surface shadow-lg
               pl-3 pr-3.5 py-2 text-[11.5px] text-fg-muted hover:text-fg hover:bg-hover
               transition-colors"
      >
        <span>📄</span>
        <span class="tabular">{pending}</span>
      </button>
    {/if}
  </div>
{/if}
