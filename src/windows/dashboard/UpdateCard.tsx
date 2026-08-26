import type { Update } from "@tauri-apps/plugin-updater";
import { CheckCircle2, Download, RefreshCw } from "lucide-react";
import { useEffect, useState } from "react";

import { checkForUpdate, installUpdate } from "../../lib/updater";
import type { UpdateProgress } from "../../lib/updateProgress";
import { Button } from "../../ui/Button";

type Status = "idle" | "checking" | "current" | "available" | "installing";

function messageFrom(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

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
      setError(messageFrom(caught));
    }
  };

  const install = async () => {
    if (!update) return;
    setStatus("installing");
    setError(null);
    setProgress(null);
    try {
      await installUpdate(update, setProgress);
    } catch (caught) {
      setStatus("available");
      setError(messageFrom(caught));
    }
  };

  const detail = status === "current"
    ? "You’re running the latest version."
    : update
      ? `Vox ${update.version} is ready to install.`
      : "Check GitHub Releases for a signed Vox update.";

  return (
    <div className="flex items-center justify-between gap-4 py-2.5">
      <div className="min-w-0">
        <p className="flex items-center gap-1.5 text-[13px] font-medium text-white/88">
          {status === "current" && <CheckCircle2 className="size-3.5 text-emerald-300/70" />}
          Software updates
        </p>
        <p className="mt-0.5 text-[10px] text-white/34">{detail}</p>
        {status === "installing" && (
          <p className="mt-1 text-[10px] text-violet-200/65">
            {progress?.percent === undefined ? "Downloading update…" : `Downloading… ${progress.percent}%`}
          </p>
        )}
        {error && <p className="mt-1 max-w-[430px] truncate text-[10px] text-rose-200/75">{error}</p>}
      </div>
      {status === "available" || status === "installing" ? (
        <Button
          icon={<Download className="size-3.5" />}
          busy={status === "installing"}
          onClick={() => void install()}
        >
          Install & restart
        </Button>
      ) : (
        <Button
          icon={<RefreshCw className="size-3.5" />}
          busy={status === "checking"}
          onClick={() => void checkNow()}
        >
          Check now
        </Button>
      )}
    </div>
  );
}
