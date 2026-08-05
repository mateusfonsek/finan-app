<script lang="ts">
  import type { Snippet } from "svelte";
  import Icon from "./Icon.svelte";
  import type { IconName } from "./icons";

  type Props = {
    /** Um símbolo diz o que a frase já disse, mais rápido. */
    icon?: IconName;
    title: string;
    /** O que fazer a seguir — vazio sem saída é beco sem saída. */
    description?: string;
    action?: Snippet;
    compact?: boolean;
  };

  let { icon, title, description, action, compact = false }: Props = $props();
</script>

<div class="flex flex-col items-center text-center gap-2 {compact ? 'py-8 px-4' : 'py-14 px-6'}">
  {#if icon}
    <div
      class="grid place-items-center rounded-[var(--radius-xl)] bg-surface-2 border border-border-subtle
             text-fg-faint {compact ? 'w-9 h-9 mb-0.5' : 'w-12 h-12 mb-1'}"
    >
      <Icon name={icon} size={compact ? 16 : 20} stroke={1.5} />
    </div>
  {/if}
  <p class="text-callout font-medium text-fg-muted">{title}</p>
  {#if description}
    <p class="text-sub text-fg-subtle max-w-[46ch] leading-relaxed">{description}</p>
  {/if}
  {#if action}
    <div class="mt-2">{@render action()}</div>
  {/if}
</div>
