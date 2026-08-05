<script module lang="ts">
  /**
   * One row of the snippet list. The `id` exists because the value cannot be
   * the key: two snippets can briefly match while typing, and `{#each}` would
   * then shuffle the fields under the cursor.
   */
  export type PatternRow = { id: number; value: string };

  let nextId = 0;

  /** Turns the backend list into rows. An empty list becomes one empty field:
   *  a rule with no snippets would match nothing. */
  export function rowsFrom(values: string[]): PatternRow[] {
    const list = values.length > 0 ? values : [""];
    return list.map((value) => ({ id: nextId++, value }));
  }

  export function valuesOf(rows: PatternRow[]): string[] {
    return rows.map((r) => r.value);
  }

  function blankRow(): PatternRow {
    return { id: nextId++, value: "" };
  }
</script>

<script lang="ts">
  import { onMount, tick } from "svelte";
  import { flip } from "svelte/animate";
  import Icon from "$lib/components/ui/Icon.svelte";
  import { locale } from "$lib/i18n/locale.svelte";
  import { DUR, SNAP, reducedMotion, rise } from "$lib/motion";

  const t = locale.t;

  type Props = {
    rows: PatternRow[];
    /** The parent owns the list; this only returns its next version. */
    onChange: (rows: PatternRow[]) => void;
    /** Puts the cursor in the first field as soon as the list appears — used
     *  by the edit panel, where typing is the first thing that happens. */
    autofocus?: boolean;
  };

  let { rows, onChange, autofocus = false }: Props = $props();

  let inputs: Record<number, HTMLInputElement | undefined> = {};

  function update(id: number, value: string) {
    onChange(rows.map((r) => (r.id === id ? { ...r, value } : r)));
  }

  /** The last row never disappears, it only empties. An empty list would hide
   *  that a rule needs at least one snippet to exist. */
  function remove(id: number) {
    if (rows.length === 1) {
      onChange([{ ...rows[0], value: "" }]);
      inputs[rows[0].id]?.focus();
      return;
    }
    onChange(rows.filter((r) => r.id !== id));
  }

  /** Focus moves to the new field: typing in it is what happens next. */
  async function add() {
    const row = blankRow();
    onChange([...rows, row]);
    await tick();
    inputs[row.id]?.focus();
  }

  /** Enter on the last field adds another instead of submitting a half-filled
   *  form — the list behaves like a list. */
  function onkeydown(e: KeyboardEvent, index: number) {
    if (e.key !== "Enter" || e.metaKey || e.ctrlKey) return;
    if (index !== rows.length - 1) return;
    if (rows[index].value.trim() === "") return;
    e.preventDefault();
    void add();
  }

  let flipParams = $derived(
    reducedMotion() ? { duration: 0 } : { duration: DUR.fast, easing: SNAP },
  );

  // Mount only: an $effect would read `rows` and steal the cursor back to the
  // first field on every keystroke in any of the others.
  onMount(async () => {
    if (!autofocus) return;
    await tick();
    const el = inputs[rows[0]?.id ?? -1];
    el?.focus();
    el?.select();
  });
</script>

<div class="flex flex-col gap-1.5">
  {#each rows as row, i (row.id)}
    <div class="flex items-center gap-1.5" animate:flip={flipParams} transition:rise>
      <input
        bind:this={inputs[row.id]}
        value={row.value}
        oninput={(e) => update(row.id, e.currentTarget.value)}
        onkeydown={(e) => onkeydown(e, i)}
        placeholder={i === 0
          ? t("rule_form.pattern_placeholder")
          : t("rule_form.pattern_placeholder_more")}
        class="field font-mono flex-1 min-w-0"
      />
      <!-- The ✕ hides (but keeps its space) when there is nothing to remove,
           so the list does not jump width between states. -->
      <button
        type="button"
        onclick={() => remove(row.id)}
        title={t("rule_form.remove_pattern")}
        aria-label={t("rule_form.remove_pattern")}
        tabindex={rows.length === 1 && row.value.trim() === "" ? -1 : 0}
        class="press w-6 h-6 shrink-0 grid place-items-center rounded-[var(--radius-sm)]
               text-fg-faint hover:bg-neg/12 hover:text-neg
               transition-colors duration-[var(--dur-fast)]
               {rows.length === 1 && row.value.trim() === '' ? 'invisible' : ''}"
      >
        <Icon name="x" size={12.5} stroke={2} />
      </button>
    </div>
  {/each}

  <button
    type="button"
    onclick={add}
    class="press-sm self-start flex items-center gap-1.5 text-foot text-fg-muted
           hover:text-accent transition-colors duration-[var(--dur-fast)] mt-0.5"
  >
    <Icon name="plus" size={12} stroke={2.2} />
    {t("rule_form.add_pattern")}
  </button>
</div>
