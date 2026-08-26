import type { Update } from "@tauri-apps/plugin-updater";
import { useEffect, useState } from "react";

import { checkForUpdate, installUpdate } from "../../lib/updater";
import type { UpdateProgress } from "../../lib/updateProgress";

type Status = "idle" | "checking" | "current" | "available" | "installing";

export function UpdateCard() {
  const [status, setStatus] = useState<Status>("idle");
  const [update, setUpdate] = useState<Update | null>(null);
  const [progress, setProgress] = useState<UpdateProgress | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => () => {
    if (update) void update.close();
  }, [update]);

  const checkNow = async () => {
    setStatus("checking");
    setError(null);
    try {
      const available = await checkForUpdate();
      setUpdate(available);
      setStatus(available ? "available" : "current");
    } catch (caught) {
      setStatus("idle");
      setError(caught instanceof Error ? caught.message : String(caught));
    }
  };

  const install = async () => {
    if (!update) return;
    setStatus("installing");
    setError(null);
    try {
      await installUpdate(update, setProgress);
    } catch (caught) {
      setStatus("available");
      setError(caught instanceof Error ? caught.message : String(caught));
    }
  };

  const label = status === "checking" ? "Checking…"
    : status === "current" ? "v0.1.0 · Up to date"
      : status === "available" ? `Vox ${update?.version} · Install`
        : status === "installing" ? `Installing${progress?.percent === undefined ? "…" : ` · ${progress.percent}%`}`
          : "v0.1.0 · Check now";

  return (
    <div className="flex h-[29px] shrink-0 items-center justify-between">
      <span className="text-[9px] leading-[13px] text-[#3d424e]">Check for updates</span>
      <button
        type="button"
        className={`max-w-[135px] truncate text-[8px] leading-[11px] ${error ? "text-[#b34d66]" : "text-[#768072]"}`}
        aria-label={status === "available" || status === "installing" ? "Install & restart" : "Check now"}
        disabled={status === "checking" || status === "installing"}
        title={error ?? undefined}
        onClick={() => void (status === "available" ? install() : checkNow())}
      >
        {error ? "Update check failed" : label}
      </button>
    </div>
  );
}
