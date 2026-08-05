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

<!-- The number is the subject: it dominates the card, the label names it, and
     the caption explains what is NOT included. Hierarchy by size and weight,
     not by capitals. -->
<div class="card p-3.5 flex flex-col gap-1">
  <div class="text-foot font-medium text-fg-muted truncate" title={label}>{label}</div>
  <div class="text-title1 font-semibold tabular {toneClass}">
    {raw ? value : formatMoney(value)}
  </div>
  {#if caption}
    <div class="text-cap text-fg-subtle leading-snug">{caption}</div>
  {/if}
</div>
