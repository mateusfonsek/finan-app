<script lang="ts">
  import MonthStepper from "$lib/components/shell/MonthStepper.svelte";
  import SearchBox from "$lib/components/shell/SearchBox.svelte";
  import type { Category } from "$lib/bindings";

  type Props = {
    categories: Category[];
    month: string | null;
    categoryId: number | null;
    q: string;
    onMonthChange: (m: string | null) => void;
    onCategoryChange: (id: number | null) => void;
    onQueryChange: (v: string) => void;
    searchInputRef?: HTMLInputElement | null;
  };

  let {
    categories,
    month,
    categoryId,
    q,
    onMonthChange,
    onCategoryChange,
    onQueryChange,
    searchInputRef = $bindable(null),
  }: Props = $props();

  let currentCategory = $derived(categories.find((c) => c.id === categoryId));
</script>

<div class="flex items-center gap-2 flex-wrap">
  <MonthStepper {month} onChange={onMonthChange} />

  <select
    value={categoryId === null ? "" : String(categoryId)}
    onchange={(e) => {
      const v = (e.currentTarget as HTMLSelectElement).value;
      onCategoryChange(v === "" ? null : Number(v));
    }}
    class="text-[12px] rounded-md border border-border bg-surface-2 px-2 py-1 text-fg"
  >
    <option value="">Todas as categorias</option>
    {#each categories as c}
      <option value={String(c.id)}>{c.name}</option>
    {/each}
  </select>

  {#if currentCategory}
    <span class="text-[11px] text-fg-faint">· {currentCategory.kind}</span>
  {/if}

  <div class="ml-auto">
    <SearchBox value={q} onInput={onQueryChange} bind:ref={searchInputRef} />
  </div>
</div>
