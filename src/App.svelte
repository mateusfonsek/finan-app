<script lang="ts">
  import Router from "svelte-spa-router";
  import Sidebar from "$lib/components/shell/Sidebar.svelte";
  import { routes } from "./routes/routes";
</script>

<div class="min-h-screen grid grid-cols-[232px_1fr]">
  <Sidebar />
  <main class="bg-bg overflow-y-auto">
    <Router {routes} />
  </main>
</div>
