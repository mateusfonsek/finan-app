<script lang="ts">
  import Icon from "$lib/components/ui/Icon.svelte";
  import type { IconName } from "$lib/components/ui/icons";
  import type { SortDir } from "$lib/stores/sort.svelte";

  type Props = {
    label: string;
    /** Esta coluna é a que está ordenando agora? */
    active: boolean;
    /** Sentido atual — só desenhado quando `active`. */
    dir: SortDir;
    /** O que o próximo clique vai fazer, em palavras. Vira o `title`. */
    hint?: string;
    align?: "left" | "right";
    /** Recuo menor, pras colunas estreitas de número. A célula do corpo tem que
     *  usar o MESMO recuo, senão o rótulo e os números saem do prumo. */
    dense?: boolean;
    /**
     * Desenha um símbolo no lugar do rótulo, pra coluna estreita cujo nome não
     * caberia. O `label` continua obrigatório: ele vira o nome acessível e a
     * primeira linha do `title`, porque um símbolo sozinho não se explica.
     */
    symbol?: IconName;
    onclick: () => void;
  };

  let {
    label,
    active,
    dir,
    hint,
    align = "left",
    dense = false,
    symbol,
    onclick,
  }: Props = $props();

  /** Com símbolo, o `title` abre com o nome da coluna — é a única forma de
   *  descobrir o que ela é. */
  let tip = $derived(symbol && hint ? `${label} — ${hint}` : (hint ?? label));

  let icon = $derived<IconName>(
    !active ? "chevronsUpDown" : dir === "asc" ? "chevronUp" : "chevronDown",
  );
</script>

<!--
  Cabeçalho de coluna clicável.

  O indicador some quando a coluna não está ativa, mas o espaço dele fica
  reservado — assim nada empurra o texto ao aparecer. Numa coluna alinhada à
  direita o ícone vem ANTES do rótulo (`flex-row-reverse`), pra que o rótulo
  continue no prumo da borda direita dos números.
-->
<button
  type="button"
  {onclick}
  title={tip}
  aria-label={symbol ? label : undefined}
  class="col-head press-sm group flex w-full items-center gap-1 py-2 select-none
         {dense ? 'px-3' : 'px-4'}
         rounded-[5px] transition-colors duration-[var(--dur-fast)] ease-[var(--ease-snap)]
         hover:text-fg {active ? 'text-fg' : ''}
         {align === 'right' ? 'flex-row-reverse justify-start text-right' : 'text-left'}"
>
  {#if symbol}
    <!-- Traço um pouco mais firme: o símbolo carrega o significado sozinho. -->
    <Icon name={symbol} size={13.5} stroke={1.9} />
  {:else}
    <span class="truncate">{label}</span>
  {/if}
  <Icon
    name={icon}
    size={12}
    stroke={2.2}
    class="transition-opacity duration-[var(--dur-fast)] ease-[var(--ease-snap)]
           {active ? 'opacity-100' : 'opacity-0 group-hover:opacity-45 group-focus-visible:opacity-45'}"
  />
</button>
