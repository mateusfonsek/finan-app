<script lang="ts">
  import Router, { link } from "svelte-spa-router";
  import { routes } from "./routes/routes";
</script>

<div class="min-h-screen grid grid-cols-[232px_1fr]">
  <aside class="bg-surface border-r border-border-subtle p-3">
    <nav class="flex flex-col gap-1">
      <a use:link href="/dashboard" class="px-2 py-1 rounded-md hover:bg-hover">Dashboard</a>
      <a use:link href="/transactions" class="px-2 py-1 rounded-md hover:bg-hover">Transações</a>
      <a use:link href="/import" class="px-2 py-1 rounded-md hover:bg-hover">Importar OFX</a>
      <a use:link href="/categories" class="px-2 py-1 rounded-md hover:bg-hover">Categorias</a>
      <a use:link href="/onboarding" class="px-2 py-1 rounded-md hover:bg-hover">Onboarding</a>
      <a use:link href="/settings" class="px-2 py-1 rounded-md hover:bg-hover">Configurações</a>
    </nav>
  </aside>
  <main class="bg-bg">
    <Router {routes} />
  </main>
</div>
