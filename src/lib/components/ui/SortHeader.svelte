<script lang="ts">
  import Icon from "$lib/components/ui/Icon.svelte";
  import type { IconName } from "$lib/components/ui/icons";
  import type { SortDir } from "$lib/stores/sort.svelte";

  type Props = {
    label: string;
    /** Is this the column currently sorting? */
    active: boolean;
    /** Current direction — only drawn when `active`. */
    dir: SortDir;
    /** What the next click will do, in words. Becomes the `title`. */
    hint?: string;
    align?: "left" | "right";
    /** Tighter padding for narrow numeric columns. Body cells must use the
     *  SAME padding or the label and the numbers fall out of alignment. */
    dense?: boolean;
    /**
     * Draws a symbol instead of the label, for a narrow column whose name
     * would not fit. `label` stays required: it becomes the accessible name and
     * the first line of the `title`, because a symbol alone explains nothing.
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

  /** With a symbol, the `title` leads with the column name — the only way to
   *  discover what it is. */
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
    <!-- Slightly heavier stroke: the symbol carries the meaning alone. -->
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
