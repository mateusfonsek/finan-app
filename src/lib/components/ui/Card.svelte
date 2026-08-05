<script lang="ts">
  import type { Snippet } from "svelte";

  type Props = {
    /** Título da seção. Frase normal — caixa alta é tique de dashboard. */
    title?: string;
    /** Contexto discreto à direita do título (contagem, unidade, ressalva). */
    note?: string;
    /** Controles à direita do título. Substitui `note` quando presente. */
    actions?: Snippet;
    /** Ponto colorido antes do título, para seções com identidade própria. */
    accent?: string;
    padded?: boolean;
    class?: string;
    children: Snippet;
  };

  let {
    title,
    note,
    actions,
    accent,
    padded = true,
    class: className = "",
    children,
  }: Props = $props();
</script>

<section class="card flex flex-col {padded ? 'p-4 gap-3' : ''} {className}">
  {#if title}
    <header class="flex items-baseline justify-between gap-3 {padded ? '' : 'px-4 pt-4 pb-1'}">
      <h2 class="section-title flex items-center gap-2 min-w-0">
        {#if accent}
          <span class="w-1.5 h-1.5 rounded-full shrink-0" style="background: {accent}"></span>
        {/if}
        <span class="truncate">{title}</span>
      </h2>
      {#if actions}
        {@render actions()}
      {:else if note}
        <span class="section-note shrink-0">{note}</span>
      {/if}
    </header>
  {/if}
  {@render children()}
</section>
