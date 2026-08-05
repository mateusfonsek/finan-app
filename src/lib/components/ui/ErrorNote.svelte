<script lang="ts">
  import Icon from "./Icon.svelte";

  type Props = { message: string; tone?: "error" | "success" };
  let { message, tone = "error" }: Props = $props();
</script>

<!-- Error feedback sits next to what failed, not in a banner at the top.
     Colour plus symbol: the state still reads without colour vision. -->
<div
  class="card px-3 py-2.5 flex items-start gap-2.5 text-callout {tone === 'error'
    ? 'text-neg'
    : 'text-pos'}"
  role={tone === "error" ? "alert" : "status"}
  style="border-color: color-mix(in oklch, var({tone === 'error'
    ? '--color-neg'
    : '--color-pos'}) 32%, var(--color-border-subtle))"
>
  <Icon
    name={tone === "error" ? "circleAlert" : "circleCheck"}
    size={14}
    stroke={2}
    class="mt-px shrink-0"
  />
  <span class="min-w-0 selectable">{message}</span>
</div>
