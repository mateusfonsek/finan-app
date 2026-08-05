<script lang="ts">
  import { onMount } from "svelte";
  import { push } from "svelte-spa-router";
  import Loading from "$lib/components/ui/Loading.svelte";
  import { listTransactions } from "$lib/api/transactions";

  let checking = $state(true);

  onMount(async () => {
    try {
      const some = await listTransactions({
        account_id: null,
        month: null,
        category_id: null,
        q: null,
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
  <!-- Rota de decisão: some em milissegundos. Um indicador centrado evita o
       flash de texto solto no canto que existia antes. -->
  <div class="h-full grid place-items-center">
    <Loading />
  </div>
{/if}
