<script lang="ts">
  import DropZone from "$lib/components/import/DropZone.svelte";
  import type { ParsedOfx } from "$lib/ofx/types";
  import { push } from "svelte-spa-router";

  let error = $state<string | null>(null);

  function onparsed(detail: { file: File; parsed: ParsedOfx }) {
    // Hand off to /import via window. Svelte stores would be cleaner;
    // acceptable for MVP, refactor in fase 5.
    (window as unknown as { __finanPending?: typeof detail }).__finanPending = detail;
    push("/import");
  }

  function onerror(msg: string) {
    error = msg;
  }
</script>

<section class="p-10 max-w-xl mx-auto flex flex-col gap-6">
  <header class="text-center flex flex-col gap-2">
    <h1 class="text-3xl font-semibold tracking-tight" style="font-family: var(--font-display)">
      Suas finanças, no seu Mac.
    </h1>
    <p class="text-fg-muted text-sm max-w-md mx-auto leading-relaxed">
      Sem nuvem, sem login, sem assinatura. Você arrasta o extrato OFX
      do seu banco e o finan organiza tudo num arquivo SQLite local.
    </p>
  </header>

  <DropZone {onparsed} {onerror} />

  {#if error}
    <div class="rounded-lg border border-border bg-surface p-3 text-sm text-neg">
      Erro ao ler o arquivo: {error}
    </div>
  {/if}

  <p class="text-fg-faint text-xs text-center">
    Seus dados ficam em <span class="font-mono text-fg-muted">~/Library/Application Support/app.finan/finan.db</span>
  </p>
</section>
