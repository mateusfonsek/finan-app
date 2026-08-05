<script lang="ts">
  type Props = { size?: number; class?: string };
  let { size = 16, class: className = "" }: Props = $props();

  const SPOKES = 8;
</script>

<!-- The macOS activity indicator: spokes fading in sequence. Under "reduce
     motion" the animation stops — the shape still reads as "working", without
     vestibular oscillation. -->
<span
  class="inline-block relative shrink-0 {className}"
  style="width: {size}px; height: {size}px"
  role="progressbar"
  aria-label="…"
>
  {#each Array(SPOKES) as _, i}
    <span
      class="absolute left-1/2 top-1/2 rounded-full bg-current spoke"
      style="
        width: {Math.max(1, size * 0.1)}px;
        height: {size * 0.28}px;
        margin-left: {-Math.max(1, size * 0.1) / 2}px;
        margin-top: {-size / 2}px;
        transform-origin: 50% {size / 2}px;
        transform: rotate({(360 / SPOKES) * i}deg);
        animation-delay: {(i / SPOKES) * 0.8}s;
      "
    ></span>
  {/each}
</span>

<style>
  .spoke {
    opacity: 0.18;
    animation: spoke-fade 0.8s linear infinite;
  }

  @keyframes spoke-fade {
    0% {
      opacity: 0.95;
    }
    100% {
      opacity: 0.18;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .spoke {
      animation: none;
      opacity: 0.45;
    }
  }
</style>
