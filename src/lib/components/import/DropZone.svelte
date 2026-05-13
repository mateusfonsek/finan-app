<script lang="ts">
  import { decodeOfxFile } from "$lib/ofx/normalize";
  import { parseOfx } from "$lib/ofx/parse";
  import type { ParsedOfx } from "$lib/ofx/types";

  type Props = {
    onparsed?: (result: { file: File; parsed: ParsedOfx }) => void;
    onerror?: (message: string) => void;
  };

  let { onparsed, onerror }: Props = $props();

  let active = $state(false);
  let busy = $state(false);
  let fileInput: HTMLInputElement | undefined = $state();

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
  class="rounded-xl border border-dashed p-9 flex flex-col items-center gap-3 cursor-pointer transition-colors text-center
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
    Arraste seu extrato OFX
  </h3>
  <p class="text-fg-muted text-xs max-w-sm">
    Exporte o extrato mensal do seu banco (Itaú, Nubank, Bradesco, etc.) no formato
    <strong class="text-fg">.ofx</strong> e solte aqui — ou clique pra escolher um arquivo.
  </p>

  {#if busy}
    <p class="text-fg-faint text-xs mt-2">Lendo arquivo…</p>
  {/if}

  <input
    bind:this={fileInput}
    type="file"
    accept=".ofx,.OFX,application/x-ofx,text/plain"
    onchange={onFilePicked}
    class="hidden"
  />
</div>
