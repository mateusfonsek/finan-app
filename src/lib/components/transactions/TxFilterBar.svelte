<script lang="ts">
  import MonthStepper from "$lib/components/shell/MonthStepper.svelte";
  import SearchBox from "$lib/components/shell/SearchBox.svelte";
  import Icon from "$lib/components/ui/Icon.svelte";
  import { locale } from "$lib/i18n/locale.svelte";
  import type { Category } from "$lib/bindings";

  const t = locale.t;

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
  let filtering = $derived(categoryId !== null || q !== "");
</script>

<div class="flex items-center gap-2 flex-wrap">
  <MonthStepper {month} onChange={onMonthChange} />

  <div class="relative inline-flex items-center">
    <select
      value={categoryId === null ? "" : String(categoryId)}
      onchange={(e) => {
        const v = (e.currentTarget as HTMLSelectElement).value;
        onCategoryChange(v === "" ? null : Number(v));
      }}
      aria-label={t("tx_table.category")}
      class="field h-7 pl-7 shadow-[var(--shadow-raised)] {categoryId !== null
        ? 'border-accent text-fg'
        : ''}"
    >
      <option value="">{t("tx_filter.all_categories")}</option>
      {#each categories as c}
        <option value={String(c.id)}>{c.name}</option>
      {/each}
    </select>
    <Icon
      name="funnel"
      size={12}
      class="absolute left-2.5 pointer-events-none {categoryId !== null
        ? 'text-accent'
        : 'text-fg-faint'}"
    />
  </div>

  {#if currentCategory}
    <span class="chip text-fg-muted">
      <span
        class="w-2 h-2 rounded-full"
        style="background: var({currentCategory.color_token ?? '--color-cat-outros'})"
      ></span>
      {t("kind." + currentCategory.kind)}
    </span>
  {/if}

  <!-- Clear-all only exists when there is something to clear. -->
  {#if filtering}
    <button
      type="button"
      onclick={() => {
        onCategoryChange(null);
        onQueryChange("");
      }}
      class="text-foot text-fg-subtle hover:text-fg transition-colors duration-[var(--dur-fast)] px-1"
    >
      {t("tx_filter.clear_all")}
    </button>
  {/if}

  <div class="ml-auto">
    <SearchBox value={q} onInput={onQueryChange} bind:ref={searchInputRef} />
  </div>
</div>
