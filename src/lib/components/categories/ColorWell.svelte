<script module lang="ts">
  /** The category palette. Lives here because two surfaces pick colours — the
   *  create form and the edit panel — and a duplicated list would diverge on the
   *  first new colour. */
  export const COLOR_TOKENS = [
    "--color-cat-mercado",
    "--color-cat-transporte",
    "--color-cat-restaurante",
    "--color-cat-casa",
    "--color-cat-saude",
    "--color-cat-lazer",
    "--color-cat-assinatura",
    "--color-cat-renda",
    "--color-cat-outros",
    "--color-cat-amarelo",
    "--color-cat-indigo",
    "--color-cat-marrom",
    "--color-cat-investimento",
  ];

  /** Colour for anything not yet chosen. */
  export const DEFAULT_COLOR = "--color-cat-outros";
</script>

<script lang="ts">
  import Icon from "$lib/components/ui/Icon.svelte";
  import { locale } from "$lib/i18n/locale.svelte";

  const t = locale.t;

  type Props = {
    value: string;
    onChange: (token: string) => void;
  };

  let { value, onChange }: Props = $props();
</script>

<!-- The selected one gets an outer ring AND an inner check: the mark does not
     rely on colour alone to be seen. -->
<div class="flex gap-2 flex-wrap" role="radiogroup" aria-label={t("categories.color")}>
  {#each COLOR_TOKENS as token (token)}
    {@const selected = value === token}
    <button
      type="button"
      role="radio"
      aria-checked={selected}
      onclick={() => onChange(token)}
      aria-label={token.replace("--color-cat-", "")}
      class="w-6 h-6 rounded-full grid place-items-center text-white
             transition-transform duration-[var(--dur-fast)] ease-[var(--ease-snap)]
             hover:scale-110 active:scale-95
             {selected ? 'ring-2 ring-accent ring-offset-2 ring-offset-surface' : ''}"
      style="background: var({token})"
    >
      {#if selected}
        <Icon name="check" size={12} stroke={3} class="drop-shadow-[0_1px_2px_rgba(0,0,0,.45)]" />
      {/if}
    </button>
  {/each}
</div>
