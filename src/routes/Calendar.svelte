<script lang="ts">
  import { onMount } from "svelte";
  import MonthStepper from "$lib/components/shell/MonthStepper.svelte";
  import CalendarGrid from "$lib/components/calendar/CalendarGrid.svelte";
  import { formatMoney } from "$lib/format/money";
  import { filters } from "$lib/stores/filters.svelte";
  import { calendarEvents } from "$lib/api/rules";
  import type { CalendarEvent } from "$lib/bindings";

  let events = $state<CalendarEvent[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  let today = new Date().toISOString().slice(0, 10);

  /** Resolve o mês a usar quando filters.month vier como "YYYY" (ano inteiro):
   *  cai pro mês atual desse ano. Calendário não suporta visão de ano. */
  function monthForCalendar(m: string | null): string {
    if (!m) {
      const d = new Date();
      return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}`;
    }
    if (m.length === 7) return m;
    if (m.length === 4) {
      // ano-only: usa mês atual
      const mm = String(new Date().getMonth() + 1).padStart(2, "0");
      return `${m}-${mm}`;
    }
    return m;
  }

  let viewMonth = $derived(monthForCalendar(filters.month));

  async function refresh() {
    loading = true;
    error = null;
    try {
      events = await calendarEvents(viewMonth);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  onMount(refresh);

  $effect(() => {
    // re-fetch quando o mês muda
    viewMonth;
    void refresh();
  });

  function onMonthChange(m: string | null) {
    filters.month = m;
  }

  // Agrupa pra a lista lateral.
  let paidEvents = $derived(events.filter((e) => e.paid_day != null));
  let pendingEvents = $derived(events.filter((e) => e.paid_day == null && e.due_day != null));

  function tokenColor(t: string | null | undefined): string {
    return t ? `var(${t})` : "var(--color-cat-outros)";
  }
</script>

<section class="p-8 max-w-6xl mx-auto flex flex-col gap-5">
  <header class="flex items-baseline justify-between gap-4 flex-wrap">
    <div class="flex flex-col gap-1">
      <h2 class="text-xl font-semibold tracking-tight" style="font-family: var(--font-display)">
        Calendário
      </h2>
      <p class="text-xs text-fg-faint max-w-xl">
        Vencimentos e pagamentos das regras com "Vence dia" definido. Pagamentos
        identificados automaticamente cruzando o pattern com transações do mês.
      </p>
    </div>
    <MonthStepper month={viewMonth} onChange={onMonthChange} />
  </header>

  {#if loading && events.length === 0}
    <div class="text-fg-faint text-sm">Carregando…</div>
  {:else if error}
    <div class="rounded-lg border border-border bg-surface p-3 text-sm text-neg">{error}</div>
  {:else}
    <div class="grid grid-cols-[1fr_320px] gap-4">
      <CalendarGrid month={viewMonth} {events} {today} />

      <aside class="flex flex-col gap-4">
        <div class="rounded-lg border border-border-subtle bg-surface p-4 flex flex-col gap-2.5">
          <div class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint">
            Pagos no mês
          </div>
          {#each paidEvents as e (e.rule_id + ":paid")}
            <div class="flex items-center gap-2 text-[12px]">
              <span class="w-2 h-2 rounded-full shrink-0" style="background: {tokenColor(e.category_color_token)}"></span>
              <span class="font-medium truncate flex-1">{e.pattern}</span>
              <span class="tabular text-fg-muted text-[11px]">dia {e.paid_day}</span>
              {#if e.paid_amount}
                <span class="tabular text-pos text-[11px]">{formatMoney(e.paid_amount)}</span>
              {/if}
            </div>
          {:else}
            <div class="text-fg-faint italic text-[11.5px]">Nenhum pagamento identificado.</div>
          {/each}
        </div>

        <div class="rounded-lg border border-border-subtle bg-surface p-4 flex flex-col gap-2.5">
          <div class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint">
            Pendentes
          </div>
          {#each pendingEvents as e (e.rule_id + ":pending")}
            <div class="flex items-center gap-2 text-[12px]">
              <span class="w-2 h-2 rounded-full shrink-0" style="background: {tokenColor(e.category_color_token)}"></span>
              <span class="font-medium truncate flex-1">{e.pattern}</span>
              <span class="tabular text-fg-muted text-[11px]">vence dia {e.due_day}</span>
            </div>
          {:else}
            <div class="text-fg-faint italic text-[11.5px]">Tudo em dia.</div>
          {/each}
        </div>
      </aside>
    </div>
  {/if}
</section>
