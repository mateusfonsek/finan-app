<script module lang="ts">
  /** A paleta das categorias. Vive aqui porque duas superfícies escolhem cor —
   *  o formulário de criar e o painel de editar — e uma lista duplicada
   *  divergiria na primeira cor nova. */
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

  /** Cor de quem não escolheu nada ainda. */
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

<!-- O selecionado ganha um anel externo E um visto por dentro: a marca não
     depende só da cor pra ser vista. -->
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
