<script lang="ts">
  import DiscoveryItem from "./DiscoveryItem.svelte";
  import EnrichmentItem from "./EnrichmentItem.svelte";
  import { activity } from "$lib/stores/activity.svelte";
  import { watch } from "$lib/stores/watch.svelte";

  // Trabalho de fundo empilha aqui. O progresso fica em cima porque é efêmero e
  // sai sozinho; o extrato encontrado fica embaixo, mais perto da mão, porque
  // exige decisão.
  let hasEnrichment = $derived(activity.visible);
  let hasDiscovery = $derived(watch.discoveries.length > 0);
</script>

<!-- Notificação paralela, não modal: sem scrim, porque não interrompe nada — a
     pessoa segue no que estava fazendo. Mesma decisão que o WatchToast já
     tomava; o container só passou a aceitar mais de um item. -->
{#if hasEnrichment || hasDiscovery}
  <div class="fixed bottom-5 right-5 z-40 flex flex-col items-end gap-2">
    {#if hasEnrichment}
      <EnrichmentItem />
    {/if}
    {#if hasDiscovery}
      <DiscoveryItem />
    {/if}
  </div>
{/if}
