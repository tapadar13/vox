import { Check, Download, HardDrive, Zap } from "lucide-react";
import { useCallback, useEffect, useState } from "react";

import { formatBytes } from "../../lib/format";
import { onModelProgress, vox } from "../../lib/tauri";
import type { ManagedModel, ModelDownloadProgress } from "../../lib/types";
import { Button } from "../../ui/Button";

interface ModelManagerProps {
  selected: string;
  onSelect: (id: string) => void;
}

export function ModelManager({ selected, onSelect }: ModelManagerProps) {
  const [models, setModels] = useState<ManagedModel[]>([]);
  const [downloading, setDownloading] = useState<string | null>(null);
  const [progress, setProgress] = useState<ModelDownloadProgress | null>(null);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => setModels(await vox.models()), []);

  useEffect(() => {
    void refresh();
    let mounted = true;
    let unlisten: () => void = () => undefined;
    void onModelProgress((value) => mounted && setProgress(value)).then((stop) => {
      if (mounted) unlisten = stop;
      else stop();
    });
    return () => {
      mounted = false;
      unlisten();
    };
  }, [refresh]);

  const download = async (id: string) => {
    setError(null);
    setProgress(null);
    setDownloading(id);
    try {
      await vox.downloadModel(id);
      await refresh();
    } catch (caught) {
      setError(typeof caught === "object" && caught && "message" in caught ? String(caught.message) : String(caught));
    } finally {
      setDownloading(null);
      setProgress(null);
    }
  };

  return (
    <div className="space-y-2">
      {models.map((model) => {
        const isSelected = model.id === selected;
        const isDownloading = downloading === model.id;
        return (
          <div
            key={model.id}
            className={`rounded-xl border p-3 transition ${isSelected ? "border-violet-300/25 bg-violet-300/[.07]" : "border-white/[.07] bg-black/10 hover:border-white/12"}`}
          >
            <div className="flex items-center gap-3">
              <button className="min-w-0 flex-1 text-left" type="button" onClick={() => onSelect(model.id)}>
                <span className="flex items-center gap-2 text-[12px] font-medium text-white/82">
                  {model.name}
                  {isSelected && <Check className="size-3 text-violet-300" />}
                </span>
                <span className="mt-1 flex items-center gap-2 text-[9px] text-white/30">
                  <span className="inline-flex items-center gap-1"><HardDrive className="size-2.5" />{formatBytes(model.sizeBytes)}</span>
                  <span>·</span>
                  <span className="inline-flex items-center gap-1"><Zap className="size-2.5" />{model.speed}</span>
                  <span>·</span>
                  <span>{model.accuracy} accuracy</span>
                </span>
              </button>
              {model.installed ? (
                <span className="rounded-full bg-emerald-300/10 px-2 py-1 text-[9px] text-emerald-300/75">Installed</span>
              ) : (
                <Button
                  className="h-7 rounded-lg px-2.5 text-[10px]"
                  icon={<Download className="size-3" />}
                  busy={isDownloading}
                  onClick={() => void download(model.id)}
                >
                  Download
                </Button>
              )}
            </div>
            {isDownloading && progress && (
              <div className="mt-2.5">
                <div className="h-1 overflow-hidden rounded-full bg-white/[.07]">
                  <div
                    className="h-full rounded-full bg-gradient-to-r from-violet-400 to-emerald-300 transition-[width]"
                    style={{ width: `${Math.round(progress.fraction * 100)}%` }}
                  />
                </div>
                <p className="mt-1 text-right text-[9px] tabular-nums text-white/30">
                  {formatBytes(progress.downloadedBytes)} / {formatBytes(progress.totalBytes)}
                </p>
              </div>
            )}
          </div>
        );
      })}
      {error && <p className="rounded-lg bg-rose-400/10 px-3 py-2 text-[10px] text-rose-200/80">{error}</p>}
    </div>
  );
}
