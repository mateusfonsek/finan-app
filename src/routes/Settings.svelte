<script lang="ts">
  import { onMount } from "svelte";
  import { Button } from "$lib/components/ui/button";
  import { open as openDialog, save as saveDialog, message } from "@tauri-apps/plugin-dialog";
  import { revealItemInDir } from "@tauri-apps/plugin-opener";
  import { dbPath, exportBackup, restoreBackup } from "$lib/api/backup";
  import { locale } from "$lib/i18n/locale.svelte";

  const t = locale.t;
  const locales = locale.list();

  let path = $state<string | null>(null);
  let busy = $state(false);
  let info = $state<string | null>(null);
  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      path = await dbPath();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  });

  async function reveal() {
    if (!path) return;
    try {
      await revealItemInDir(path);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
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
      <strong class="text-fg">finan app</strong> v0.1.0 — {t("settings.about_line")}
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
