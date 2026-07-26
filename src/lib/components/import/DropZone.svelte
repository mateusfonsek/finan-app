<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
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
    await setAppSetting(WATCH_HINT_DISMISSED_KEY, "1");
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

<div
  role="button"
  tabindex="0"
  ondragover={onDragOver}
  ondragleave={onDragLeave}
  ondrop={onDrop}
  onclick={() => fileInput?.click()}
  onkeydown={(e) => (e.key === "Enter" || e.key === " ") && fileInput?.click()}
  class="rounded-xl border border-dashed p-9 flex flex-col items-center gap-3 transition-colors text-center
         {active ? 'border-accent bg-accent-soft' : 'border-border bg-surface hover:bg-surface-2'}"
>
  <div class="w-14 h-14 rounded-2xl grid place-items-center"
       style="background: var(--color-surface-2); border: 1px solid var(--color-border); color: var(--color-accent-hi)">
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7"
         stroke-linecap="round" stroke-linejoin="round" class="w-7 h-7">
      <path d="M12 4v11m0 0 4-4m-4 4-4-4" />
      <path d="M4 17v2.5A1.5 1.5 0 0 0 5.5 21h13a1.5 1.5 0 0 0 1.5-1.5V17" />
    </svg>
  </div>

  <h3 class="text-base font-semibold tracking-tight" style="font-family: var(--font-display)">
    {t("import.dropzone_title")}
  </h3>
  <p class="text-fg-muted text-xs max-w-sm">
    {t("import.dropzone_desc_1")}
    <strong class="text-fg">.ofx</strong> {t("import.dropzone_desc_2")}
  </p>

  {#if busy}
    <p class="text-fg-faint text-xs mt-2">{t("import.reading")}</p>
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
  <div class="flex items-center justify-center gap-2 mt-3 text-[11.5px] text-fg-faint">
    <button
      type="button"
      onclick={() => push("/settings")}
      class="hover:text-fg transition-colors"
    >
      ⚡ {t("watch.hint_dropzone")} →
    </button>
    <button
      type="button"
      onclick={dismissHint}
      title={t("watch.hint_dismiss")}
      class="hover:text-fg transition-colors px-1"
    >
      ×
    </button>
  </div>
{/if}
