<script lang="ts">
  import { ICONS, type IconName } from "./icons";

  type Props = {
    name: IconName;
    /** Tamanho em px. Alinhe ao tamanho do texto ao lado, não ao contêiner. */
    size?: number;
    /** Traço mais firme quando o símbolo carrega significado sozinho. */
    stroke?: number;
    class?: string;
    /** Símbolo com significado próprio ganha rótulo; decorativo fica oculto. */
    label?: string;
  };

  let { name, size = 15, stroke = 1.7, class: className = "", label }: Props = $props();

  let node = $derived(ICONS[name]);
</script>

<svg
  xmlns="http://www.w3.org/2000/svg"
  viewBox="0 0 24 24"
  width={size}
  height={size}
  fill="none"
  stroke="currentColor"
  stroke-width={stroke}
  stroke-linecap="round"
  stroke-linejoin="round"
  class="shrink-0 {className}"
  aria-hidden={label ? undefined : "true"}
  role={label ? "img" : undefined}
  aria-label={label}
>
  {#each node as [tag, attrs]}
    <svelte:element this={tag} {...attrs} />
  {/each}
</svg>
