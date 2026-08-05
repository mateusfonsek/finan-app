<script module lang="ts">
  /**
   * Uma linha da lista de trechos. O `id` existe porque o valor não serve de
   * chave: dois trechos podem ficar momentaneamente iguais enquanto se digita,
   * e aí o `{#each}` embaralharia os campos debaixo do cursor.
   */
  export type PatternRow = { id: number; value: string };

  let nextId = 0;

  /** Converte a lista vinda do backend em linhas. Lista vazia vira um campo
   *  vazio: uma regra sem nenhum trecho não casaria nada. */
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
    /** O pai é dono da lista; aqui só devolvemos a próxima versão dela. */
    onChange: (rows: PatternRow[]) => void;
    /** Coloca o cursor no primeiro campo assim que a lista aparece — usado
     *  pelo painel de edição, onde digitar é a primeira coisa que se faz. */
    autofocus?: boolean;
  };

  let { rows, onChange, autofocus = false }: Props = $props();

  let inputs: Record<number, HTMLInputElement | undefined> = {};

  function update(id: number, value: string) {
    onChange(rows.map((r) => (r.id === id ? { ...r, value } : r)));
  }

  /** A última linha nunca some — ela só esvazia. Uma lista vazia esconderia
   *  que a regra precisa de pelo menos um trecho pra existir. */
  function remove(id: number) {
    if (rows.length === 1) {
      onChange([{ ...rows[0], value: "" }]);
      inputs[rows[0].id]?.focus();
      return;
    }
    onChange(rows.filter((r) => r.id !== id));
  }

  /** O foco vai pro campo novo: digitar nele é a próxima coisa que acontece. */
  async function add() {
    const row = blankRow();
    onChange([...rows, row]);
    await tick();
    inputs[row.id]?.focus();
  }

  /** ↩ no último campo adiciona outro, em vez de submeter o formulário meio
   *  preenchido — a lista se comporta como lista. */
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

  // Só na montagem: um $effect leria `rows` e roubaria o cursor de volta pro
  // primeiro campo a cada tecla digitada em qualquer um dos outros.
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
      <!-- O ✕ some (mas guarda o lugar) quando não há o que remover, pra lista
           não pular de largura entre um estado e outro. -->
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
