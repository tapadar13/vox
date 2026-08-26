import type { DownloadEvent } from "@tauri-apps/plugin-updater";

export interface UpdateProgress {
  downloadedBytes: number;
  totalBytes?: number;
  percent?: number;
  finished: boolean;
}

export const emptyUpdateProgress: UpdateProgress = {
  downloadedBytes: 0,
  finished: false,
};

export function advanceUpdateProgress(current: UpdateProgress, event: DownloadEvent): UpdateProgress {
  if (event.event === "Started") {
    return {
      downloadedBytes: 0,
      totalBytes: event.data.contentLength,
      percent: event.data.contentLength ? 0 : undefined,
      finished: false,
    };
  }

  if (event.event === "Finished") {
    return { ...current, percent: current.totalBytes ? 100 : current.percent, finished: true };
  }

  const downloadedBytes = current.downloadedBytes + event.data.chunkLength;
  return {
    ...current,
    downloadedBytes,
    percent: current.totalBytes
      ? Math.min(100, Math.round((downloadedBytes / current.totalBytes) * 100))
      : undefined,
  };
}
