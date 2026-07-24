<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { locale } from "$lib/i18n/locale.svelte";
  import type { Category, Rule } from "$lib/bindings";

  const t = locale.t;

  type Props = {
    categories: Category[];
    initial?: Rule | null;
    onSave: (data: {
      pattern: string;
      categoryId: number;
      priority: number;
      dueDay: number | null;
    }) => Promise<void>;
    onCancel?: () => void;
    submitLabel?: string;
  };

  let { categories, initial = null, onSave, onCancel, submitLabel }: Props = $props();

  let pattern = $state("");
  let categoryId = $state<number | null>(null);
  let priority = $state(0);
  /** Svelte 5 coage <input type="number"> pra number | null; mantemos compatível. */
  let dueDayValue = $state<number | null>(null);
  let busy = $state(false);
  let error = $state<string | null>(null);

  $effect(() => {
    pattern = initial?.pattern ?? "";
    categoryId = initial?.category_id ?? null;
    priority = initial?.priority ?? 0;
    dueDayValue = initial?.due_day ?? null;
  });

  async function submit(e: Event) {
    e.preventDefault();
    error = null;
    if (pattern.trim().length === 0) {
      error = t("rule_form.pattern_required");
      return;
    }
    if (categoryId === null) {
      error = t("rule_form.category_required");
      return;
    }
    let dueDay: number | null = null;
    if (dueDayValue != null) {
      if (!Number.isInteger(dueDayValue) || dueDayValue < 1 || dueDayValue > 31) {
        error = t("rule_form.due_day_invalid");
        return;
      }
      dueDay = dueDayValue;
    }
    busy = true;
    try {
      await onSave({ pattern: pattern.trim(), categoryId, priority, dueDay });
      if (!initial) {
        pattern = "";
        categoryId = null;
        priority = 0;
        dueDayValue = null;
      }
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }
</script>

<form onsubmit={submit} class="rounded-lg border border-border-subtle bg-surface p-4 flex flex-col gap-3">
  <div class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint">
    {initial ? t("rule_form.form_edit") : t("rule_form.form_new")}
  </div>

  <div class="grid grid-cols-[1fr_180px_90px_110px_auto] gap-2 items-end">
    <label class="flex flex-col gap-1">
      <span class="text-[11px] text-fg-muted">{t("rule_form.pattern_label")}</span>
      <input
        bind:value={pattern}
        placeholder={t("rule_form.pattern_placeholder")}
        class="rounded-md border border-border bg-surface-2 px-2 py-1 text-[12px] text-fg focus:outline-none focus:border-accent"
      />
    </label>

    <label class="flex flex-col gap-1">
      <span class="text-[11px] text-fg-muted">{t("rule_form.category")}</span>
      <select
        value={categoryId === null ? "" : String(categoryId)}
        onchange={(e) => {
          const v = (e.currentTarget as HTMLSelectElement).value;
          categoryId = v === "" ? null : Number(v);
        }}
        class="rounded-md border border-border bg-surface-2 px-2 py-1 text-[12px] text-fg"
      >
        <option value="">{t("rule_form.select_placeholder")}</option>
        {#each categories as c}
          <option value={String(c.id)}>{c.name}</option>
        {/each}
      </select>
    </label>

    <label class="flex flex-col gap-1">
      <span class="text-[11px] text-fg-muted">{t("rule_form.priority")}</span>
      <input
        type="number"
        bind:value={priority}
        class="rounded-md border border-border bg-surface-2 px-2 py-1 text-[12px] text-fg tabular focus:outline-none focus:border-accent"
      />
    </label>

    <label class="flex flex-col gap-1">
      <span class="text-[11px] text-fg-muted" title={t("rule_form.due_day_title")}>
        {t("rule_form.due_day")}
      </span>
      <input
        type="number"
        min="1"
        max="31"
        placeholder="—"
        bind:value={dueDayValue}
        class="rounded-md border border-border bg-surface-2 px-2 py-1 text-[12px] text-fg tabular focus:outline-none focus:border-accent"
      />
    </label>

    <div class="flex gap-2">
      {#if onCancel}
        <Button variant="ghost" onclick={onCancel} type="button">{t("common.cancel")}</Button>
      {/if}
      <Button type="submit" disabled={busy}>
        {busy ? t("rule_form.saving") : (submitLabel ?? (initial ? t("rule_form.save") : t("rule_form.add")))}
      </Button>
    </div>
  </div>

  {#if error}
    <div class="text-[11px] text-neg">{error}</div>
  {/if}
</form>
