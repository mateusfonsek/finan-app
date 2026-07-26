import { commands } from "../bindings";
import type { DiscoveredFile, WatchedFolder } from "$lib/bindings";

function unwrap<T>(result: { status: "ok"; data: T } | { status: "error"; error: string }): T {
  if (result.status === "error") throw new Error(result.error);
  return result.data;
}

export type FileStatus = "pending" | "imported" | "ignored" | "invalid";

export const WATCH_ENABLED_KEY = "watch_enabled";
export const WATCH_HINT_DISMISSED_KEY = "watch_hint_dismissed";
export const ICLOUD_PENDING_KEY = "watch_icloud_pending";

/** Cria a pasta do preset do iCloud. Única escrita em disco da feature. */
export async function ensureDir(path: string): Promise<void> {
  unwrap(await commands.ensureDir(path));
}

export async function dirExists(path: string): Promise<boolean> {
  return commands.dirExists(path);
}

export async function listWatchedFolders(): Promise<WatchedFolder[]> {
  return unwrap(await commands.listWatchedFolders());
}

export async function addWatchedFolder(path: string): Promise<WatchedFolder> {
  return unwrap(await commands.addWatchedFolder(path));
}

export async function updateWatchedFolderPath(id: number, path: string): Promise<WatchedFolder> {
  return unwrap(await commands.updateWatchedFolderPath(id, path));
}

export async function removeWatchedFolder(id: number): Promise<void> {
  unwrap(await commands.removeWatchedFolder(id));
}

export async function scanWatchedFolders(): Promise<DiscoveredFile[]> {
  return unwrap(await commands.scanWatchedFolders());
}

export async function listPendingFiles(): Promise<DiscoveredFile[]> {
  return unwrap(await commands.listPendingFiles());
}

export async function markFile(contentHash: string, status: FileStatus): Promise<void> {
  unwrap(await commands.markFile(contentHash, status));
}

export async function getAppSetting(key: string): Promise<string | null> {
  return unwrap(await commands.getAppSetting(key));
}

export async function setAppSetting(key: string, value: string): Promise<void> {
  unwrap(await commands.setAppSetting(key, value));
}
