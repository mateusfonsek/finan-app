<script lang="ts">
  type Props = {
    value: string;
    placeholder?: string;
    onInput: (v: string) => void;
    ref?: HTMLInputElement | null;
  };

  let { value, placeholder = "Buscar…", onInput, ref = $bindable(null) }: Props = $props();
</script>

<div class="inline-flex items-center gap-1.5 px-2 py-1 rounded-md border border-border bg-surface-2 focus-within:border-accent focus-within:bg-bg transition-colors">
  <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" class="text-fg-faint">
    <circle cx="11" cy="11" r="7"/>
    <path d="M21 21l-4.3-4.3"/>
  </svg>
  <input
    bind:this={ref}
    type="text"
    data-search-input
    {value}
    {placeholder}
    oninput={(e) => onInput((e.currentTarget as HTMLInputElement).value)}
    class="bg-transparent border-0 outline-none text-[12px] w-44 text-fg placeholder:text-fg-faint"
  />
  {#if value}
    <button
      type="button"
      onclick={() => onInput("")}
      aria-label="Limpar busca"
      class="text-fg-faint hover:text-fg-muted text-[11px]"
    >
      ✕
    </button>
  {/if}
</div>
