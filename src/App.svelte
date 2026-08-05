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

  // Scroll state of the content pane. Each screen's header is translucent
  // material pinned to the top; the rule separating it from the content only
  // appears when content passes beneath — never as permanent decoration.
  let scroller: HTMLElement | undefined = $state();
  let scrolled = $state(false);

  function onMainScroll() {
    const next = (scroller?.scrollTop ?? 0) > 2;
    if (next !== scrolled) scrolled = next;
  }

  /** Switching screens always starts at the top, like a navigation push. */
  function resetScroll() {
    scroller?.scrollTo({ top: 0 });
    scrolled = false;
  }

  // "Open with finan": drains the .ofx opened via Finder, loads them and hands
  // them to the Import screen through the same stash it already reads.
  async function handleOpenedOfx() {
    const paths = await takePendingOfx();
    for (const path of paths) {
      try {
        await openOfxPath(path);
      } catch {
        // An invalid file opened from Finder: Import shows the error on the
        // next attempt. Not worth interrupting boot.
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
    // Cold start: the file may already be queued before this listener exists.
    void handleOpenedOfx();

    // Scan triggers: app launch and window focus. Focus is what makes a
    // filesystem watcher unnecessary — the user sends the file from their phone
    // and then comes to look at the Mac.
    void watch.loadEnabled().then(() => watch.refresh({ force: true }));
    const onFocus = () => void watch.refresh();
    window.addEventListener("focus", onFocus);
    unlisten.push(() => window.removeEventListener("focus", onFocus));

    return () => unlisten.forEach((u) => u());
  });
</script>

<div class="h-screen grid grid-cols-[236px_1fr] overflow-hidden">
  <Sidebar onAbout={() => (aboutOpen = true)} />
  <!-- `data-scrolled` feeds the sticky header's edge effect: the separating
       rule only exists when content passes beneath it. -->
  <main
    bind:this={scroller}
    onscroll={onMainScroll}
    data-scrolled={scrolled}
    class="bg-bg overflow-y-auto"
  >
    <Router {routes} onRouteLoaded={resetScroll} />
  </main>
</div>

<WatchToast />

<AboutDialog open={aboutOpen} onClose={() => (aboutOpen = false)} />
