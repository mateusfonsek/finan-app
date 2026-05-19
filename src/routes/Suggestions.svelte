<script lang="ts">
  import { onMount } from "svelte";
  import { Button } from "$lib/components/ui/button";
  import { listCategories } from "$lib/api/categories";
  import { createRule } from "$lib/api/rules";
  import { suggestRules } from "$lib/api/suggestions";
  import { formatMoney } from "$lib/format/money";
  import type { Category, RuleSuggestion } from "$lib/bindings";

  let categories = $state<Category[]>([]);
  let suggestions = $state<RuleSuggestion[]>([]);
  let minCount = $state(4);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let busyKey = $state<string | null>(null);
  let chosen = $state<Record<string, number | null>>({});
  let patternOverride = $state<Record<string, string>>({});

  async function refresh() {
    suggestions = await suggestRules(minCount);
  }

  onMount(async () => {
    try {
      categories = await listCategories();
      await refresh();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  });

  async function onMinCountChange(v: number) {
    minCount = Math.max(1, Math.floor(v));
    try {
      await refresh();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  async function createRuleFor(s: RuleSuggestion) {
    const categoryId = chosen[s.key];
    if (categoryId == null) {
      error = "Escolha uma categoria antes de criar a regra.";
      return;
    }
    const finalPattern = (patternOverride[s.key] ?? s.suggested_pattern).trim();
    if (!finalPattern) {
      error = "Pattern não pode ser vazio.";
      return;
    }
    busyKey = s.key;
    error = null;
    try {
      await createRule({
        pattern: finalPattern,
        category_id: categoryId,
        priority: 5,
        due_day: null,
      });
      await refresh();
      delete chosen[s.key];
      delete patternOverride[s.key];
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      busyKey = null;
    }
  }

  function fmtAbsTotal(amount: string): string {
    return formatMoney(String(Math.abs(Number(amount))));
  }

  /** Tipo do agrupamento, derivado da chave estável vinda do backend. */
  function badgeFor(key: string): { text: string; tone: string } {
    if (key.startsWith("cnpj:")) return { text: "CNPJ", tone: "indigo" };
    if (key.startsWith("debito:")) return { text: "DÉBITO", tone: "neutral" };
    if (key.startsWith("pix_out:")) return { text: "PIX →", tone: "amarelo" };
    if (key.startsWith("pix_in:")) return { text: "PIX ←", tone: "mercado" };
    if (key.startsWith("ted_in:")) return { text: "TRANSF. ←", tone: "mercado" };
    if (key.startsWith("boleto:")) return { text: "BOLETO", tone: "marrom" };
    if (key.startsWith("system:")) return { text: "SISTEMA", tone: "neutral" };
    return { text: "OUTRO", tone: "neutral" };
  }

  function toneColor(tone: string): string {
    switch (tone) {
      case "indigo":   return "var(--color-cat-indigo)";
      case "amarelo":  return "var(--color-cat-amarelo)";
      case "mercado":  return "var(--color-cat-mercado)";
      case "marrom":   return "var(--color-cat-marrom)";
      default:         return "var(--color-cat-outros)";
    }
  }
</script>

<section class="p-8 max-w-5xl mx-auto flex flex-col gap-5">
  <header class="flex flex-col gap-1">
    <h2 class="text-xl font-semibold tracking-tight" style="font-family: var(--font-display)">
      Sugestões de regras
    </h2>
    <p class="text-xs text-fg-faint max-w-xl">
      <strong>Gastos</strong> recorrentes sem CNPJ (débito de cartão, Pix entre
      pessoas, boletos) que se repetem. CNPJs já são resolvidos automaticamente
      na tela de importação. Entradas não aparecem aqui — renda é rastreada por
      contraparte no Dashboard (Fontes de Renda).
    </p>
  </header>

  {#if loading}
    <div class="text-fg-faint text-sm">Carregando…</div>
  {:else}
    {#if error}
      <div class="rounded-lg border border-border bg-surface p-3 text-sm text-neg">{error}</div>
    {/if}

    <div class="rounded-xl bg-surface border border-border-subtle p-4 flex flex-col gap-3">
      <div class="flex items-baseline justify-between">
        <div class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint">
          Recorrências sem categoria
        </div>
        <label class="flex items-center gap-2 text-[11px] text-fg-muted">
          mín. ocorrências
          <input
            type="number"
            min="1"
            max="50"
            value={minCount}
            onchange={(e) => onMinCountChange(Number((e.currentTarget as HTMLInputElement).value))}
            class="w-14 rounded-md border border-border bg-surface-2 px-2 py-0.5 text-[12px] text-fg tabular focus:outline-none focus:border-accent"
          />
        </label>
      </div>

      {#if suggestions.length === 0}
        <div class="text-[12px] text-fg-faint py-4">
          Nada por aqui — todas as contrapartes recorrentes já têm categoria, ou
          aumente o filtro de ocorrências.
        </div>
      {:else}
        <div class="flex flex-col gap-2">
          {#each suggestions as s (s.key)}
            {@const badge = badgeFor(s.key)}
            {@const tone = toneColor(badge.tone)}
            <article class="rounded-md border border-border-subtle bg-surface-2/50 p-3 flex flex-col gap-2 min-w-0">
              <!-- Header: tipo + label + métricas -->
              <div class="flex items-start gap-2 min-w-0">
                <span
                  class="text-[10px] uppercase tracking-wider font-semibold px-1.5 py-0.5 rounded border whitespace-nowrap shrink-0"
                  style="color: {tone}; border-color: {tone}; opacity: 0.9;"
                >
                  {badge.text}
                </span>
                <div class="flex-1 min-w-0">
                  <div class="text-[12.5px] text-fg font-medium truncate" title={s.label}>
                    {s.label}
                  </div>
                  <div class="text-[10.5px] text-fg-faint truncate" title={s.sample_description}>
                    {s.sample_description}
                  </div>
                </div>
                <div class="shrink-0 flex items-baseline gap-3 text-[11.5px] tabular">
                  <span class="text-fg-muted">{s.count}×</span>
                  <span class="{Number(s.total) < 0 ? 'text-neg' : 'text-pos'} font-medium">
                    {fmtAbsTotal(s.total)}
                  </span>
                </div>
              </div>

              <!-- Footer: pattern + categoria + ação -->
              <div class="flex gap-2 items-center min-w-0">
                <label class="flex-1 min-w-0 flex items-center gap-1.5">
                  <span class="text-[10px] uppercase tracking-wider text-fg-faint shrink-0">Pattern</span>
                  <input
                    value={patternOverride[s.key] ?? s.suggested_pattern}
                    oninput={(e) => {
                      patternOverride[s.key] = (e.currentTarget as HTMLInputElement).value;
                    }}
                    class="flex-1 min-w-0 rounded-md border border-border bg-surface px-2 py-1 text-[12px] text-fg font-mono focus:outline-none focus:border-accent"
                    title="LIKE substring que será gravado na regra"
                  />
                </label>
                <select
                  value={chosen[s.key] == null ? "" : String(chosen[s.key])}
                  onchange={(e) => {
                    const v = (e.currentTarget as HTMLSelectElement).value;
                    chosen[s.key] = v === "" ? null : Number(v);
                  }}
                  class="w-36 shrink-0 rounded-md border border-border bg-surface px-2 py-1 text-[12px] text-fg"
                >
                  <option value="">— categoria —</option>
                  {#each categories as c}
                    <option value={String(c.id)}>{c.name}</option>
                  {/each}
                </select>
                <Button
                  onclick={() => createRuleFor(s)}
                  disabled={busyKey === s.key || chosen[s.key] == null}
                >
                  {busyKey === s.key ? "…" : "Criar"}
                </Button>
              </div>
            </article>
          {/each}
        </div>
      {/if}
    </div>
  {/if}
</section>
