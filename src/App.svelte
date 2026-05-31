<script lang="ts">
  import { onMount } from "svelte";
  import Router, { push } from "svelte-spa-router";
  import { listen } from "@tauri-apps/api/event";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import Sidebar from "$lib/components/shell/Sidebar.svelte";
  import AboutDialog from "$lib/components/shell/AboutDialog.svelte";
  import { routes } from "./routes/routes";

  const GITHUB_URL = "https://github.com/MateusFonseK/finan-app";

  let aboutOpen = $state(false);

  const SHORTCUTS: Record<string, () => void> = {
    "1": () => push("/dashboard"),
    "2": () => push("/transactions"),
    "3": () => push("/calendar"),
    "4": () => push("/import"),
    "5": () => push("/categories"),
    "6": () => push("/rules"),
    "7": () => push("/suggestions"),
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

  onMount(() => {
    const unlisten: Array<() => void> = [];
    listen("menu:about", () => (aboutOpen = true)).then((u) => unlisten.push(u));
    listen<string>("menu:navigate", (e) => push(e.payload)).then((u) => unlisten.push(u));
    listen("menu:github", () => void openUrl(GITHUB_URL)).then((u) => unlisten.push(u));
    return () => unlisten.forEach((u) => u());
  });
</script>

<div class="h-screen grid grid-cols-[232px_1fr]">
  <Sidebar onAbout={() => (aboutOpen = true)} />
  <main class="bg-bg overflow-y-auto">
    <Router {routes} />
  </main>
</div>

{#if aboutOpen}
  <AboutDialog onClose={() => (aboutOpen = false)} />
{/if}
