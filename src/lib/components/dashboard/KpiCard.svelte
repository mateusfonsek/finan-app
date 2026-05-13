<script lang="ts">
  import { formatMoney } from "$lib/format/money";

  type Props = {
    label: string;
    value: string;
    caption?: string;
    tone?: "pos" | "neg" | "muted";
    raw?: boolean;
  };

  let { label, value, caption, tone = "muted", raw = false }: Props = $props();

  let toneClass = $derived(
    tone === "pos" ? "text-pos" : tone === "neg" ? "text-neg" : "text-fg",
  );
</script>

<div class="rounded-xl bg-surface border border-border-subtle p-4 flex flex-col gap-1.5">
  <div class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint">
    {label}
  </div>
  <div class="text-2xl font-semibold tracking-tight tabular {toneClass}" style="font-family: var(--font-display)">
    {raw ? value : formatMoney(value)}
  </div>
  {#if caption}
    <div class="text-[11px] text-fg-faint tabular">{caption}</div>
  {/if}
</div>
