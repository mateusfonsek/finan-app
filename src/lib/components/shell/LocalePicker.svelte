<script lang="ts">
  import { locale } from "$lib/i18n/locale.svelte";
  import Icon from "$lib/components/ui/Icon.svelte";

  const locales = locale.list();
</script>

<div class="flex flex-wrap gap-2">
  {#each locales as l}
    {@const active = locale.code === l.code}
    <button
      type="button"
      onclick={() => locale.set(l.code)}
      class="press flex items-center gap-2 h-7 px-3 rounded-[var(--radius-md)] text-callout font-medium
             border transition-colors duration-[var(--dur-fast)]
             {active
        ? 'bg-accent text-accent-on border-transparent'
        : 'text-fg-muted border-border bg-surface-2 hover:bg-hover hover:text-fg'}"
      aria-pressed={active}
    >
      {#if l.flag}<span>{l.flag}</span>{/if}
      <span>{l.name}</span>
      {#if active}<Icon name="check" size={12} stroke={2.6} />{/if}
    </button>
  {/each}
</div>
