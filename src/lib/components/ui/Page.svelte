<script lang="ts">
  import type { Snippet } from "svelte";

  type Props = {
    title: string;
    /** One line of context under the title. Short, no jargon. */
    subtitle?: string;
    /** Screen controls, right of the title. */
    toolbar?: Snippet;
    /** Content column width. The header always uses the SAME one. */
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

<!-- Fixed chrome: content passes UNDERNEATH it (translucent material) rather
     than being cut off by an opaque strip, and the rule only appears when
     something is passing below. The top padding reserves the traffic-light
     strip — with `titleBarStyle: Overlay` the window has no bar of its own. -->
<header
  data-tauri-drag-region="deep"
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
