<script lang="ts">
  import { onMount } from "svelte";
  import Router, { push } from "svelte-spa-router";
  import Sidebar from "$lib/components/shell/Sidebar.svelte";
  import { routes } from "./routes/routes";

  const SHORTCUTS: Record<string, () => void> = {
    "1": () => push("/dashboard"),
    "2": () => push("/transactions"),
    "3": () => push("/import"),
    "4": () => push("/categories"),
    "5": () => push("/rules"),
    ",": () => push("/settings"),
  };

  function onKeydown(e: KeyboardEvent) {
    if (!(e.metaKey || e.ctrlKey)) return;
    const target = e.target as HTMLElement | null;
    const inEditable =
      target !== null &&
      (target.tagName === "INPUT" ||
        target.tagName === "TEXTAREA" ||
        target.isContentEditable);

    const k = e.key.toLowerCase();

    if (k === "f") {
      e.preventDefault();
      const wasOnTransactions = window.location.hash === "#/transactions";
      if (!wasOnTransactions) push("/transactions");
      setTimeout(() => {
        const input = document.querySelector<HTMLInputElement>("[data-search-input]");
        input?.focus();
        input?.select();
      }, 40);
      return;
    }

    if (k === "o") {
      e.preventDefault();
      push("/import");
      return;
    }

    if (inEditable) return;

    if (SHORTCUTS[e.key]) {
      e.preventDefault();
      SHORTCUTS[e.key]();
    }
  }

  onMount(() => {
    window.addEventListener("keydown", onKeydown);
    return () => window.removeEventListener("keydown", onKeydown);
  });
</script>

<div class="min-h-screen grid grid-cols-[232px_1fr]">
  <Sidebar />
  <main class="bg-bg overflow-y-auto">
    <Router {routes} />
  </main>
</div>
