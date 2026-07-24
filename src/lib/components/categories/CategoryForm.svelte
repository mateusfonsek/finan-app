<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { locale } from "$lib/i18n/locale.svelte";
  import type { CategoryWithCount } from "$lib/bindings";

  const t = locale.t;

  type Props = {
    initial?: CategoryWithCount | null;
    onSave: (data: { name: string; colorToken: string; kind: string }) => Promise<void>;
    onCancel?: () => void;
  };

  let { initial = null, onSave, onCancel }: Props = $props();

  const COLOR_TOKENS = [
    "--color-cat-mercado",
    "--color-cat-transporte",
    "--color-cat-restaurante",
    "--color-cat-casa",
    "--color-cat-saude",
    "--color-cat-lazer",
    "--color-cat-assinatura",
    "--color-cat-renda",
    "--color-cat-outros",
    "--color-cat-amarelo",
    "--color-cat-indigo",
    "--color-cat-marrom",
    "--color-cat-investimento",
  ];

  let name = $state("");
  let kind = $state<"expense" | "income" | "transfer">("expense");
  let colorToken = $state(COLOR_TOKENS[0]);
  let busy = $state(false);
  let error = $state<string | null>(null);

  $effect(() => {
    name = initial?.name ?? "";
    kind = (initial?.kind ?? "expense") as "expense" | "income" | "transfer";
    colorToken = initial?.color_token ?? COLOR_TOKENS[8]; // outros default
  });

  async function submit(e: Event) {
    e.preventDefault();
    error = null;
    if (name.trim().length === 0) {
      error = t("categories.name_required");
      return;
    }
    busy = true;
    try {
      await onSave({ name: name.trim(), colorToken, kind });
      if (!initial) {
        name = "";
        kind = "expense";
        colorToken = COLOR_TOKENS[8];
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
    {initial ? t("categories.form_edit") : t("categories.form_new")}
  </div>

  <div class="grid grid-cols-[1fr_140px_auto] gap-2 items-end">
    <label class="flex flex-col gap-1">
      <span class="text-[11px] text-fg-muted">{t("categories.name")}</span>
      <input
        bind:value={name}
        placeholder={t("categories.name_placeholder")}
        class="rounded-md border border-border bg-surface-2 px-2 py-1 text-[12px] text-fg focus:outline-none focus:border-accent"
      />
    </label>

    <label class="flex flex-col gap-1">
      <span class="text-[11px] text-fg-muted">{t("categories.type")}</span>
      <select
        bind:value={kind}
        class="rounded-md border border-border bg-surface-2 px-2 py-1 text-[12px] text-fg"
      >
        <option value="expense">{t("kind.expense")}</option>
        <option value="income">{t("kind.income")}</option>
        <option value="transfer">{t("kind.transfer")}</option>
      </select>
    </label>

    <div class="flex gap-2">
      {#if onCancel}
        <Button variant="ghost" onclick={onCancel} type="button">{t("common.cancel")}</Button>
      {/if}
      <Button type="submit" disabled={busy}>
        {busy ? t("categories.saving") : initial ? t("categories.save") : t("categories.add")}
      </Button>
    </div>
  </div>

  <div class="flex flex-col gap-1.5">
    <span class="text-[11px] text-fg-muted">{t("categories.color")}</span>
    <div class="flex gap-1.5 flex-wrap">
      {#each COLOR_TOKENS as token}
        <button
          type="button"
          onclick={() => (colorToken = token)}
          aria-label={token}
          class="w-6 h-6 rounded-full transition-all
                 {colorToken === token ? 'ring-2 ring-accent ring-offset-2 ring-offset-surface scale-110' : 'hover:scale-105'}"
          style="background: var({token})"
        ></button>
      {/each}
    </div>
  </div>

  {#if error}
    <div class="text-[11px] text-neg">{error}</div>
  {/if}
</form>
