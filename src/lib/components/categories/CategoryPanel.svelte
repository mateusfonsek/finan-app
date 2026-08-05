<script lang="ts">
  import { tick } from "svelte";
  import Icon from "$lib/components/ui/Icon.svelte";
  import { Button } from "$lib/components/ui/button";
  import ErrorNote from "$lib/components/ui/ErrorNote.svelte";
  import { locale } from "$lib/i18n/locale.svelte";
  import { scrim, sheet } from "$lib/motion";
  import ColorWell, { DEFAULT_COLOR } from "./ColorWell.svelte";
  import type { CategoryWithCount } from "$lib/bindings";

  const t = locale.t;

  type Props = {
    category: CategoryWithCount;
    onClose: () => void;
    onSave: (
      categoryId: number,
      data: { name: string; colorToken: string; kind: string },
    ) => Promise<void>;
  };

  let { category, onClose, onSave }: Props = $props();

  let name = $state("");
  let kind = $state<"expense" | "income" | "transfer">("expense");
  let colorToken = $state(DEFAULT_COLOR);
  let busy = $state(false);
  let error = $state<string | null>(null);
  let nameInput: HTMLInputElement | undefined = $state();

  // Reloads the draft when the panel switches categories (clicking another row
  // while it is open).
  $effect(() => {
    name = category.name;
    kind = (category.kind ?? "expense") as "expense" | "income" | "transfer";
    colorToken = category.color_token ?? DEFAULT_COLOR;
    error = null;
  });

  /** Focus the name as soon as the panel opens: that is where editing starts,
   *  and without it an extra click would be needed. */
  $effect(() => {
    void category.id;
    void (async () => {
      await tick();
      nameInput?.focus();
      nameInput?.select();
    })();
  });

  async function save() {
    error = null;
    const trimmed = name.trim();
    if (trimmed.length === 0) {
      error = t("categories.name_required");
      nameInput?.focus();
      return;
    }
    busy = true;
    try {
      await onSave(category.id, { name: trimmed, colorToken, kind });
      onClose();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      onClose();
    }
    // Cmd+Enter saves — same shortcut as the other edit panels.
    if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      void save();
    }
  }
</script>

<svelte:window {onkeydown} />

<!-- Same scrim + sheet pair as the rule and transaction panels: enters from
     the right, leaves to the right. Light scrim because the task is focused,
     not blocking. -->
<button
  type="button"
  aria-label={t("common.close")}
  onclick={onClose}
  transition:scrim
  class="fixed inset-0 z-50 bg-black/25"
></button>

<div
  transition:sheet={{ side: "right" }}
  class="fixed right-0 top-0 bottom-0 z-60 w-[372px] bg-surface border-l border-border-subtle
         flex flex-col shadow-[var(--shadow-sheet)]"
  role="dialog"
  aria-label={t("category_panel.title")}
>
  <header
    data-tauri-drag-region="deep"
    class="flex items-center justify-between px-4 pb-3 border-b border-border-subtle"
    style="padding-top: max(12px, var(--titlebar-h))"
  >
    <span class="section-title">{t("category_panel.title")}</span>
    <button
      type="button"
      onclick={onClose}
      aria-label={t("common.close")}
      class="press w-6 h-6 grid place-items-center rounded-full text-fg-faint
             hover:text-fg hover:bg-hover transition-colors duration-[var(--dur-fast)]"
    >
      <Icon name="x" size={13} stroke={2} />
    </button>
  </header>

  <div class="flex-1 overflow-y-auto">
    <!-- The panel's subject is the category as it appears everywhere else: the
         colour and name follow what is being typed, so the result is visible
         before saving. -->
    <div class="px-4 pt-4 pb-3 flex flex-col gap-1.5">
      <span class="flex items-center gap-2.5 min-w-0">
        <span
          class="w-3.5 h-3.5 rounded-[4px] shrink-0 transition-colors duration-[var(--dur-fast)]"
          style="background: var({colorToken})"
        ></span>
        <span class="text-title2 font-semibold text-fg truncate">
          {name.trim() || "—"}
        </span>
      </span>
      <span class="text-sub text-fg-subtle tabular">
        {category.transaction_count === 0
          ? t("category_panel.tx_none")
          : category.transaction_count === 1
            ? t("category_panel.tx_one")
            : t("category_panel.tx_many", { n: category.transaction_count })}
      </span>
    </div>

    <div class="hairline"></div>

    <div class="px-4 py-4 flex flex-col gap-3.5">
      <label class="flex flex-col gap-1">
        <span class="text-foot text-fg-subtle">{t("categories.name")}</span>
        <input
          bind:this={nameInput}
          bind:value={name}
          placeholder={t("categories.name_placeholder")}
          class="field"
        />
      </label>

      <label class="flex flex-col gap-1">
        <span class="text-foot text-fg-subtle">{t("categories.type")}</span>
        <select bind:value={kind} class="field">
          <option value="expense">{t("kind.expense")}</option>
          <option value="income">{t("kind.income")}</option>
          <option value="transfer">{t("kind.transfer")}</option>
        </select>
        <!-- A visible hint rather than a tooltip: anyone not hovering also
             needs to understand what the field decides. -->
        <span class="text-cap text-fg-faint leading-snug">{t("category_panel.kind_hint")}</span>
      </label>

      <div class="flex flex-col gap-1.5">
        <span class="text-foot text-fg-subtle">{t("categories.color")}</span>
        <ColorWell value={colorToken} onChange={(token) => (colorToken = token)} />
        <span class="text-cap text-fg-faint leading-snug mt-0.5">
          {t("category_panel.color_hint")}
        </span>
      </div>

      {#if error}
        <ErrorNote message={error} />
      {/if}
    </div>

    <div class="hairline"></div>

    <p class="px-4 py-3 text-cap text-fg-faint leading-snug">
      {t("category_panel.scope_note")}
    </p>
  </div>

  <footer class="px-4 py-3 border-t border-border-subtle flex justify-end gap-2">
    <Button variant="ghost" onclick={onClose} disabled={busy}>{t("common.cancel")}</Button>
    <Button onclick={save} disabled={busy}>
      {busy ? t("categories.saving") : t("categories.save")}
    </Button>
  </footer>
</div>
