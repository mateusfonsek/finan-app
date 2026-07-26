import { describe, it, expect, vi, beforeEach } from "vitest";

const api = vi.hoisted(() => ({
  scanWatchedFolders: vi.fn(),
  listPendingFiles: vi.fn(),
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
});
