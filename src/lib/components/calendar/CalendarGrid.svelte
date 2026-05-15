<script lang="ts">
  import { formatMoney } from "$lib/format/money";
  import type { CalendarEvent } from "$lib/bindings";

  type Props = {
    /** "YYYY-MM" do mês exibido */
    month: string;
    events: CalendarEvent[];
    /** Hoje, no formato "YYYY-MM-DD" */
    today: string;
  };

  let { month, events, today }: Props = $props();

  const WEEKDAYS = ["Dom", "Seg", "Ter", "Qua", "Qui", "Sex", "Sáb"];

  type DayCell = {
    day: number | null;
    isToday: boolean;
    /** Eventos cujo due_day cai neste dia (pendentes/vencidos) */
    due: CalendarEvent[];
    /** Eventos cujo paid_day cai neste dia */
    paid: CalendarEvent[];
  };

  let cells = $derived(buildGrid(month, events, today));

  function buildGrid(
    monthStr: string,
    evs: CalendarEvent[],
    todayStr: string,
  ): DayCell[] {
    const [yStr, mStr] = monthStr.split("-");
    const year = Number(yStr);
    const monthIdx = Number(mStr) - 1;
    const first = new Date(year, monthIdx, 1);
    const daysInMonth = new Date(year, monthIdx + 1, 0).getDate();
    const startWeekday = first.getDay(); // 0 = Dom

    const todayPrefix = todayStr.slice(0, 7);
    const todayDay = todayPrefix === monthStr ? Number(todayStr.slice(8, 10)) : -1;

    // Pre-bucket events by due_day and paid_day.
    const dueByDay = new Map<number, CalendarEvent[]>();
    const paidByDay = new Map<number, CalendarEvent[]>();
    for (const e of evs) {
      if (e.due_day != null) {
        // Se due_day > daysInMonth (ex: dia 31 num mês de 30), ancora no último dia.
        const d = Math.min(e.due_day, daysInMonth);
        const list = dueByDay.get(d) ?? [];
        list.push(e);
        dueByDay.set(d, list);
      }
      if (e.paid_day != null) {
        const list = paidByDay.get(e.paid_day) ?? [];
        list.push(e);
        paidByDay.set(e.paid_day, list);
      }
    }

    const out: DayCell[] = [];
    for (let i = 0; i < startWeekday; i++) {
      out.push({ day: null, isToday: false, due: [], paid: [] });
    }
    for (let d = 1; d <= daysInMonth; d++) {
      out.push({
        day: d,
        isToday: d === todayDay,
        due: dueByDay.get(d) ?? [],
        paid: paidByDay.get(d) ?? [],
      });
    }
    return out;
  }

  function isPaidElsewhere(e: CalendarEvent): boolean {
    return e.paid_day != null && e.due_day != null && e.paid_day !== e.due_day;
  }

  function isOverdue(e: CalendarEvent, todayDay: number): boolean {
    return e.due_day != null && e.paid_day == null && e.due_day < todayDay;
  }

  function tokenStyle(t: string | null | undefined): string {
    return t ? `var(${t})` : "var(--color-cat-outros)";
  }

  let todayDayInMonth = $derived(
    today.slice(0, 7) === month ? Number(today.slice(8, 10)) : -1,
  );
</script>

<div class="rounded-lg border border-border-subtle bg-surface overflow-hidden">
  <div class="grid grid-cols-7 bg-surface-2 border-b border-border-subtle">
    {#each WEEKDAYS as wd}
      <div class="px-2 py-1.5 text-[10px] uppercase tracking-wider font-semibold text-fg-faint text-center">
        {wd}
      </div>
    {/each}
  </div>

  <div class="grid grid-cols-7">
    {#each cells as cell, i}
      <div
        class="min-h-[88px] border-r border-b border-border-subtle p-1.5 flex flex-col gap-1
               {cell.day === null ? 'bg-bg/40' : ''}
               {cell.isToday ? 'bg-accent-soft/30' : ''}
               {i % 7 === 6 ? 'border-r-0' : ''}"
      >
        {#if cell.day !== null}
          <div class="flex items-center justify-between">
            <span class="text-[10.5px] tabular {cell.isToday ? 'text-accent font-semibold' : 'text-fg-faint'}">
              {cell.day}
            </span>
          </div>

          <!-- Eventos PAGOS no dia (verde, contorno cheio) -->
          {#each cell.paid as e}
            <div
              class="text-[10px] rounded px-1.5 py-0.5 truncate flex items-center gap-1"
              style="background: color-mix(in oklch, {tokenStyle(e.category_color_token)} 22%, transparent); color: var(--color-fg)"
              title={`${e.pattern} — pago${e.paid_amount ? ' ' + formatMoney(e.paid_amount) : ''}${isPaidElsewhere(e) ? ` (vence dia ${e.due_day})` : ''}`}
            >
              <span class="w-1.5 h-1.5 rounded-full shrink-0" style="background: {tokenStyle(e.category_color_token)}"></span>
              <span class="truncate">✓ {e.pattern}</span>
            </div>
          {/each}

          <!-- Eventos com VENCIMENTO no dia (não pagos: outline; pagos noutro dia: ainda mostra como pendente futuro? não — se já foi pago, só aparece a marca de pago) -->
          {#each cell.due as e}
            {#if e.paid_day == null}
              <div
                class="text-[10px] rounded px-1.5 py-0.5 truncate flex items-center gap-1 border
                       {isOverdue(e, todayDayInMonth) ? 'border-neg/60 text-neg' : 'border-fg-faint/40 text-fg-muted'}"
                title={`${e.pattern} — vence dia ${e.due_day}${isOverdue(e, todayDayInMonth) ? ' (atrasado)' : ''}`}
              >
                <span class="w-1.5 h-1.5 rounded-full shrink-0" style="background: {tokenStyle(e.category_color_token)}"></span>
                <span class="truncate">{e.pattern}</span>
              </div>
            {/if}
          {/each}
        {/if}
      </div>
    {/each}
  </div>
</div>
