<script lang="ts">
  import { onMount } from "svelte";
  import { Button } from "$lib/components/ui/button";
  import { open as openDialog, save as saveDialog, message } from "@tauri-apps/plugin-dialog";
  import { revealItemInDir } from "@tauri-apps/plugin-opener";
  import { dbPath, exportBackup, restoreBackup } from "$lib/api/backup";

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
      title: "Salvar backup do finan",
      defaultPath: `finan-backup-${new Date().toISOString().slice(0, 10)}.db`,
      filters: [{ name: "SQLite DB", extensions: ["db"] }],
    });
    if (!target) return;

    busy = true;
    try {
      const written = await exportBackup(target);
      info = `Backup salvo em ${written}`;
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
      title: "Restaurar backup do finan",
      multiple: false,
      filters: [{ name: "SQLite DB", extensions: ["db"] }],
    });
    if (!picked || Array.isArray(picked)) return;

    const confirmed = confirm(
      "Isso vai SUBSTITUIR seu banco de dados atual. Você precisa reiniciar o app depois. Continuar?",
    );
    if (!confirmed) return;

    busy = true;
    try {
      await restoreBackup(picked);
      await message(
        "Backup restaurado. Feche e abra o finan de novo (Cmd+Q então abra) pra ver as transações restauradas.",
        { title: "Restauração concluída", kind: "info" },
      );
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
      Configurações
    </h2>
  </header>

  <div class="rounded-xl bg-surface border border-border-subtle p-5 flex flex-col gap-3">
    <div class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint">
      Banco de dados
    </div>
    <p class="text-xs text-fg-muted leading-relaxed">
      Todas as suas transações ficam neste arquivo no seu Mac. Nada é enviado pra nuvem.
    </p>
    <div class="font-mono text-[11.5px] text-fg break-all bg-surface-2 rounded-md p-2 border border-border-subtle">
      {path ?? "Carregando…"}
    </div>
    <div>
      <Button variant="outline" onclick={reveal} disabled={!path}>
        Abrir no Finder
      </Button>
    </div>
  </div>

  <div class="rounded-xl bg-surface border border-border-subtle p-5 flex flex-col gap-3">
    <div class="text-[10.5px] uppercase tracking-wider font-semibold text-fg-faint">
      Backup
    </div>
    <p class="text-xs text-fg-muted leading-relaxed">
      Exporta uma cópia do arquivo pra qualquer pasta. Restaurar SUBSTITUI o banco atual —
      faça backup antes.
    </p>
    <div class="flex gap-2">
      <Button onclick={doExport} disabled={busy}>Exportar backup…</Button>
      <Button variant="outline" onclick={doRestore} disabled={busy}>Restaurar backup…</Button>
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
      Sobre
    </div>
    <div class="text-[12px] text-fg-muted">
      <strong class="text-fg">finan</strong> v0.1.0 — finanças pessoais 100% locais no Mac.
    </div>
    <div class="text-[11px] text-fg-faint">
      Atalhos: <span class="font-mono">⌘1</span> Dashboard ·
      <span class="font-mono">⌘2</span> Transações ·
      <span class="font-mono">⌘3</span> Importar ·
      <span class="font-mono">⌘4</span> Categorias ·
      <span class="font-mono">⌘5</span> Regras ·
      <span class="font-mono">⌘,</span> Configurações ·
      <span class="font-mono">⌘F</span> Buscar ·
      <span class="font-mono">⌘O</span> Abrir OFX
    </div>
  </div>
</section>
