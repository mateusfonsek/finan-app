<script lang="ts">
  import { onMount } from "svelte";
  import { healthCheck } from "$lib/api/health";
  import type { HealthInfo } from "$lib/bindings";

  let info = $state<HealthInfo | null>(null);
  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      info = await healthCheck();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  });
</script>

<section class="p-8">
  <h2 class="text-xl font-semibold">Dashboard</h2>
  <p class="text-fg-muted mt-1">KPIs, donut, barras, recent.</p>

  <div class="mt-6 p-4 rounded-lg bg-surface border border-border-subtle max-w-xl">
    <div class="text-[10.5px] font-semibold uppercase tracking-wider text-fg-faint mb-2">
      Health check
    </div>
    {#if error}
      <div class="text-neg text-sm">Erro: {error}</div>
    {:else if info}
      <div class="grid grid-cols-[120px_1fr] gap-y-1 text-[12px]">
        <span class="text-fg-muted">Version</span><span class="tabular">{info.version}</span>
        <span class="text-fg-muted">DB path</span><span class="font-mono text-[11px] break-all">{info.db_path}</span>
        <span class="text-fg-muted">Categories</span><span class="tabular">{info.category_count}</span>
      </div>
    {:else}
      <div class="text-fg-faint text-sm">Carregando…</div>
    {/if}
  </div>
</section>
