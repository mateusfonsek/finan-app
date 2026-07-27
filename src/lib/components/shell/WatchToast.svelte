<script lang="ts">
  import { push, router } from "svelte-spa-router";
  import { locale } from "$lib/i18n/locale.svelte";
  import { watch } from "$lib/stores/watch.svelte";

  const t = locale.t;

  /** Some sozinho depois de um tempo; o badge da sidebar é que persiste. */
  const AUTO_HIDE_MS = 8000;

  let hiddenFor = $state<string | null>(null);
  let timer: ReturnType<typeof setTimeout> | undefined;

  let current = $derived(watch.discoveries[0] ?? null);
  let visible = $derived(
    !watch.suppressToast && current !== null && hiddenFor !== current.hash,
  );

  $effect(() => {
    if (!visible || !current) return;
    const hash = current.hash;
    clearTimeout(timer);
    timer = setTimeout(() => (hiddenFor = hash), AUTO_HIDE_MS);
    return () => clearTimeout(timer);
  });

  const IMPORT_ROUTE = "/import";

  function review() {
    if (!current) return;
    // Quem carrega o arquivo é a tela de Importar, via sinal na store: daqui
    // não dá pra navegar pra `/import` estando já em `/import` (o `push` não
    // dispara `hashchange`, o Import não remonta, e o clique viraria nada).
    watch.requestOpen(current);
    // Esconder aqui não desalinha nada com o badge: se a abertura falhar lá no
    // Import, a descoberta sai da lista de um jeito ou de outro — `invalid`
    // quando o conteúdo não presta, ou só some desta rodada quando não deu pra
    // ler agora. Toast e badge continuam contando a mesma coisa.
    hiddenFor = current.hash;
    if (router.location !== IMPORT_ROUTE) void push(IMPORT_ROUTE);
  }

  async function ignore() {
    if (!current) return;
    await watch.resolve(current.hash, "ignored");
  }
</script>

{#if visible && current}
  <div
    class="fixed bottom-5 right-5 z-50 w-[320px] rounded-xl border border-border bg-surface shadow-xl p-4 flex flex-col gap-2"
    role="status"
  >
    <div class="text-[12.5px] font-semibold text-fg flex items-center gap-2">
      <span>📄</span>
      <span>
        {watch.pendingCount > 1
          ? t("watch.toast_title_many", { n: watch.pendingCount })
          : t("watch.toast_title")}
      </span>
    </div>
    <div class="text-[12px] text-fg truncate" title={current.fileName}>{current.fileName}</div>
    <div class="text-[11px] text-fg-faint tabular">
      {current.txCount === 1
        ? t("watch.toast_meta_one", {
            n: current.txCount,
            from: current.earliest ?? "?",
            to: current.latest ?? "?",
          })
        : t("watch.toast_meta_many", {
            n: current.txCount,
            from: current.earliest ?? "?",
            to: current.latest ?? "?",
          })}
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
{/if}
