<script lang="ts">
  import { onMount } from "svelte";
  import { Button } from "$lib/components/ui/button";
  import { open as openDialog, save as saveDialog, message } from "@tauri-apps/plugin-dialog";
  import { revealItemInDir } from "@tauri-apps/plugin-opener";
  import { homeDir, join } from "@tauri-apps/api/path";
  import { dbPath, exportBackup, restoreBackup } from "$lib/api/backup";
  import { locale } from "$lib/i18n/locale.svelte";
  import { watch } from "$lib/stores/watch.svelte";
  import {
    addWatchedFolder,
    dirExists,
    ensureDir,
    getAppSetting,
    ICLOUD_PENDING_KEY,
    LAST_SCAN_KEY,
    listWatchedFolders,
    removeWatchedFolder,
    updateWatchedFolderPath,
  } from "$lib/api/watch";
  import type { WatchedFolder } from "$lib/bindings";

  const t = locale.t;
  const locales = locale.list();

  let path = $state<string | null>(null);
  let busy = $state(false);
  let info = $state<string | null>(null);
  let error = $state<string | null>(null);

  let folders = $state<WatchedFolder[]>([]);
  let menuOpen = $state(false);
  let menuWrapperEl: HTMLDivElement | undefined = $state();
  let icloudWaiting = $state(0);
  let lastScanAt = $state<string | null>(null);

  async function loadFolders() {
    folders = await listWatchedFolders();
    icloudWaiting = Number((await getAppSetting(ICLOUD_PENDING_KEY)) ?? "0");
    lastScanAt = await getAppSetting(LAST_SCAN_KEY);
  }

  /** O carimbo vem do SQLite em UTC e sem sufixo de fuso; o "Z" faz o `Date`
   *  interpretar como UTC pra exibir na hora local de quem está lendo. */
  function formatScan(stamp: string): string {
    const d = new Date(`${stamp.replace(" ", "T")}Z`);
    if (Number.isNaN(d.getTime())) return stamp;
    return d.toLocaleString(locale.dateLocale, { dateStyle: "short", timeStyle: "short" });
  }

  /** O Rust devolve mensagem de desenvolvedor, em inglês, como no resto dos
   *  comandos — nada disso pode ir cru pra tela. Aqui vira texto traduzido, e
   *  o caso que o usuário de fato encontra (apontar pra uma pasta que já está
   *  na lista) ganha frase própria em vez de "UNIQUE constraint failed". */
  function folderError(e: unknown, fallbackKey: string): string {
    const raw = e instanceof Error ? e.message : String(e);
    if (raw.includes("watched_folders.path")) return t("watch.error_duplicate");
    if (raw.includes("failed to resolve path") || raw.includes("is not a directory")) {
      return t("watch.error_unreachable");
    }
    return t(fallbackKey);
  }

  onMount(async () => {
    try {
      path = await dbPath();
      await watch.loadEnabled();
      if (watch.enabled) await loadFolders();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  });

  // Fecha o menu "+ Adicionar pasta" por clique fora ou Esc — mesmo padrão do
  // CategoryPicker: listener único registrado no mount, virando no-op via
  // guarda `!menuOpen` quando o menu já está fechado, e removido no unmount.
  onMount(() => {
    function onDocClick(e: MouseEvent) {
      if (!menuOpen) return;
      const target = e.target as Node | null;
      if (target && !menuWrapperEl?.contains(target)) menuOpen = false;
    }
    function onKeydown(e: KeyboardEvent) {
      if (e.key === "Escape" && menuOpen) menuOpen = false;
    }
    document.addEventListener("mousedown", onDocClick);
    document.addEventListener("keydown", onKeydown);
    return () => {
      document.removeEventListener("mousedown", onDocClick);
      document.removeEventListener("keydown", onKeydown);
    };
  });

  async function reveal() {
    if (!path) return;
    try {
      await revealItemInDir(path);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  /** Todos os presets passam pelo painel do Finder, com `defaultPath` já
   *  apontado. Não é cerimônia: é assim que o consentimento TCC do macOS pra
   *  ~/Downloads e ~/Mesa é concedido pelo caminho natural do sistema, em vez
   *  de um diálogo de permissão surgindo sem contexto no primeiro scan. */
  async function pickFolder(defaultPath?: string): Promise<string | null> {
    const picked = await openDialog({
      title: t("watch.picker_title"),
      directory: true,
      multiple: false,
      defaultPath,
    });
    return typeof picked === "string" ? picked : null;
  }

  async function addFrom(defaultPath?: string) {
    menuOpen = false;
    error = null;
    const picked = await pickFolder(defaultPath);
    if (!picked) return;
    try {
      await addWatchedFolder(picked);
      await loadFolders();
      await watch.refresh({ force: true });
    } catch (e) {
      error = folderError(e, "watch.error_add");
    }
  }

  async function addICloud() {
    menuOpen = false;
    error = null;
    const home = await homeDir();
    const target = await join(home, "Library", "Mobile Documents", "com~apple~CloudDocs", "finan");
    // Criar a pasta é a ÚNICA escrita em disco da feature — por isso pergunta,
    // e só quando ela realmente não existe.
    if (!(await dirExists(target))) {
      if (!confirm(t("watch.create_icloud_confirm"))) return;
      try {
        await ensureDir(target);
      } catch (e) {
        error = folderError(e, "watch.error_create_icloud");
        return;
      }
    }
    await addFrom(target);
  }

  // Os presets `await` o `addFrom` (como o do iCloud já fazia): sem isso, uma
  // falha antes do picker — `homeDir`/`join` — viraria rejeição sem dono e a
  // tela não mostraria nada.
  async function addDownloads() {
    try {
      await addFrom(await join(await homeDir(), "Downloads"));
    } catch (e) {
      error = folderError(e, "watch.error_add");
    }
  }

  async function addDesktop() {
    try {
      await addFrom(await join(await homeDir(), "Desktop"));
    } catch (e) {
      error = folderError(e, "watch.error_add");
    }
  }

  async function enableWatch() {
    error = null;
    const picked = await pickFolder();
    if (!picked) return; // cancelar não ativa nada — sem estado zumbi
    try {
      await addWatchedFolder(picked);
      await watch.setEnabled(true);
      await loadFolders();
    } catch (e) {
      error = folderError(e, "watch.error_toggle");
    }
  }

  async function disableWatch() {
    error = null;
    try {
      await watch.setEnabled(false); // mantém a lista de pastas
    } catch (e) {
      error = folderError(e, "watch.error_toggle");
    }
  }

  async function relocate(folder: WatchedFolder) {
    error = null;
    const picked = await pickFolder();
    if (!picked) return;
    try {
      await updateWatchedFolderPath(folder.id, picked);
      await loadFolders();
      await watch.refresh({ force: true });
    } catch (e) {
      error = folderError(e, "watch.error_relocate");
    }
  }

  async function dropFolder(folder: WatchedFolder) {
    error = null;
    try {
      await removeWatchedFolder(folder.id);
      await loadFolders();
    } catch (e) {
      error = folderError(e, "watch.error_remove");
    }
  }

  /** Abrir no Finder pode falhar (pasta desmontada entre o render e o clique)
   *  — mesmo cuidado que o botão do banco de dados já tinha. */
  async function revealFolder(folder: WatchedFolder) {
    error = null;
    try {
      await revealItemInDir(folder.path);
    } catch (e) {
      error = folderError(e, "watch.error_reveal");
    }
  }

  // Varredura manual — o terceiro gatilho previsto na spec §5.1, ao lado da
  // abertura do app e do foco da janela.
  async function scanNow() {
    error = null;
    try {
      await watch.refresh({ force: true });
      await loadFolders();
    } catch (e) {
      error = folderError(e, "watch.error_scan");
    }
  }

  async function doExport() {
    info = null;
    error = null;
    const target = await saveDialog({
      title: t("settings.save_title"),
      defaultPath: `finan-backup-${new Date().toISOString().slice(0, 10)}.db`,
      filters: [{ name: "SQLite DB", extensions: ["db"] }],
    });
    if (!target) return;

    busy = true;
    try {
      const written = await exportBackup(target);
      info = t("settings.backup_saved", { path: written });
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }

  async function doRestore() {
    info = null;
    error = null;
    const picked = await openDialog({
      title: t("settings.restore_title"),
      multiple: false,
      filters: [{ name: "SQLite DB", extensions: ["db"] }],
    });
    if (!picked || Array.isArray(picked)) return;

    const confirmed = confirm(t("settings.restore_confirm"));
    if (!confirmed) return;

    busy = true;
    try {
      await restoreBackup(picked);
      await message(t("settings.restore_done_msg"), {
        title: t("settings.restore_done_title"),
        kind: "info",
      });
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }
</script>

<section class="p-8 max-w-3xl mx-auto flex flex-col gap-6">
  <header>
    <h2 class="text-xl font-semibold tracking-tight" style="font-family: var(--font-display)">
      {t("settings.title")}
    </h2>
  </header>

  <div class="rounded-xl bg-surface border border-border-subtle p-5 flex flex-col gap-3">
    <div class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint">
      {t("settings.language")}
    </div>
    <p class="text-xs text-fg-muted leading-relaxed">
      {t("settings.language_desc")}
    </p>
    <div class="flex flex-wrap gap-2">
      {#each locales as l}
        <button
          type="button"
          onclick={() => locale.set(l.code)}
          class="flex items-center gap-2 px-3 py-1.5 rounded-md text-[12.5px] font-medium border transition-colors
                 {locale.code === l.code
                   ? 'bg-accent-soft text-fg border-accent'
                   : 'text-fg-muted border-border hover:bg-hover hover:text-fg'}"
          aria-pressed={locale.code === l.code}
        >
          {#if l.flag}<span>{l.flag}</span>{/if}
          <span>{l.name}</span>
        </button>
      {/each}
    </div>
  </div>

  <div class="rounded-xl bg-surface border border-border-subtle p-5 flex flex-col gap-3">
    <div class="flex items-center justify-between">
      <div class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint">
        {t("watch.section_title")}
      </div>
      {#if watch.enabled}
        <button
          type="button"
          onclick={disableWatch}
          class="text-[11px] text-fg-muted hover:text-fg transition-colors"
        >
          {t("watch.disable")}
        </button>
      {/if}
    </div>

    {#if !watch.enabled}
      <div class="text-[13px] font-medium text-fg">{t("watch.pitch_title")}</div>
      <p class="text-xs text-fg-muted leading-relaxed">{t("watch.pitch_body")}</p>
      <p class="text-[11px] text-fg-faint leading-relaxed">🔒 {t("watch.privacy_note")}</p>
      <div>
        <Button onclick={enableWatch}>{t("watch.enable_cta")}</Button>
      </div>
    {:else}
      <div class="text-[10px] uppercase tracking-wider text-fg-faint">
        {t("watch.folders_label")}
      </div>
      <div class="rounded-lg border border-border-subtle divide-y divide-border-subtle">
        {#each folders as f (f.id)}
          <div class="p-3 flex items-start gap-3">
            <span class="text-base leading-none pt-0.5">{f.exists ? "📁" : "⚠️"}</span>
            <div class="flex-1 min-w-0">
              <div class="text-[12.5px] font-medium text-fg">{f.label}</div>
              <div class="text-[10.5px] text-fg-faint truncate">{f.path}</div>
              {#if !f.exists}
                <div class="text-[11px] text-neg mt-1">{t("watch.folder_missing")}</div>
                <div class="flex gap-2 mt-1.5">
                  <Button variant="outline" onclick={() => relocate(f)}>
                    {t("watch.folder_relocate")}
                  </Button>
                  <Button variant="ghost" onclick={() => dropFolder(f)}>
                    {t("watch.folder_remove")}
                  </Button>
                </div>
              {:else}
                <div class="text-[10.5px] text-fg-muted mt-0.5">
                  {#if f.imported_count === 0}
                    {t("watch.imported_none")}
                  {:else}
                    {f.imported_count === 1
                      ? t("watch.imported_one", { n: f.imported_count })
                      : t("watch.imported_many", { n: f.imported_count })}
                    {#if f.last_imported_at}
                      · {t("watch.imported_last", { date: f.last_imported_at.slice(0, 10) })}
                    {/if}
                  {/if}
                </div>
              {/if}
            </div>
            {#if f.exists}
              <div class="flex gap-1 shrink-0">
                <button
                  type="button"
                  onclick={() => revealFolder(f)}
                  title={t("watch.folder_menu_reveal")}
                  class="text-[11px] text-fg-muted hover:text-fg px-1.5 py-0.5 rounded hover:bg-hover"
                >
                  ↗
                </button>
                <button
                  type="button"
                  onclick={() => dropFolder(f)}
                  title={t("watch.folder_menu_remove")}
                  class="text-[11px] text-fg-muted hover:text-neg px-1.5 py-0.5 rounded hover:bg-hover"
                >
                  ×
                </button>
              </div>
            {/if}
          </div>
        {/each}
      </div>

      <div class="relative" bind:this={menuWrapperEl}>
        <Button variant="outline" onclick={() => (menuOpen = !menuOpen)}>
          + {t("watch.add_folder")} ▾
        </Button>
        {#if menuOpen}
          <div class="absolute z-10 mt-1 w-64 rounded-lg border border-border bg-surface shadow-lg py-1 text-[12.5px]">
            <button type="button" onclick={addICloud}
              class="w-full text-left px-3 py-1.5 hover:bg-hover text-fg">
              ☁️ {t("watch.preset_icloud")}
            </button>
            <button type="button" onclick={addDownloads}
              class="w-full text-left px-3 py-1.5 hover:bg-hover text-fg flex justify-between">
              <span>⬇️ {t("watch.preset_downloads")}</span>
              <span class="text-fg-faint text-[10.5px]">{t("watch.preset_downloads_hint")}</span>
            </button>
            <button type="button" onclick={addDesktop}
              class="w-full text-left px-3 py-1.5 hover:bg-hover text-fg">
              🖥️ {t("watch.preset_desktop")}
            </button>
            <div class="border-t border-border-subtle my-1"></div>
            <button type="button" onclick={() => addFrom()}
              class="w-full text-left px-3 py-1.5 hover:bg-hover text-fg">
              📁 {t("watch.preset_other")}
            </button>
          </div>
        {/if}
      </div>

      {#if icloudWaiting > 0}
        <div class="text-[11px] text-fg-faint">
          ☁️ {icloudWaiting === 1
            ? t("watch.icloud_waiting_one", { n: icloudWaiting })
            : t("watch.icloud_waiting_many", { n: icloudWaiting })}
        </div>
      {/if}

      <div class="flex items-center justify-between pt-1 border-t border-border-subtle">
        <!-- Evidência, não status genérico (spec §4.2): a hora real da última
             varredura, ou o reconhecimento de que ainda não houve nenhuma. -->
        <span class="text-[10.5px] text-fg-faint">
          {lastScanAt
            ? t("watch.last_scan_at", { time: formatScan(lastScanAt) })
            : t("watch.last_scan_never")}
        </span>
        <button
          type="button"
          onclick={scanNow}
          class="text-[11px] text-fg-muted hover:text-fg transition-colors"
        >
          {t("watch.folder_menu_scan")}
        </button>
      </div>
    {/if}
  </div>

  <div class="rounded-xl bg-surface border border-border-subtle p-5 flex flex-col gap-3">
    <div class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint">
      {t("settings.database")}
    </div>
    <p class="text-xs text-fg-muted leading-relaxed">
      {t("settings.database_desc")}
    </p>
    <div class="font-mono text-[11.5px] text-fg break-all bg-surface-2 rounded-md p-2 border border-border-subtle">
      {path ?? t("common.loading")}
    </div>
    <div>
      <Button variant="outline" onclick={reveal} disabled={!path}>
        {t("settings.open_in_finder")}
      </Button>
    </div>
  </div>

  <div class="rounded-xl bg-surface border border-border-subtle p-5 flex flex-col gap-3">
    <div class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint">
      {t("settings.backup")}
    </div>
    <p class="text-xs text-fg-muted leading-relaxed">
      {t("settings.backup_desc")}
    </p>
    <div class="flex gap-2">
      <Button onclick={doExport} disabled={busy}>{t("settings.export")}</Button>
      <Button variant="outline" onclick={doRestore} disabled={busy}>{t("settings.restore")}</Button>
    </div>
    {#if info}
      <div class="text-[11.5px] text-pos">{info}</div>
    {/if}
    {#if error}
      <div class="text-[11.5px] text-neg">{error}</div>
    {/if}
  </div>

  <div class="rounded-xl bg-surface border border-border-subtle p-5 flex flex-col gap-2">
    <div class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint">
      {t("settings.about")}
    </div>
    <div class="text-[12px] text-fg-muted">
      <strong class="text-fg">finan app</strong> v0.2.0 — {t("settings.about_line")}
    </div>
    <div class="text-[11px] text-fg-faint">
      {t("settings.shortcuts_label")}
      <span class="font-mono">⌘1</span> {t("nav.dashboard")} ·
      <span class="font-mono">⌘2</span> {t("nav.transactions")} ·
      <span class="font-mono">⌘3</span> {t("nav.calendar")} ·
      <span class="font-mono">⌘4</span> {t("sidebar.import")} ·
      <span class="font-mono">⌘5</span> {t("nav.categories")} ·
      <span class="font-mono">⌘6</span> {t("nav.rules")} ·
      <span class="font-mono">⌘7</span> {t("nav.suggestions")} ·
      <span class="font-mono">⌘F</span> {t("settings.shortcut_search")} ·
      <span class="font-mono">⌘O</span> {t("settings.shortcut_open_ofx")}
    </div>
  </div>
</section>
