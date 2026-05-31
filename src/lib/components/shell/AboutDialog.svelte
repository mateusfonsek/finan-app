<script lang="ts">
  import { onMount } from "svelte";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import logoUrl from "$lib/assets/logo.png";

  type Props = { onClose: () => void };
  let { onClose }: Props = $props();

  const GITHUB_URL = "https://github.com/MateusFonseK/finan-app";

  let closeEl: HTMLButtonElement | undefined = $state();

  const specs = [
    { label: "Sem internet", value: "Funciona sem precisar de conexão" },
    { label: "Sem conta", value: "Nada de cadastro nem senha" },
    { label: "Sem custo", value: "Grátis e sem anúncios" },
    { label: "Sem pegadinha", value: "Você vê tudo que ele faz" },
  ];

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      onClose();
    }
  }

  async function openGithub() {
    await openUrl(GITHUB_URL);
  }

  onMount(() => {
    closeEl?.focus();
  });
</script>

<svelte:window {onkeydown} />

<!-- backdrop -->
<button
  type="button"
  aria-label="Fechar"
  onclick={onClose}
  class="fixed inset-0 z-40 bg-black/50"
  style="backdrop-filter: blur(2px)"
></button>

<!-- dialog -->
<div
  role="dialog"
  aria-modal="true"
  aria-label="Sobre o finan app"
  class="fixed left-1/2 top-1/2 z-50 w-[min(400px,calc(100vw-2rem))] max-h-[calc(100vh-3rem)] -translate-x-1/2 -translate-y-1/2 overflow-y-auto rounded-2xl border border-border bg-surface"
  style="box-shadow: 0 24px 64px -16px rgba(0,0,0,.65), 0 0 0 1px var(--color-border)"
>
  <button
    bind:this={closeEl}
    type="button"
    onclick={onClose}
    aria-label="Fechar"
    class="absolute right-4 top-4 z-10 text-fg-faint hover:text-fg transition-colors"
  >
    ✕
  </button>

  <div class="flex flex-col py-10">
    <!-- hero -->
    <div class="px-10 flex flex-col items-center text-center gap-4 pb-9">
      <img
        src={logoUrl}
        alt=""
        draggable="false"
        class="w-14 h-14 rounded-2xl"
        style="box-shadow: 0 4px 16px -6px rgba(0,0,0,.45)"
      />
      <div class="flex flex-col items-center gap-1.5">
        <h2 class="text-[22px] font-semibold tracking-tight leading-none" style="font-family: var(--font-display)">
          finan app
        </h2>
        <p class="text-[12.5px] text-fg-muted">O dinheiro é seu. Os dados também.</p>
      </div>
    </div>

    <div class="h-px bg-border-subtle"></div>

    <!-- promise -->
    <p class="px-10 py-7 text-center text-[12.5px] text-fg-muted leading-relaxed">
      Sem conta, sem nuvem, sem internet. Tudo que você registra fica no seu
      computador e só você vê.
    </p>

    <div class="h-px bg-border-subtle"></div>

    <!-- spec sheet -->
    <dl class="px-10 py-7 grid grid-cols-[auto_1fr] gap-x-7 gap-y-3.5 items-baseline">
      {#each specs as s}
        <dt class="text-[12.5px] font-semibold text-accent whitespace-nowrap">{s.label}</dt>
        <dd class="text-[12px] text-fg-muted">{s.value}</dd>
      {/each}
    </dl>

    <div class="h-px bg-border-subtle"></div>

    <!-- colophon -->
    <div class="px-10 pt-8 flex flex-col gap-4">
      <div class="flex items-baseline justify-between gap-3">
        <span class="text-[13px] font-medium text-fg">Mateus Fonseca</span>
        <span class="font-mono text-[11px] text-fg-faint tabular">v0.1.0</span>
      </div>
      <button
        type="button"
        onclick={openGithub}
        class="self-start text-[12px] text-accent hover:text-accent-hi underline-offset-4 hover:underline transition-colors"
      >
        github.com/MateusFonseK ↗
      </button>
      <p class="text-[10.5px] text-fg-faint leading-relaxed pt-2">
        Serve pra você organizar suas finanças pessoais. Não dá conselhos de
        investimento nem funciona como banco.
      </p>
    </div>
  </div>
</div>
