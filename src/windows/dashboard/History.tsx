import { Check, Clipboard, Search, Trash2 } from "lucide-react";
import { useCallback, useEffect, useState } from "react";

import { formatDuration, relativeDate } from "../../lib/format";
import { useDebouncedValue, useHistoryVersion } from "../../lib/hooks";
import { vox } from "../../lib/tauri";
import type { TranscriptRecord } from "../../lib/types";
import { Button } from "../../ui/Button";
import { GlassPanel } from "../../ui/GlassPanel";

export function History() {
  const version = useHistoryVersion();
  const [search, setSearch] = useState("");
  const query = useDebouncedValue(search.trim(), 180);
  const [records, setRecords] = useState<TranscriptRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [copiedId, setCopiedId] = useState<number | null>(null);

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      setRecords(await vox.history(query || undefined));
    } finally {
      setLoading(false);
    }
  }, [query]);

  useEffect(() => {
    void refresh();
  }, [refresh, version]);

  const copy = async (record: TranscriptRecord) => {
    await vox.copyText(record.text);
    setCopiedId(record.id);
    window.setTimeout(() => setCopiedId(null), 1_000);
  };

  const remove = async (record: TranscriptRecord) => {
    if (record.id == null) return;
    await vox.deleteHistory(record.id);
    setRecords((items) => items.filter((item) => item.id !== record.id));
  };

  const clear = async () => {
    if (!window.confirm("Clear every transcription from this Mac? This cannot be undone.")) return;
    await vox.clearHistory();
    setRecords([]);
  };

  return (
    <div className="flex h-full min-h-0 flex-col">
      <header className="flex items-end justify-between">
        <div>
          <h1 className="text-[24px] font-semibold tracking-tight">History</h1>
          <p className="mt-1 text-[11px] text-white/38">Your local safety net. Audio is never stored.</p>
        </div>
        {records.length > 0 && (
          <Button variant="ghost" className="h-8 text-[11px] text-rose-200/65" onClick={() => void clear()}>
            Clear all
          </Button>
        )}
      </header>

      <label className="mt-4 flex h-10 items-center gap-2.5 rounded-xl border border-white/[.08] bg-white/[.045] px-3 focus-within:border-violet-300/25">
        <Search className="size-3.5 text-white/28" />
        <input
          className="min-w-0 flex-1 bg-transparent text-[12px] text-white/85 outline-none placeholder:text-white/25"
          value={search}
          onChange={(event) => setSearch(event.target.value)}
          placeholder="Search transcriptions"
        />
      </label>

      <div className={`mt-3 min-h-0 flex-1 space-y-2 overflow-y-auto pr-1 transition-opacity ${loading ? "opacity-45" : "opacity-100"}`}>
        {!loading && records.length === 0 && (
          <GlassPanel className="grid min-h-48 place-items-center p-8 text-center">
            <div>
              <Clipboard className="mx-auto size-6 text-white/20" />
              <p className="mt-3 text-[13px] font-medium text-white/60">
                {query ? "No matching transcriptions" : "Your words will appear here"}
              </p>
              <p className="mt-1 text-[10px] text-white/30">
                {query ? "Try a different search." : "Press your hotkey and start speaking."}
              </p>
            </div>
          </GlassPanel>
        )}

        {records.map((record) => (
          <GlassPanel key={record.id ?? record.createdAt} className="group p-3.5">
            <div className="flex items-start gap-3">
              <p className="line-clamp-3 min-w-0 flex-1 text-[12px] leading-[1.55] text-white/72">{record.text}</p>
              <div className="flex shrink-0 opacity-0 transition group-hover:opacity-100 group-focus-within:opacity-100">
                <button
                  className="grid size-7 place-items-center rounded-lg text-white/35 transition hover:bg-white/[.07] hover:text-white"
                  onClick={() => void copy(record)}
                  aria-label="Copy transcription"
                >
                  {copiedId === record.id ? <Check className="size-3.5 text-emerald-300" /> : <Clipboard className="size-3.5" />}
                </button>
                <button
                  className="grid size-7 place-items-center rounded-lg text-white/30 transition hover:bg-rose-400/10 hover:text-rose-300"
                  onClick={() => void remove(record)}
                  aria-label="Delete transcription"
                >
                  <Trash2 className="size-3.5" />
                </button>
              </div>
            </div>
            <div className="mt-2.5 flex items-center gap-2 text-[9px] text-white/28">
              <span>{relativeDate(record.createdAt)}</span>
              <span>·</span>
              <span>{record.wordCount} words</span>
              <span>·</span>
              <span>{formatDuration(record.latencyMs)} latency</span>
              <span className="ml-auto uppercase tracking-wider">{record.language}</span>
            </div>
          </GlassPanel>
        ))}
      </div>
    </div>
  );
}
