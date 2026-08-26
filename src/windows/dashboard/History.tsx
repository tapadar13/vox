import { useCallback, useEffect, useMemo, useState } from "react";

import { formatDuration, relativeDate } from "../../lib/format";
import { useDebouncedValue, useHistoryVersion } from "../../lib/hooks";
import { vox } from "../../lib/tauri";
import type { TranscriptRecord } from "../../lib/types";

export function History() {
  const version = useHistoryVersion();
  const [search, setSearch] = useState("");
  const [language, setLanguage] = useState("all");
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

  const visible = useMemo(
    () => language === "all" ? records : records.filter((record) => record.language.toLowerCase().startsWith(language)),
    [language, records],
  );

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

  return (
    <div className="flex h-full flex-col gap-3 overflow-hidden px-[22px] py-5">
      <div className="flex h-[38px] shrink-0 items-center gap-[9px]" data-tauri-drag-region>
        <label className="flex h-[38px] min-w-0 flex-1 items-center gap-2 rounded-[13px] bg-white/75 px-3 shadow-[inset_0_0_0_1px_rgba(63,69,84,.07)] focus-within:shadow-[inset_0_0_0_1px_rgba(184,77,148,.25)]">
          <SearchIcon />
          <input
            className="min-w-0 flex-1 bg-transparent text-xs leading-4 text-[#343843] outline-none placeholder:text-[#737a88]"
            value={search}
            onChange={(event) => setSearch(event.target.value)}
            placeholder="Search transcriptions"
          />
          <kbd className="rounded-md bg-[#eef0f4] px-1.5 py-[3px] text-[9px] font-normal leading-3 text-[#8d94a0]">⌘F</kbd>
        </label>

        <label className="relative flex h-[38px] shrink-0 items-center gap-1.5 rounded-[13px] bg-white/70 px-[11px] shadow-[inset_0_0_0_1px_rgba(63,69,84,.07)]">
          <select className="appearance-none bg-transparent pr-4 text-[10px] font-semibold leading-[13px] text-[#4c5260] outline-none" value={language} onChange={(event) => setLanguage(event.target.value)}>
            <option value="all">All languages</option>
            <option value="en">English</option>
            <option value="hi">Hindi</option>
            <option value="es">Spanish</option>
          </select>
          <ChevronIcon />
        </label>
      </div>

      <section className={`flex h-[358px] shrink-0 gap-2.5 transition-opacity ${loading ? "opacity-50" : "opacity-100"}`}>
        <div className="flex h-full w-[374px] shrink-0 flex-col gap-2 overflow-y-auto pr-px">
          {!loading && visible.length === 0 && (
            <div className="vox-paper-panel grid h-full place-items-center rounded-[17px] px-8 text-center">
              <div>
                <p className="text-[13px] font-[650] text-[#343843]">{query ? "No matching transcriptions" : "Nothing here yet"}</p>
                <p className="mt-1 text-[10px] text-[#7c8290]">{query ? "Try a different search." : "Press your hotkey and just talk."}</p>
              </div>
            </div>
          )}
          {visible.map((record, index) => (
            <article
              key={record.id ?? record.createdAt}
              className={`${index === 0 ? "min-h-[126px] bg-white/80 shadow-[inset_0_0_0_1px_rgba(255,255,255,.9),0_10px_24px_rgba(50,57,78,.08)]" : "min-h-[94px] bg-white/55 shadow-[inset_0_0_0_1px_rgba(255,255,255,.84)]"} flex shrink-0 flex-col gap-[7px] rounded-[17px] px-3.5 py-3`}
            >
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-[7px]">
                  <LanguageChip language={record.language} />
                  <span className="text-[9px] leading-3 text-[#8a909d]">{relativeDate(record.createdAt)}</span>
                </div>
                {index === 0 ? (
                  <div className="flex items-center gap-[7px]">
                    <button type="button" className="vox-paper-gradient flex h-6 items-center gap-[5px] rounded-full px-[9px] text-[9px] font-semibold text-white" onClick={() => void copy(record)}>
                      <CopyIcon /> {copiedId === record.id ? "Copied" : "Copy"}
                    </button>
                    <button type="button" className="text-[9px] text-[#9298a4] hover:text-[#b34d66]" onClick={() => void remove(record)}>Delete</button>
                  </div>
                ) : (
                  <span className="text-[9px] leading-3 text-[#9a9fab]">{formatDuration(record.durationMs)} · {record.wordCount} words</span>
                )}
              </div>
              <p className={`${index === 0 ? "line-clamp-2 text-xs leading-[17px]" : "line-clamp-2 text-[11px] leading-4"} font-medium text-[#343741]`}>{record.text}</p>
              {index === 0 && (
                <div className="mt-auto flex items-center gap-2.5 text-[9px] leading-3 text-[#8a909d]">
                  <span>{formatDuration(record.durationMs)}</span>
                  <span>{record.wordCount} words</span>
                  <span>{record.delivered ? "Vox pasted at your cursor" : "Copied to clipboard"}</span>
                </div>
              )}
            </article>
          ))}
        </div>

        <div className="vox-paper-panel flex h-full min-w-0 flex-1 flex-col items-center justify-center gap-3 rounded-[18px] p-[18px] text-center">
          <div className="grid size-[82px] shrink-0 place-items-center rounded-[28px] bg-[linear-gradient(145deg,oklab(0.708_0.16_0.092/.16),oklab(0.686_0.218_0.012/.16)_50%,oklab(0.606_0.085_-0.202/.18))] shadow-[inset_0_0_0_1px_rgba(255,255,255,.74)]">
            <div className="flex items-center gap-[3px]">
              {[14, 26, 38, 22, 11].map((height, index) => <span key={height} className="w-1 rounded" style={{ height, backgroundColor: ["#ff6b57", "#ff4d8d", "#cd55c3", "#a15bea", "#8b5cf6"][index] }} />)}
            </div>
          </div>
          <p className="text-base font-[650] leading-5 tracking-[-.02em] text-[#282b35]">Nothing here yet</p>
          <p className="text-[10px] leading-[15px] text-[#7c8290]">Press your hotkey and just talk.</p>
        </div>
      </section>
    </div>
  );
}

function LanguageChip({ language }: { language: string }) {
  const value = language.slice(0, 2).toUpperCase() || "—";
  const palette = value === "HI" ? "bg-[#6b60b917] text-[#6b60b9]" : value === "ES" ? "bg-[#a4527d14] text-[#a4527d]" : "bg-[#b14bbe17] text-[#a74cb2]";
  return <span className={`rounded-md px-1.5 py-[3px] text-[9px] font-[650] leading-3 ${palette}`}>{value}</span>;
}

function SearchIcon() {
  return <svg width="15" height="15" viewBox="0 0 18 18" className="shrink-0" aria-hidden="true"><circle cx="7.5" cy="7.5" r="4.8" fill="none" stroke="#7c8290" strokeWidth="1.5" /><path d="m11.2 11.2 3.8 3.8" fill="none" stroke="#7c8290" strokeWidth="1.5" strokeLinecap="round" /></svg>;
}

function ChevronIcon() {
  return <svg width="9" height="6" viewBox="0 0 9 6" className="pointer-events-none absolute right-[11px] shrink-0" aria-hidden="true"><path d="m1 1 3.5 3.5L8 1" fill="none" stroke="#7c8290" strokeWidth="1.3" strokeLinecap="round" /></svg>;
}

function CopyIcon() {
  return <svg width="10" height="11" viewBox="0 0 12 13" aria-hidden="true"><rect x="3.5" y="3.5" width="6" height="7" rx="1.5" fill="none" stroke="white" strokeWidth="1.1" /><path d="M2.5 8.5h-1v-7h6v1" fill="none" stroke="white" strokeWidth="1.1" strokeLinecap="round" /></svg>;
}
