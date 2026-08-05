<script lang="ts">
  import type { Snippet } from "svelte";

  type Props = {
    title: string;
    /** Uma linha de contexto sob o título. Frase curta, sem jargão. */
    subtitle?: string;
    /** Controles da tela, à direita do título. */
    toolbar?: Snippet;
    /** Largura da coluna de conteúdo. O cabeçalho usa a MESMA, sempre. */
    width?: "narrow" | "regular" | "wide";
    children: Snippet;
  };

  let { title, subtitle, toolbar, width = "regular", children }: Props = $props();

  const WIDTHS = {
    narrow: "max-w-3xl",
    regular: "max-w-5xl",
    wide: "max-w-6xl",
  } as const;

  let col = $derived(WIDTHS[width]);
</script>

<!-- Cromo fixo: o conteúdo passa POR BAIXO dele (material translúcido) em vez
     de ser cortado por uma faixa opaca, e a régua só aparece quando há algo
     passando por baixo. O padding do topo reserva a faixa dos semáforos —
     com `titleBarStyle: Overlay` a janela não tem barra própria. -->
<header
  data-tauri-drag-region
  class="material-chrome scroll-edge sticky top-0 z-20"
  style="padding-top: var(--titlebar-h)"
>
  <div class="mx-auto w-full {col} px-8 pb-4 flex items-end justify-between gap-5 flex-wrap">
    <div class="flex flex-col gap-1 min-w-0">
      <h1 class="text-title1 font-semibold text-fg">{title}</h1>
      {#if subtitle}
        <p class="text-sub text-fg-subtle max-w-[62ch]">{subtitle}</p>
      {/if}
    </div>
    {#if toolbar}
      <div class="flex items-center gap-2 shrink-0">
        {@render toolbar()}
      </div>
    {/if}
  </div>
</header>

<section class="mx-auto w-full {col} px-8 pb-10 pt-1 flex flex-col gap-4">
  {@render children()}
</section>
