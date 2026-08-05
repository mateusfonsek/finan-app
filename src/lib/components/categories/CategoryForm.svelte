<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import Icon from "$lib/components/ui/Icon.svelte";
  import ErrorNote from "$lib/components/ui/ErrorNote.svelte";
  import { locale } from "$lib/i18n/locale.svelte";
  import ColorWell, { DEFAULT_COLOR } from "./ColorWell.svelte";

  const t = locale.t;

  /** Só cria. Editar acontece no painel lateral (`CategoryPanel`), aberto pela
   *  linha da lista — um formulário que troca de identidade no meio do uso
   *  esconde de qual categoria ele está falando. */
  type Props = {
    onSave: (data: { name: string; colorToken: string; kind: string }) => Promise<void>;
  };

  let { onSave }: Props = $props();

  let name = $state("");
  let kind = $state<"expense" | "income" | "transfer">("expense");
  let colorToken = $state(DEFAULT_COLOR);
  let busy = $state(false);
  let error = $state<string | null>(null);

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
      name = "";
      kind = "expense";
      colorToken = DEFAULT_COLOR;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }
</script>

<form onsubmit={submit} class="card p-4 flex flex-col gap-3.5">
  <div class="flex items-center gap-2">
    <Icon name="plus" size={13} class="text-fg-subtle" />
    <span class="section-title">{t("categories.form_new")}</span>
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

    <Button type="submit" disabled={busy}>
      {busy ? t("categories.saving") : t("categories.add")}
    </Button>
  </div>

  <div class="flex flex-col gap-1.5">
    <span class="text-foot text-fg-subtle">{t("categories.color")}</span>
    <ColorWell value={colorToken} onChange={(token) => (colorToken = token)} />
  </div>

  {#if error}
    <ErrorNote message={error} />
  {/if}
</form>
