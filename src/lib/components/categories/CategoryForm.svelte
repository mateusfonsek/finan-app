<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import Icon from "$lib/components/ui/Icon.svelte";
  import ErrorNote from "$lib/components/ui/ErrorNote.svelte";
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

<form
  onsubmit={submit}
  class="card p-4 flex flex-col gap-3.5 {initial ? 'ring-[1.5px] ring-accent ring-inset' : ''}"
>
  <div class="flex items-center gap-2">
    <Icon name={initial ? "pencil" : "plus"} size={13} class="text-fg-subtle" />
    <span class="section-title">{initial ? t("categories.form_edit") : t("categories.form_new")}</span>
  </div>

  <div class="grid grid-cols-[minmax(0,320px)_150px_auto] gap-2.5 items-end justify-start">
    <label class="flex flex-col gap-1">
      <span class="text-foot text-fg-subtle">{t("categories.name")}</span>
      <input bind:value={name} placeholder={t("categories.name_placeholder")} class="field" />
    </label>

    <label class="flex flex-col gap-1">
      <span class="text-foot text-fg-subtle">{t("categories.type")}</span>
      <select bind:value={kind} class="field">
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
    <span class="text-foot text-fg-subtle">{t("categories.color")}</span>
    <!-- Poço de cores: o selecionado ganha um anel externo e um visto por
         dentro — a marca não depende só do contorno pra ser vista. -->
    <div class="flex gap-2 flex-wrap" role="radiogroup" aria-label={t("categories.color")}>
      {#each COLOR_TOKENS as token}
        {@const selected = colorToken === token}
        <button
          type="button"
          role="radio"
          aria-checked={selected}
          onclick={() => (colorToken = token)}
          aria-label={token.replace("--color-cat-", "")}
          class="w-6 h-6 rounded-full grid place-items-center text-white
                 transition-transform duration-[var(--dur-fast)] ease-[var(--ease-snap)]
                 hover:scale-110 active:scale-95
                 {selected ? 'ring-2 ring-accent ring-offset-2 ring-offset-surface' : ''}"
          style="background: var({token})"
        >
          {#if selected}
            <Icon name="check" size={12} stroke={3} class="drop-shadow-[0_1px_2px_rgba(0,0,0,.45)]" />
          {/if}
        </button>
      {/each}
    </div>
  </div>

  {#if error}
    <ErrorNote message={error} />
  {/if}
</form>
