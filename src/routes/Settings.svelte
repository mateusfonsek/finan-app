<script lang="ts">
  import { onMount } from "svelte";
  import { Button } from "$lib/components/ui/button";
  import Page from "$lib/components/ui/Page.svelte";
  import Card from "$lib/components/ui/Card.svelte";
  import Icon from "$lib/components/ui/Icon.svelte";
  import Switch from "$lib/components/ui/Switch.svelte";
  import ErrorNote from "$lib/components/ui/ErrorNote.svelte";
  import { popover } from "$lib/motion";
  import { healthCheck } from "$lib/api/health";
  import {
    confirm,
    open as openDialog,
    save as saveDialog,
    message,
  } from "@tauri-apps/plugin-dialog";
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
  import { enrichmentStatus, setEnrichmentEnabled } from "$lib/api/enrichment";
  import type { EnrichmentStatus, WatchedFolder } from "$lib/bindings";

  const t = locale.t;
  const locales = locale.list();

  let path = $state<string | null>(null);
  let busy = $state(false);
  let info = $state<string | null>(null);
  let error = $state<string | null>(null);

  /** `null` while loading. Availability comes from the backend, never from a
   *  language comparison here. */
  let enrich = $state<EnrichmentStatus | null>(null);

  async function toggleEnrich(value: boolean) {
    await setEnrichmentEnabled(value);
    enrich = await enrichmentStatus();
  }

  let folders = $state<WatchedFolder[]>([]);
  let menuOpen = $state(false);
  let menuWrapperEl: HTMLDivElement | undefined = $state();
  let icloudWaiting = $state(0);
  let lastScanAt = $state<string | null>(null);
  /** A versão vem do binário — literal no template envelhece sem avisar. */
  let version = $state<string | null>(null);

  async function loadEnrich() {
    try {
      enrich = await enrichmentStatus();
    } catch {
      // No status, no setting — better hidden than showing a switch that does
      // not know its own state.
      enrich = null;
    }
  }

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
      void healthCheck().then((h) => (version = h.version)).catch(() => {});
      await loadEnrich();
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
      const ok = await confirm(t("watch.create_icloud_confirm"), {
        title: t("watch.preset_icloud"),
        okLabel: t("common.create"),
        cancelLabel: t("common.cancel"),
      });
      if (!ok) return;
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

    // Restaurar SUBSTITUI o banco — destrutivo e irreversível, então é um dos
    // poucos lugares que merece um alerta nativo bloqueando o caminho.
    const confirmed = await confirm(t("settings.restore_confirm"), {
      title: t("settings.restore_title"),
      kind: "warning",
      okLabel: t("settings.restore_ok"),
      cancelLabel: t("common.cancel"),
    });
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

<Page title={t("settings.title")} width="narrow">
  <!-- ── Idioma ─────────────────────────────────────────────────────────── -->
  <Card title={t("settings.language")}>
    <p class="text-sub text-fg-muted leading-relaxed">{t("settings.language_desc")}</p>
    <div class="flex flex-wrap gap-2">
      {#each locales as l}
        {@const active = locale.code === l.code}
        <button
          type="button"
          onclick={() => locale.set(l.code)}
          class="press flex items-center gap-2 h-7 px-3 rounded-[var(--radius-md)] text-callout font-medium
                 border transition-colors duration-[var(--dur-fast)]
                 {active
            ? 'bg-accent text-accent-on border-transparent'
            : 'text-fg-muted border-border bg-surface-2 hover:bg-hover hover:text-fg'}"
          aria-pressed={active}
        >
          {#if l.flag}<span>{l.flag}</span>{/if}
          <span>{l.name}</span>
          {#if active}<Icon name="check" size={12} stroke={2.6} />{/if}
        </button>
      {/each}
    </div>
  </Card>

  <!-- ── Tax-id company lookup ──────────────────────────────────────────── -->
  <!-- Only exists when the locale pack declares both a format and a provider. -->
  {#if enrich?.available}
    <Card title={t("settings.enrich_title", { taxId: enrich.tax_id_name })}>
      <div class="flex items-start justify-between gap-4">
        <div class="flex flex-col gap-1.5 min-w-0">
          <p class="text-sub text-fg-muted leading-relaxed">
            {t("settings.enrich_desc", { taxId: enrich.tax_id_name })}
          </p>
          <!-- Who answers the lookup is visible before opting in. -->
          <p class="text-foot text-fg-subtle flex items-start gap-1.5">
            <span class="mt-px"><Icon name="cloud" size={11.5} /></span>
            <span>{t("settings.enrich_provider", { provider: enrich.provider })}</span>
          </p>
        </div>
        <div class="flex items-center gap-2.5 shrink-0">
          <span class="text-foot text-fg-subtle">
            {enrich.enabled ? t("settings.enrich_on") : t("settings.enrich_off")}
          </span>
          <Switch
            checked={enrich.enabled}
            onChange={toggleEnrich}
            label={t("settings.enrich_title", { taxId: enrich.tax_id_name })}
          />
        </div>
      </div>
    </Card>
  {/if}

  <!-- ── Importação automática ──────────────────────────────────────────── -->
  <Card title={t("watch.section_title")}>
    {#snippet actions()}
      {#if watch.enabled}
        <button
          type="button"
          onclick={disableWatch}
          class="text-foot text-fg-subtle hover:text-fg transition-colors duration-[var(--dur-fast)] shrink-0"
        >
          {t("watch.disable")}
        </button>
      {/if}
    {/snippet}

    {#if !watch.enabled}
      <div class="flex flex-col gap-2">
        <div class="text-title3 font-semibold text-fg">{t("watch.pitch_title")}</div>
        <p class="text-sub text-fg-muted leading-relaxed">{t("watch.pitch_body")}</p>
        <p class="text-foot text-fg-subtle leading-relaxed flex items-start gap-1.5 pt-1">
          <span class="mt-px"><Icon name="lock" size={11.5} /></span>
          <span>{t("watch.privacy_note")}</span>
        </p>
      </div>
      <div>
        <Button onclick={enableWatch}>{t("watch.enable_cta")}</Button>
      </div>
    {:else}
      <div class="text-foot text-fg-subtle">{t("watch.folders_label")}</div>
      <div class="card-inset divide-y divide-border-subtle overflow-hidden">
        {#each folders as f (f.id)}
          <div class="group p-3 flex items-start gap-2.5">
            <span
              class="w-6 h-6 shrink-0 grid place-items-center rounded-[var(--radius-sm)] mt-px
                     {f.exists ? 'bg-surface-2 text-fg-subtle' : 'bg-neg/12 text-neg'}"
            >
              <Icon name={f.exists ? "folder" : "triangleAlert"} size={13} />
            </span>
            <div class="flex-1 min-w-0">
              <div class="text-callout font-medium text-fg">{f.label}</div>
              <div class="text-cap text-fg-subtle truncate font-mono" title={f.path}>{f.path}</div>
              {#if !f.exists}
                <div class="text-foot text-neg mt-1">{t("watch.folder_missing")}</div>
                <div class="flex gap-2 mt-1.5">
                  <Button variant="outline" size="sm" onclick={() => relocate(f)}>
                    {t("watch.folder_relocate")}
                  </Button>
                  <Button variant="ghost" size="sm" onclick={() => dropFolder(f)}>
                    {t("watch.folder_remove")}
                  </Button>
                </div>
              {:else}
                <div class="text-cap text-fg-muted mt-0.5">
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
              <div
                class="flex gap-1 shrink-0 opacity-0 group-hover:opacity-100 focus-within:opacity-100
                       transition-opacity duration-[var(--dur-fast)]"
              >
                <button
                  type="button"
                  onclick={() => revealFolder(f)}
                  title={t("watch.folder_menu_reveal")}
                  aria-label={t("watch.folder_menu_reveal")}
                  class="press w-6 h-6 grid place-items-center rounded-[var(--radius-sm)] text-fg-muted
                         hover:bg-hover hover:text-fg transition-colors duration-[var(--dur-fast)]"
                >
                  <Icon name="externalLink" size={12} />
                </button>
                <button
                  type="button"
                  onclick={() => dropFolder(f)}
                  title={t("watch.folder_menu_remove")}
                  aria-label={t("watch.folder_menu_remove")}
                  class="press w-6 h-6 grid place-items-center rounded-[var(--radius-sm)] text-fg-muted
                         hover:bg-neg/12 hover:text-neg transition-colors duration-[var(--dur-fast)]"
                >
                  <Icon name="x" size={12} stroke={2.2} />
                </button>
              </div>
            {/if}
          </div>
        {/each}
      </div>

      <div class="relative" bind:this={menuWrapperEl}>
        <Button variant="outline" onclick={() => (menuOpen = !menuOpen)} aria-expanded={menuOpen}>
          <Icon name="plus" size={12} stroke={2.4} />
          {t("watch.add_folder")}
          <Icon name="chevronDown" size={11} stroke={2.2} class="opacity-60" />
        </Button>
        {#if menuOpen}
          <div
            transition:popover={{ origin: "top left" }}
            class="material-pop absolute z-10 mt-1.5 w-72 p-1 text-callout"
            role="menu"
          >
            {#each [{ icon: "cloud" as const, label: t("watch.preset_icloud"), hint: "", fn: addICloud }, { icon: "download" as const, label: t("watch.preset_downloads"), hint: t("watch.preset_downloads_hint"), fn: addDownloads }, { icon: "monitor" as const, label: t("watch.preset_desktop"), hint: "", fn: addDesktop }] as item}
              <button
                type="button"
                role="menuitem"
                onclick={item.fn}
                class="w-full text-left px-2 h-7 rounded-[var(--radius-sm)] flex items-center gap-2
                       text-fg-muted hover:bg-accent hover:text-accent-on
                       transition-colors duration-[var(--dur-instant)]"
              >
                <Icon name={item.icon} size={13} />
                <span class="flex-1 truncate">{item.label}</span>
                {#if item.hint}
                  <span class="text-cap opacity-70">{item.hint}</span>
                {/if}
              </button>
            {/each}
            <div class="hairline my-1"></div>
            <button
              type="button"
              role="menuitem"
              onclick={() => addFrom()}
              class="w-full text-left px-2 h-7 rounded-[var(--radius-sm)] flex items-center gap-2
                     text-fg-muted hover:bg-accent hover:text-accent-on
                     transition-colors duration-[var(--dur-instant)]"
            >
              <Icon name="folder" size={13} />
              <span class="flex-1 truncate">{t("watch.preset_other")}</span>
            </button>
          </div>
        {/if}
      </div>

      {#if icloudWaiting > 0}
        <div class="text-foot text-fg-subtle flex items-center gap-1.5">
          <Icon name="cloud" size={12} />
          {icloudWaiting === 1
            ? t("watch.icloud_waiting_one", { n: icloudWaiting })
            : t("watch.icloud_waiting_many", { n: icloudWaiting })}
        </div>
      {/if}

      <div class="flex items-center justify-between pt-2.5 border-t border-border-subtle">
        <!-- Evidência, não status genérico (spec §4.2): a hora real da última
             varredura, ou o reconhecimento de que ainda não houve nenhuma. -->
        <span class="text-cap text-fg-subtle tabular">
          {lastScanAt
            ? t("watch.last_scan_at", { time: formatScan(lastScanAt) })
            : t("watch.last_scan_never")}
        </span>
        <Button variant="ghost" size="sm" onclick={scanNow}>
          <Icon name="rotateCw" size={11.5} />
          {t("watch.folder_menu_scan")}
        </Button>
      </div>
    {/if}
  </Card>

  <!-- ── Banco de dados ─────────────────────────────────────────────────── -->
  <Card title={t("settings.database")}>
    <p class="text-sub text-fg-muted leading-relaxed">{t("settings.database_desc")}</p>
    <div class="card-inset font-mono text-foot text-fg break-all p-2.5 selectable">
      {path ?? t("common.loading")}
    </div>
    <div>
      <Button variant="outline" onclick={reveal} disabled={!path}>
        <Icon name="externalLink" size={12} />
        {t("settings.open_in_finder")}
      </Button>
    </div>
  </Card>

  <!-- ── Backup ─────────────────────────────────────────────────────────── -->
  <Card title={t("settings.backup")}>
    <p class="text-sub text-fg-muted leading-relaxed">{t("settings.backup_desc")}</p>
    <div class="flex gap-2">
      <Button onclick={doExport} disabled={busy}>
        <Icon name="upload" size={12} />
        {t("settings.export")}
      </Button>
      <Button variant="outline" onclick={doRestore} disabled={busy}>
        <Icon name="download" size={12} />
        {t("settings.restore")}
      </Button>
    </div>
    {#if info}
      <ErrorNote message={info} tone="success" />
    {/if}
    {#if error}
      <ErrorNote message={error} />
    {/if}
  </Card>

  <!-- ── Sobre + atalhos ────────────────────────────────────────────────── -->
  <Card title={t("settings.about")}>
    <div class="text-sub text-fg-muted">
      <strong class="text-fg font-semibold">finan app</strong>
      <span class="tabular">{version ? `v${version}` : ""}</span>
      — {t("settings.about_line")}
    </div>

    <!-- Atalhos numa grade: procurar um atalho numa frase corrida é trabalho. -->
    <div class="grid grid-cols-3 gap-x-5 gap-y-1.5 pt-1">
      {#each [["⌘1", t("nav.dashboard")], ["⌘2", t("nav.transactions")], ["⌘3", t("nav.calendar")], ["⌘4", t("sidebar.import")], ["⌘5", t("nav.categories")], ["⌘6", t("nav.rules")], ["⌘7", t("nav.suggestions")], ["⌘F", t("settings.shortcut_search")], ["⌘O", t("settings.shortcut_open_ofx")]] as [key, label]}
        <div class="flex items-center gap-2 min-w-0">
          <kbd
            class="shrink-0 min-w-[26px] h-5 px-1.5 grid place-items-center rounded-[5px] border border-border
                   bg-surface-2 text-cap font-mono text-fg-muted"
          >
            {key}
          </kbd>
          <span class="text-foot text-fg-subtle truncate">{label}</span>
        </div>
      {/each}
    </div>
  </Card>
</Page>
