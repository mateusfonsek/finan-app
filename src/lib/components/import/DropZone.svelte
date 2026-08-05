<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import Icon from "$lib/components/ui/Icon.svelte";
  import Spinner from "$lib/components/ui/Spinner.svelte";
  import { decodeOfxFile } from "$lib/ofx/normalize";
  import { parseOfx } from "$lib/ofx/parse";
  import { readFileBytes } from "$lib/api/files";
  import { locale } from "$lib/i18n/locale.svelte";
  import type { ParsedOfx } from "$lib/ofx/types";
  import { push } from "svelte-spa-router";
  import { watch } from "$lib/stores/watch.svelte";
  import { getAppSetting, setAppSetting, WATCH_HINT_DISMISSED_KEY } from "$lib/api/watch";

  const t = locale.t;

  type Props = {
    onparsed?: (result: { file: File; parsed: ParsedOfx }) => void;
    onerror?: (message: string) => void;
  };

  let { onparsed, onerror }: Props = $props();

  let active = $state(false);
  let busy = $state(false);
  let fileInput: HTMLInputElement | undefined = $state();

  let hintDismissed = $state(true); // pessimista até carregar, evita piscar

  onMount(() => {
    void getAppSetting(WATCH_HINT_DISMISSED_KEY).then((v) => (hintDismissed = v === "1"));
  });

  async function dismissHint() {
    hintDismissed = true;
    try {
      await setAppSetting(WATCH_HINT_DISMISSED_KEY, "1");
    } catch {
      // Este componente não tem superfície de erro (não é um fluxo de import
      // que já mostra `error`) — dispensar a isca é cosmético, não vale um
      // toast. Mas se a escrita falhar silenciosamente, `hintDismissed` local
      // ficaria mentindo: a isca reaparece no próximo boot (nunca foi salva)
      // enquanto o usuário acha que já dispensou. Desfazer o otimismo local
      // mantém a UI honesta com o disco, sem inventar uma superfície de erro
      // nova só pra isso.
      hintDismissed = false;
    }
  }

  // Drag-and-drop nativo: o Tauri intercepta o drop do Finder e entrega só o
  // caminho (não um File), via evento da webview. Lemos os bytes e reusamos o
  // mesmo pipeline do file picker construindo um File a partir deles.
  async function handlePath(path: string) {
    try {
      const bytes = await readFileBytes(path);
      const name = path.split(/[\\/]/).pop() || "extrato.ofx";
      await handleFile(new File([bytes as BlobPart], name));
    } catch (e) {
      onerror?.(e instanceof Error ? e.message : String(e));
    }
  }

  onMount(() => {
    let unlisten: (() => void) | undefined;
    getCurrentWebview()
      .onDragDropEvent((event) => {
        const p = event.payload;
        if (p.type === "enter" || p.type === "over") {
          active = true;
        } else if (p.type === "leave") {
          active = false;
        } else if (p.type === "drop") {
          active = false;
          const ofx = p.paths.find((path) => path.toLowerCase().endsWith(".ofx"));
          if (ofx) void handlePath(ofx);
          else if (p.paths.length > 0) onerror?.(t("import.drop_ofx_error"));
        }
      })
      .then((un) => (unlisten = un));
    return () => unlisten?.();
  });

  async function handleFile(file: File) {
    busy = true;
    try {
      const content = await decodeOfxFile(file);
      const parsed = parseOfx(content);
      onparsed?.({ file, parsed });
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      onerror?.(msg);
    } finally {
      busy = false;
    }
  }

  function onDragOver(e: DragEvent) {
    e.preventDefault();
    active = true;
  }

  function onDragLeave(e: DragEvent) {
    e.preventDefault();
    active = false;
  }

  async function onDrop(e: DragEvent) {
    e.preventDefault();
    active = false;
    const file = e.dataTransfer?.files?.[0];
    if (file) await handleFile(file);
  }

  async function onFilePicked(e: Event) {
    const input = e.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    if (file) await handleFile(file);
  }
</script>

<!-- A zona reage ao arraste ANTES do drop: a moldura acende, o ícone sobe e a
     superfície ganha o acento. O alvo confirma que aceitou antes de soltar. -->
<div
  role="button"
  tabindex="0"
  ondragover={onDragOver}
  ondragleave={onDragLeave}
  ondrop={onDrop}
  onclick={() => fileInput?.click()}
  onkeydown={(e) => (e.key === "Enter" || e.key === " ") && fileInput?.click()}
  class="group rounded-[var(--radius-2xl)] border border-dashed p-10 flex flex-col items-center gap-3 text-center
         transition-[background-color,border-color,transform] duration-[var(--dur)] ease-[var(--ease-snap)]
         active:scale-[0.995]
         {active
    ? 'border-accent bg-accent-soft'
    : 'border-border bg-surface hover:bg-surface-2 hover:border-fg-faint'}"
>
  <div
    class="w-14 h-14 rounded-[var(--radius-xl)] grid place-items-center border
           transition-[transform,color,background-color,border-color] duration-[var(--dur)] ease-[var(--ease-snap)]
           {active
      ? 'bg-accent text-accent-on border-transparent -translate-y-1 scale-105'
      : 'bg-surface-2 border-border text-accent group-hover:-translate-y-0.5'}"
  >
    <Icon name="download" size={26} stroke={1.6} />
  </div>

  <h3 class="text-title3 font-semibold text-fg">
    {t("import.dropzone_title")}
  </h3>
  <p class="text-sub text-fg-muted max-w-sm leading-relaxed">
    {t("import.dropzone_desc_1")}
    <strong class="text-fg font-semibold">.ofx</strong>
    {t("import.dropzone_desc_2")}
  </p>

  {#if busy}
    <p class="text-sub text-fg-faint mt-1 flex items-center gap-2">
      <Spinner size={13} />
      {t("import.reading")}
    </p>
  {/if}

  <input
    bind:this={fileInput}
    type="file"
    accept=".ofx,.OFX,application/x-ofx,text/plain"
    onchange={onFilePicked}
    class="hidden"
  />
</div>

{#if !watch.enabled && !hintDismissed}
  <!-- Fora da dropzone: o elemento acima tem onclick pro file picker, então a
       isca precisa ser irmã, não filha, senão o clique nela abriria o picker. -->
  <div class="flex items-center justify-center gap-1 mt-3 text-sub text-fg-subtle">
    <button
      type="button"
      onclick={() => push("/settings")}
      class="inline-flex items-center gap-1.5 px-2 py-1 rounded-[var(--radius-sm)] text-left
             hover:text-fg hover:bg-hover transition-colors duration-[var(--dur-fast)]"
    >
      <Icon name="zap" size={12} class="shrink-0" />
      <span>{t("watch.hint_dropzone")}</span>
      <Icon name="chevronRight" size={11} stroke={2} class="shrink-0" />
    </button>
    <button
      type="button"
      onclick={dismissHint}
      title={t("watch.hint_dismiss")}
      aria-label={t("watch.hint_dismiss")}
      class="press w-5 h-5 grid place-items-center rounded-full hover:text-fg hover:bg-hover
             transition-colors duration-[var(--dur-fast)]"
    >
      <Icon name="x" size={10} stroke={2.4} />
    </button>
  </div>
{/if}
