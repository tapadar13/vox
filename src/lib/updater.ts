import { relaunch } from "@tauri-apps/plugin-process";
import { check, type Update } from "@tauri-apps/plugin-updater";

import { advanceUpdateProgress, emptyUpdateProgress, type UpdateProgress } from "./updateProgress";

const CHECK_TIMEOUT_MS = 10_000;
const DOWNLOAD_TIMEOUT_MS = 120_000;

export async function checkForUpdate(): Promise<Update | null> {
  return check({ timeout: CHECK_TIMEOUT_MS });
}

export async function installUpdate(
  update: Update,
  onProgress: (progress: UpdateProgress) => void,
): Promise<void> {
  let progress = emptyUpdateProgress;
  await update.downloadAndInstall(
    (event) => {
      progress = advanceUpdateProgress(progress, event);
      onProgress(progress);
    },
    { timeout: DOWNLOAD_TIMEOUT_MS },
  );
  await relaunch();
}
