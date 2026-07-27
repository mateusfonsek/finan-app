import { describe, it, expect, vi, beforeEach } from "vitest";

const api = vi.hoisted(() => ({
  scanWatchedFolders: vi.fn(),
  markFile: vi.fn(),
  getAppSetting: vi.fn(),
  setAppSetting: vi.fn(),
}));

vi.mock("$lib/api/watch", () => ({
  ...api,
  WATCH_ENABLED_KEY: "watch_enabled",
  WATCH_HINT_DISMISSED_KEY: "watch_hint_dismissed",
}));

const loadOfxFromPath = vi.hoisted(() => vi.fn());
vi.mock("$lib/ofx/load", () => ({ loadOfxFromPath }));

import { OfxReadError } from "$lib/ofx/errors";
import { createWatchStore } from "./watch.svelte";

function discovered(hash: string, name: string) {
  return {
    id: 1,
    content_hash: hash,
    path: `/tmp/${name}`,
    file_name: name,
    size: 100,
    status: "pending",
    seen_at: "2026-07-26",
  };
}

function parsed(txCount: number) {
  return {
    file: new File([""], "x.ofx"),
    parsed: {
      transactions: Array.from({ length: txCount }, () => ({})),
      summary: { earliest: "2026-07-01", latest: "2026-07-31" },
    },
  };
}

describe("watch store", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    api.getAppSetting.mockResolvedValue("1");
  });

  it("expõe descobertas que parseiam, com contagem e período", async () => {
    api.scanWatchedFolders.mockResolvedValue([discovered("h1", "extrato.ofx")]);
    loadOfxFromPath.mockResolvedValue(parsed(87));

    const store = createWatchStore();
    await store.refresh();

    expect(store.pendingCount).toBe(1);
    expect(store.discoveries[0]).toMatchObject({
      hash: "h1",
      fileName: "extrato.ofx",
      txCount: 87,
      earliest: "2026-07-01",
      latest: "2026-07-31",
    });
  });

  it("marca como invalid e não expõe arquivo que não parseia", async () => {
    api.scanWatchedFolders.mockResolvedValue([discovered("h2", "lixo.ofx")]);
    loadOfxFromPath.mockRejectedValue(new Error("not ofx"));

    const store = createWatchStore();
    await store.refresh();

    expect(store.pendingCount).toBe(0);
    expect(api.markFile).toHaveBeenCalledWith("h2", "invalid");
  });

  it("falha de leitura não marca invalid — o arquivo continua pendente", async () => {
    // Stub do iCloud que voltou a ser placeholder, arquivo movido, permissão
    // negada: `invalid` é permanente e enterraria o extrato pra sempre, já que
    // a chave é o hash do conteúdo (spec §5.4).
    api.scanWatchedFolders.mockResolvedValue([discovered("h6", "nubank.ofx")]);
    loadOfxFromPath.mockRejectedValue(
      new OfxReadError("/tmp/nubank.ofx", new Error("No such file or directory")),
    );

    const store = createWatchStore();
    await store.refresh();

    expect(api.markFile).not.toHaveBeenCalled();
    expect(store.pendingCount).toBe(0);
  });

  it("arquivo que não deu pra ler volta a aparecer na varredura seguinte", async () => {
    api.scanWatchedFolders.mockResolvedValue([discovered("h7", "nubank.ofx")]);
    loadOfxFromPath.mockRejectedValueOnce(
      new OfxReadError("/tmp/nubank.ofx", new Error("No such file or directory")),
    );

    const store = createWatchStore();
    await store.refresh();
    expect(store.pendingCount).toBe(0);

    // iCloud terminou o download: a mesma descoberta agora lê e parseia.
    loadOfxFromPath.mockResolvedValue(parsed(12));
    await store.refresh({ force: true });

    expect(store.pendingCount).toBe(1);
    expect(store.discoveries[0]).toMatchObject({ hash: "h7", txCount: 12 });
  });

  it("pedido de abertura é entregue uma vez só", async () => {
    const store = createWatchStore();
    const discovery = {
      hash: "h8",
      path: "/tmp/a.ofx",
      fileName: "a.ofx",
      txCount: 3,
      earliest: null,
      latest: null,
    };

    store.requestOpen(discovery);

    expect(store.openRequest).toEqual(discovery);
    expect(store.takeOpenRequest()).toEqual(discovery);
    // Sem isso, um pedido antigo reabriria um extrato ao voltar pra tela.
    expect(store.takeOpenRequest()).toBeNull();
    expect(store.openRequest).toBeNull();
  });

  it("resolve remove a descoberta da lista", async () => {
    api.scanWatchedFolders.mockResolvedValue([discovered("h3", "a.ofx")]);
    loadOfxFromPath.mockResolvedValue(parsed(10));

    const store = createWatchStore();
    await store.refresh();
    expect(store.pendingCount).toBe(1);

    await store.resolve("h3", "ignored");

    expect(api.markFile).toHaveBeenCalledWith("h3", "ignored");
    expect(store.pendingCount).toBe(0);
  });

  it("não varre de novo dentro da janela de throttle", async () => {
    api.scanWatchedFolders.mockResolvedValue([]);
    const store = createWatchStore();

    await store.refresh();
    await store.refresh();

    expect(api.scanWatchedFolders).toHaveBeenCalledTimes(1);
  });

  it("força a varredura quando pedido explicitamente", async () => {
    api.scanWatchedFolders.mockResolvedValue([]);
    const store = createWatchStore();

    await store.refresh();
    await store.refresh({ force: true });

    expect(api.scanWatchedFolders).toHaveBeenCalledTimes(2);
  });

  it("não varre quando a feature está desligada", async () => {
    api.getAppSetting.mockResolvedValue(null);
    const store = createWatchStore();

    await store.loadEnabled();
    await store.refresh();

    expect(store.enabled).toBe(false);
    expect(api.scanWatchedFolders).not.toHaveBeenCalled();
  });

  it("habilitar persiste e inverte a flag", async () => {
    const store = createWatchStore();

    await store.setEnabled(true);

    expect(api.setAppSetting).toHaveBeenCalledWith("watch_enabled", "1");
    expect(store.enabled).toBe(true);
  });

  it("habilitar força varredura imediata", async () => {
    api.scanWatchedFolders.mockResolvedValue([]);
    const store = createWatchStore();

    await store.setEnabled(true);

    expect(api.scanWatchedFolders).toHaveBeenCalled();
  });

  it("desabilitar persiste \"0\" e desativa a flag", async () => {
    const store = createWatchStore();

    await store.setEnabled(false);

    expect(api.setAppSetting).toHaveBeenCalledWith("watch_enabled", "0");
    expect(store.enabled).toBe(false);
  });

  it("desabilitar limpa a lista de descobertas", async () => {
    api.scanWatchedFolders.mockResolvedValue([discovered("h4", "a.ofx")]);
    loadOfxFromPath.mockResolvedValue(parsed(5));
    const store = createWatchStore();

    await store.refresh();
    expect(store.pendingCount).toBe(1);

    await store.setEnabled(false);

    expect(store.pendingCount).toBe(0);
  });

  it("desabilitar não varre", async () => {
    const store = createWatchStore();

    await store.setEnabled(false);

    expect(api.scanWatchedFolders).not.toHaveBeenCalled();
  });

  it("duas chamadas concorrentes de refresh compartilham uma única leitura de settings", async () => {
    api.scanWatchedFolders.mockResolvedValue([]);
    const store = createWatchStore();

    // Nenhum await entre as duas chamadas — simula App.svelte (boot) e o
    // listener de foco disparando quase ao mesmo tempo.
    await Promise.all([store.refresh(), store.refresh()]);

    expect(api.getAppSetting).toHaveBeenCalledTimes(1);
  });

  it("resolve durante uma varredura em andamento não revive o arquivo já resolvido", async () => {
    let releaseScan!: (files: ReturnType<typeof discovered>[]) => void;
    const scanPromise = new Promise<ReturnType<typeof discovered>[]>((resolve) => {
      releaseScan = resolve;
    });
    api.scanWatchedFolders.mockReturnValue(scanPromise);
    loadOfxFromPath.mockResolvedValue(parsed(3));

    const store = createWatchStore();
    const refreshPromise = store.refresh();

    // Espera a varredura realmente começar (i.e. scanWatchedFolders já foi
    // chamado) antes de resolver o arquivo — só assim o cenário testado
    // (resolve() enquanto o scan está em voo) de fato acontece.
    while (api.scanWatchedFolders.mock.calls.length === 0) {
      await Promise.resolve();
    }

    await store.resolve("h5", "ignored");

    releaseScan([discovered("h5", "b.ofx")]);
    await refreshPromise;

    expect(store.discoveries.find((d) => d.hash === "h5")).toBeUndefined();
    expect(store.pendingCount).toBe(0);
  });
});
