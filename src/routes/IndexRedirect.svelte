<script lang="ts">
  import { onMount } from "svelte";
  import { push } from "svelte-spa-router";
  import { listTransactions } from "$lib/api/transactions";

  let checking = $state(true);

  onMount(async () => {
    try {
      const some = await listTransactions({
        account_id: null,
        month: null,
        category_id: null,
        limit: 1,
      });
      push(some.length === 0 ? "/onboarding" : "/dashboard");
    } catch {
      push("/onboarding");
    } finally {
      checking = false;
    }
  });
</script>

{#if checking}
  <section class="p-8 text-fg-faint text-sm">Carregando…</section>
{/if}
