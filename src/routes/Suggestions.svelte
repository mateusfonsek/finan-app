<script lang="ts">
  import { locale } from "$lib/i18n/locale.svelte";

  const t = locale.t;
  import { onMount } from "svelte";
  import { Button } from "$lib/components/ui/button";
  import Page from "$lib/components/ui/Page.svelte";
  import Card from "$lib/components/ui/Card.svelte";
  import Loading from "$lib/components/ui/Loading.svelte";
  import ErrorNote from "$lib/components/ui/ErrorNote.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
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
      error = t("import.choose_category_first");
      return;
    }
    const finalPattern = (patternOverride[s.key] ?? s.suggested_pattern).trim();
    if (!finalPattern) {
      error = t("rule_form.pattern_required");
      return;
    }
    busyKey = s.key;
    error = null;
    try {
      await createRule({
        patterns: [finalPattern],
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
    if (key.startsWith("cnpj:")) return { text: t("suggestions.badge_cnpj"), tone: "indigo" };
    if (key.startsWith("debito:")) return { text: t("suggestions.badge_debito"), tone: "neutral" };
    if (key.startsWith("pix_out:")) return { text: t("suggestions.badge_pix_out"), tone: "amarelo" };
    if (key.startsWith("pix_in:")) return { text: t("suggestions.badge_pix_in"), tone: "mercado" };
    if (key.startsWith("ted_in:")) return { text: t("suggestions.badge_ted_in"), tone: "mercado" };
    if (key.startsWith("boleto:")) return { text: t("suggestions.badge_boleto"), tone: "marrom" };
    if (key.startsWith("system:")) return { text: t("suggestions.badge_system"), tone: "neutral" };
    return { text: t("suggestions.badge_other"), tone: "neutral" };
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

<Page title={t("suggestions.title")} subtitle={t("suggestions.subtitle")}>
  {#if loading}
    <Loading />
  {:else}
    {#if error}
      <ErrorNote message={error} />
    {/if}

    <Card title={t("suggestions.recurring_no_category")}>
      {#snippet actions()}
        <label class="flex items-center gap-2 text-foot text-fg-muted shrink-0">
          {t("suggestions.min_occurrences")}
          <input
            type="number"
            min="1"
            max="50"
            value={minCount}
            onchange={(e) => onMinCountChange(Number((e.currentTarget as HTMLInputElement).value))}
            class="field w-14 tabular"
          />
        </label>
      {/snippet}

      {#if suggestions.length === 0}
        <EmptyState icon="sparkles" title={t("suggestions.empty_title")} description={t("suggestions.empty")} compact />
      {:else}
        <div class="flex flex-col gap-2">
          {#each suggestions as s (s.key)}
            {@const badge = badgeFor(s.key)}
            {@const tone = toneColor(badge.tone)}
            <article class="card-inset p-3 flex flex-col gap-2.5 min-w-0">
              <!-- Cabeçalho: tipo + rótulo + métricas -->
              <div class="flex items-start gap-2.5 min-w-0">
                <span
                  class="text-cap2 font-semibold px-1.5 py-0.5 rounded-full whitespace-nowrap shrink-0 mt-px"
                  style="color: {tone}; background: color-mix(in oklch, {tone} 15%, transparent);"
                >
                  {badge.text}
                </span>
                <div class="flex-1 min-w-0">
                  <div class="text-callout text-fg font-medium truncate" title={s.label}>
                    {s.label}
                  </div>
                  <div class="text-cap text-fg-subtle truncate" title={s.sample_description}>
                    {s.sample_description}
                  </div>
                </div>
                <div class="shrink-0 flex items-baseline gap-3 text-sub tabular">
                  <span class="text-fg-subtle">{s.count}×</span>
                  <span class="{Number(s.total) < 0 ? 'text-neg' : 'text-pos'} font-medium">
                    {fmtAbsTotal(s.total)}
                  </span>
                </div>
              </div>

              <!-- Rodapé: pattern + categoria + ação -->
              <div class="flex gap-2 items-center min-w-0">
                <label class="flex-1 min-w-0 flex items-center gap-2">
                  <span class="text-foot text-fg-subtle shrink-0">
                    {t("suggestions.pattern_label")}
                  </span>
                  <input
                    value={patternOverride[s.key] ?? s.suggested_pattern}
                    oninput={(e) => {
                      patternOverride[s.key] = (e.currentTarget as HTMLInputElement).value;
                    }}
                    class="field flex-1 min-w-0 font-mono !bg-surface"
                    title={t("tx_notes.pattern_title")}
                  />
                </label>
                <select
                  value={chosen[s.key] == null ? "" : String(chosen[s.key])}
                  onchange={(e) => {
                    const v = (e.currentTarget as HTMLSelectElement).value;
                    chosen[s.key] = v === "" ? null : Number(v);
                  }}
                  aria-label={t("import.category_placeholder")}
                  class="field w-36 shrink-0 !bg-surface"
                >
                  <option value="">{t("import.category_placeholder")}</option>
                  {#each categories as c}
                    <option value={String(c.id)}>{c.name}</option>
                  {/each}
                </select>
                <Button
                  onclick={() => createRuleFor(s)}
                  disabled={busyKey === s.key || chosen[s.key] == null}
                >
                  {busyKey === s.key ? t("import.creating") : t("suggestions.create")}
                </Button>
              </div>
            </article>
          {/each}
        </div>
      {/if}
    </Card>
  {/if}
</Page>
