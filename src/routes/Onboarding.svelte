<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { push } from "svelte-spa-router";

  function goToImport() {
    push("/import");
  }
</script>

<section class="p-10 max-w-xl mx-auto flex flex-col gap-6">
  <header class="text-center flex flex-col gap-2">
    <h1 class="text-3xl font-semibold tracking-tight" style="font-family: var(--font-display)">
      Suas finanças, no seu Mac.
    </h1>
    <p class="text-fg-muted text-sm max-w-md mx-auto leading-relaxed">
      Sem nuvem, sem login, sem assinatura. Você arrasta o extrato OFX
      do seu banco e o finan organiza tudo num arquivo SQLite local.
    </p>
  </header>

  <div class="flex justify-center gap-2">
    <Button onclick={goToImport}>Importar meu primeiro OFX</Button>
    <Button variant="outline" onclick={() => push("/dashboard")}>Ver dashboard</Button>
  </div>
</section>
