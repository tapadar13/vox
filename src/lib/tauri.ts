import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import {
  defaultSettings,
  emptyStats,
  idleState,
  type AppState,
  type ManagedModel,
  type ModelDownloadProgress,
  type Settings,
  type StatsSnapshot,
  type TranscriptRecord,
} from "./types";

declare global {
  interface Window {
    __TAURI_INTERNALS__?: unknown;
  }
}

export const isTauri = () => Boolean(window.__TAURI_INTERNALS__);

async function command<T>(name: string, args: Record<string, unknown> = {}, fallback: T): Promise<T> {
  if (!isTauri()) return fallback;
  return invoke<T>(name, args);
}

export const vox = {
  state: () => command<AppState>("get_state", {}, idleState),
  toggle: () => command<AppState>("toggle_dictation", {}, idleState),
  cancel: () => command<AppState>("cancel_dictation", {}, idleState),
  retry: () => command<AppState>("retry_dictation", {}, idleState),
  dismiss: () => command<AppState>("dismiss_dictation", {}, idleState),
  settings: () => command<Settings>("get_settings", {}, defaultSettings),
  updateSettings: (settings: Settings) => command<void>("update_settings", { settings }, undefined),
  history: (search?: string, limit = 50, offset = 0) =>
    command<TranscriptRecord[]>("get_history", { search, limit, offset }, []),
  deleteHistory: (id: number) => command<void>("delete_history", { id }, undefined),
  clearHistory: () => command<void>("clear_history", {}, undefined),
  stats: () => command<StatsSnapshot>("get_stats", {}, emptyStats),
  models: () => command<ManagedModel[]>("list_models", {}, []),
  downloadModel: (id: string) => command<string>("download_model", { id }, ""),
  selectModel: (id: string) => command<void>("select_model", { id }, undefined),
  copyText: (text: string) =>
    isTauri() ? command<void>("copy_text", { text }, undefined) : navigator.clipboard.writeText(text),
  showDashboard: () => command<void>("show_dashboard", {}, undefined),
  hideDashboard: () => command<void>("hide_dashboard", {}, undefined),
};

export function onState(handler: (state: AppState) => void): Promise<UnlistenFn> {
  if (!isTauri()) return Promise.resolve(() => undefined);
  return listen<AppState>("vox://state", ({ payload }) => handler(payload));
}

export function onHistoryChanged(handler: () => void): Promise<UnlistenFn> {
  if (!isTauri()) return Promise.resolve(() => undefined);
  return listen("vox://history-changed", handler);
}

export function onModelProgress(
  handler: (progress: ModelDownloadProgress) => void,
): Promise<UnlistenFn> {
  if (!isTauri()) return Promise.resolve(() => undefined);
  return listen<ModelDownloadProgress>("vox://model-progress", ({ payload }) => handler(payload));
}

export function onDashboardNavigation(handler: (page: "home" | "history" | "settings") => void): Promise<UnlistenFn> {
  if (!isTauri()) return Promise.resolve(() => undefined);
  return listen<"home" | "history" | "settings">("vox://navigate", ({ payload }) => handler(payload));
}
