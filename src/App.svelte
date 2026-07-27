<script lang="ts">
  import { onMount } from "svelte";
  import Router, { push } from "svelte-spa-router";
  import { listen } from "@tauri-apps/api/event";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import Sidebar from "$lib/components/shell/Sidebar.svelte";
  import AboutDialog from "$lib/components/shell/AboutDialog.svelte";
  import WatchToast from "$lib/components/shell/WatchToast.svelte";
  import { openOfxPath } from "$lib/ofx/open";
  import { takePendingOfx } from "$lib/api/files";
  import { locale } from "$lib/i18n/locale.svelte";
  import { watch } from "$lib/stores/watch.svelte";
  import { routes } from "./routes/routes";

  // Sync the UI language from the backend's persisted choice on boot.
  void locale.init();

  const GITHUB_URL = "https://github.com/MateusFonseK/finan-app";

  let aboutOpen = $state(false);

  // "Abrir com finan": drena os .ofx abertos via Finder, carrega e entrega pra
  // tela de Importar via o mesmo stash que ela já lê no onMount.
  async function handleOpenedOfx() {
    const paths = await takePendingOfx();
    for (const path of paths) {
      try {
        await openOfxPath(path);
      } catch {
        // Arquivo inválido aberto pelo Finder: o Import mostra o erro quando
        // o usuário tenta de novo. Não vale interromper o boot por isso.
      }
    }
  }

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
    listen("open-ofx", () => void handleOpenedOfx()).then((u) => unlisten.push(u));
    // Cold start: o arquivo pode já estar na fila antes deste listener existir.
    void handleOpenedOfx();

    // Gatilhos de varredura: abertura do app e foco da janela. É o foco que
    // torna o watcher de filesystem desnecessário — o usuário manda o arquivo
    // do celular e então vem olhar o Mac.
    void watch.loadEnabled().then(() => watch.refresh({ force: true }));
    const onFocus = () => void watch.refresh();
    window.addEventListener("focus", onFocus);
    unlisten.push(() => window.removeEventListener("focus", onFocus));

    return () => unlisten.forEach((u) => u());
  });
</script>

<div class="h-screen grid grid-cols-[232px_1fr]">
  <Sidebar onAbout={() => (aboutOpen = true)} />
  <main class="bg-bg overflow-y-auto">
    <Router {routes} />
  </main>
</div>

<WatchToast />

{#if aboutOpen}
  <AboutDialog onClose={() => (aboutOpen = false)} />
{/if}
