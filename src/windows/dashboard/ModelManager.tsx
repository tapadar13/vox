import { useCallback, useEffect, useMemo, useState } from "react";

import { formatBytes } from "../../lib/format";
import { isTauri, onModelProgress, vox } from "../../lib/tauri";
import type { ManagedModel, ModelDownloadProgress } from "../../lib/types";

interface ModelManagerProps {
  selected: string;
  onSelect: (id: string) => void;
}

const previewModels: ManagedModel[] = [
  { id: "whisper-large-v3-turbo-q5_0", name: "Turbo · Best quality", filename: "", sizeBytes: 600_000_000, sha256: "", url: "", speed: "Fastest quality", accuracy: "Best", multilingual: true, installed: true, active: true },
  { id: "whisper-small-q5_1", name: "Small · Balanced", filename: "", sizeBytes: 190_000_000, sha256: "", url: "", speed: "Ultra-fast", accuracy: "Good", multilingual: true, installed: false, active: false },
];

export function ModelManager({ selected, onSelect }: ModelManagerProps) {
  const [models, setModels] = useState<ManagedModel[]>(isTauri() ? [] : previewModels);
  const [downloading, setDownloading] = useState<string | null>(null);
  const [progress, setProgress] = useState<ModelDownloadProgress | null>(null);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    const next = await vox.models();
    if (next.length > 0) setModels(next);
  }, []);

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

  const active = useMemo(() => models.find((model) => model.id === selected) ?? models[0], [models, selected]);
  const alternative = useMemo(() => models.find((model) => model.id !== active?.id), [active?.id, models]);

  const download = async (model: ManagedModel) => {
    setDownloading(model.id);
    setError(null);
    try {
      await vox.downloadModel(model.id);
      await refresh();
      onSelect(model.id);
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : String(caught));
    } finally {
      setDownloading(null);
      setProgress(null);
    }
  };

  return (
    <div className="flex h-full flex-col gap-2">
      <header className="flex h-[22px] shrink-0 items-center justify-between">
        <h2 className="text-[11px] font-bold leading-[15px] tracking-[.09em] text-[#606775]">MODEL &amp; ENGINE</h2>
        <span className="text-[8px] leading-[11px] text-[#818895]">On-device</span>
      </header>

      {active && (
        <button type="button" className="flex h-28 w-full shrink-0 flex-col gap-2 rounded-[14px] bg-white/85 p-3 text-left shadow-[inset_0_0_0_1px_rgba(174,86,206,.14),0_8px_20px_rgba(61,50,80,.05)]" onClick={() => onSelect(active.id)}>
          <span className="flex items-start justify-between">
            <span className="vox-paper-gradient grid size-[30px] place-items-center rounded-[10px]"><MiniMark /></span>
            <span className={`rounded-md px-1.5 py-[3px] text-[8px] font-semibold leading-[11px] ${active.installed ? "bg-[#43845f17] text-[#43845f]" : "bg-[#8b5cf617] text-[#7d61b0]"}`}>{active.installed ? "Downloaded ✓" : "Not downloaded"}</span>
          </span>
          <span className="text-[11px] font-[650] leading-[15px] text-[#252832]">{displayName(active)}</span>
          <span className="text-[8px] leading-3 text-[#7d8491]">{formatBytes(active.sizeBytes)} · {active.multilingual ? "99 languages" : "English"} · ★ {active.speed}</span>
        </button>
      )}

      {alternative && (
        <div className="flex h-[60px] w-full shrink-0 items-center justify-between rounded-[13px] bg-white/50 px-[11px] py-2.5 shadow-[inset_0_0_0_1px_rgba(255,255,255,.8)]">
          <div className="min-w-0">
            <p className="truncate text-[10px] font-semibold leading-[14px] text-[#343843]">{displayName(alternative)}</p>
            <p className="mt-[3px] text-[8px] leading-[11px] text-[#858b98]">{alternative.multilingual ? "Multilingual" : "English"} · {alternative.speed.toLowerCase()}</p>
            {downloading === alternative.id && progress && <div className="mt-1 h-0.5 w-24 overflow-hidden rounded-full bg-[#e2e4e9]"><div className="vox-paper-gradient h-full" style={{ width: `${progress.fraction * 100}%` }} /></div>}
          </div>
          {alternative.installed ? (
            <button type="button" className="h-6 rounded-lg bg-[#eceef3] px-[9px] text-[8px] font-semibold text-[#555b68]" onClick={() => onSelect(alternative.id)}>Use</button>
          ) : (
            <button type="button" className="h-6 rounded-lg bg-[#eceef3] px-[9px] text-[8px] font-semibold text-[#555b68]" disabled={downloading === alternative.id} onClick={() => void download(alternative)}>{downloading === alternative.id ? "Downloading" : "Download"}</button>
          )}
        </div>
      )}

      <div className="flex w-full gap-2 rounded-xl bg-[#8b5cf60e] px-2.5 py-[9px]">
        <InfoIcon />
        <p className="text-[8px] leading-3 text-[#707684]">Turbo balances near-instant speed with excellent multilingual accuracy.</p>
      </div>
      {error && <p className="line-clamp-2 text-[8px] leading-3 text-[#b34d66]">{error}</p>}
    </div>
  );
}

function displayName(model: ManagedModel): string {
  if (model.id.includes("large-v3-turbo")) return "Whisper Large v3 Turbo";
  if (model.id.includes("small")) return "Whisper Small";
  if (model.id.includes("base")) return "Whisper Base";
  return model.name;
}

function MiniMark() {
  return <span className="flex items-center gap-0.5">{[9, 17, 12].map((height) => <span key={height} className="w-[3px] rounded bg-white" style={{ height }} />)}</span>;
}

function InfoIcon() {
  return <svg width="14" height="14" viewBox="0 0 16 16" className="shrink-0" aria-hidden="true"><circle cx="8" cy="8" r="6" fill="none" stroke="#9361cc" strokeWidth="1.2" /><path d="M8 7v4M8 4.7h.01" fill="none" stroke="#9361cc" strokeWidth="1.3" strokeLinecap="round" /></svg>;
}
