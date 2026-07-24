<script lang="ts">
  import { onMount, tick } from "svelte";
  import { locale } from "$lib/i18n/locale.svelte";
  import type { Category } from "$lib/bindings";

  const t = locale.t;

  type Props = {
    categories: Category[];
    currentId: number | null;
    onselect: (categoryId: number | null) => void | Promise<void>;
    /** When user wants to create a new category from the typed text. */
    oncreate: (name: string) => Promise<Category>;
  };

  let { categories, currentId, onselect, oncreate }: Props = $props();

  let open = $state(false);
  let dropUp = $state(false);
  let query = $state("");
  let highlighted = $state(0);
  let triggerEl: HTMLButtonElement | undefined = $state();
  let inputEl: HTMLInputElement | undefined = $state();

  let current = $derived(categories.find((c) => c.id === currentId));

  let filtered = $derived(
    categories.filter((c) => c.name.toLowerCase().includes(query.toLowerCase())),
  );

  type Option =
    | { kind: "clear" }
    | { kind: "category"; category: Category }
    | { kind: "create"; name: string };

  let options = $derived<Option[]>(buildOptions(query, filtered, currentId));

  function buildOptions(q: string, list: Category[], cur: number | null): Option[] {
    const out: Option[] = [];
    if (cur !== null) out.push({ kind: "clear" });
    for (const c of list) out.push({ kind: "category", category: c });
    const trimmed = q.trim();
    if (trimmed && !list.some((c) => c.name.toLowerCase() === trimmed.toLowerCase())) {
      out.push({ kind: "create", name: trimmed });
    }
    return out;
  }

  async function openPicker() {
    // Decide direção antes de abrir: se não cabe abaixo do trigger mas cabe
    // acima, abre pra cima. Evita o menu sumir fora da tela nas últimas linhas.
    if (triggerEl) {
      const rect = triggerEl.getBoundingClientRect();
      const menuH = Math.min(options.length * 30, 240) + 48; // ul + input
      const spaceBelow = window.innerHeight - rect.bottom;
      dropUp = spaceBelow < menuH && rect.top > spaceBelow;
    }
    open = true;
    query = "";
    highlighted = 0;
    await tick();
    inputEl?.focus();
  }

  function closePicker() {
    open = false;
    triggerEl?.focus();
  }

  async function choose(opt: Option) {
    if (opt.kind === "clear") {
      await onselect(null);
    } else if (opt.kind === "category") {
      await onselect(opt.category.id);
    } else {
      const created = await oncreate(opt.name);
      await onselect(created.id);
    }
    closePicker();
  }

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      closePicker();
      return;
    }
    if (e.key === "ArrowDown") {
      e.preventDefault();
      highlighted = Math.min(highlighted + 1, options.length - 1);
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      highlighted = Math.max(highlighted - 1, 0);
    } else if (e.key === "Enter") {
      e.preventDefault();
      const opt = options[highlighted];
      if (opt) void choose(opt);
    }
  }

  function colorStyle(token: string | null | undefined): string {
    return token ? `background: var(${token})` : "background: var(--color-cat-outros)";
  }

  onMount(() => {
    function clickOutside(e: MouseEvent) {
      if (!open) return;
      const target = e.target as Node | null;
      if (
        target &&
        !triggerEl?.contains(target) &&
        !inputEl?.parentElement?.parentElement?.contains(target)
      ) {
        closePicker();
      }
    }
    document.addEventListener("mousedown", clickOutside);
    return () => document.removeEventListener("mousedown", clickOutside);
  });
</script>

<div class="relative inline-block">
  <button
    bind:this={triggerEl}
    type="button"
    onclick={openPicker}
    class="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full border border-border bg-surface-2 hover:bg-surface-3 text-[11px] font-medium text-fg-muted"
  >
    {#if current}
      <span class="w-2 h-2 rounded-full" style={colorStyle(current.color_token)}></span>
      <span class="text-fg">{current.name}</span>
    {:else}
      <span class="text-fg-faint">{t("category_picker.no_category")}</span>
    {/if}
  </button>

  {#if open}
    <div
      class="absolute z-30 w-56 rounded-lg border border-border bg-surface overflow-hidden {dropUp ? 'bottom-full mb-1' : 'top-full mt-1'}"
      style="box-shadow: 0 12px 32px -8px rgba(0,0,0,.55), 0 0 0 1px var(--color-border)"
    >
      <div class="border-b border-border-subtle p-1.5">
        <input
          bind:this={inputEl}
          bind:value={query}
          {onkeydown}
          placeholder={t("category_picker.search_or_create")}
          class="w-full bg-transparent border-0 outline-none text-[12px] px-1.5 py-1"
        />
      </div>
      <ul class="max-h-60 overflow-y-auto py-1 text-[12px]">
        {#each options as opt, i}
          <li>
            <!-- svelte-ignore a11y_interactive_supports_focus -->
            <button
              type="button"
              tabindex="-1"
              onmouseenter={() => (highlighted = i)}
              onclick={() => choose(opt)}
              class="w-full flex items-center gap-2 px-2.5 py-1.5 text-left
                     {i === highlighted ? 'bg-accent-soft text-fg' : 'text-fg-muted hover:bg-hover'}"
            >
              {#if opt.kind === "clear"}
                <span class="w-2 h-2 rounded-full bg-transparent border border-border"></span>
                <span class="italic">{t("category_picker.remove_category")}</span>
              {:else if opt.kind === "category"}
                <span class="w-2 h-2 rounded-full" style={colorStyle(opt.category.color_token)}></span>
                <span>{opt.category.name}</span>
                <span class="ml-auto text-[10px] text-fg-faint">{t("kind." + opt.category.kind)}</span>
              {:else}
                <span class="w-2 h-2 rounded-full bg-accent"></span>
                <span>{t("category_picker.create_prefix")} <strong class="text-fg">"{opt.name}"</strong></span>
              {/if}
            </button>
          </li>
        {:else}
          <li class="px-2.5 py-2 text-fg-faint italic">{t("category_picker.empty")}</li>
        {/each}
      </ul>
    </div>
  {/if}
</div>
