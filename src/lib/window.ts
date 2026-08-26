import { LogicalSize } from "@tauri-apps/api/dpi";
import { getCurrentWindow } from "@tauri-apps/api/window";

import { isTauri } from "./tauri";

export async function resizeVoxWindow(width: number, height: number): Promise<void> {
  if (!isTauri()) return;
  const appWindow = getCurrentWindow();
  await appWindow.setSize(new LogicalSize(width, height));
  await appWindow.center();
}
