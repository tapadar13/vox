import { useEffect, useMemo, useState } from "react";

import { formatDuration, formatHotkey, relativeDate } from "../../lib/format";
import { useHistoryVersion } from "../../lib/hooks";
import { vox } from "../../lib/tauri";
import { emptyStats, type StatsSnapshot, type TranscriptRecord } from "../../lib/types";

interface StatsProps {
  hotkey: string;
  onViewHistory: () => void;
}

const barColors = [
  "#f4a0ac", "#f58ba8", "#ed77ad", "#df68bc", "#cf63c9", "#c35fd5",
  "#b65ddf", "#ad5de7", "#a25deb", "#985cf0", "#915cf3", "gradient",
];

export function Stats({ hotkey, onViewHistory }: StatsProps) {
  const historyVersion = useHistoryVersion();
  const [stats, setStats] = useState<StatsSnapshot>(emptyStats);
  const [records, setRecords] = useState<TranscriptRecord[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let mounted = true;
    void Promise.all([vox.stats(), vox.history(undefined, 3)]).then(([nextStats, nextRecords]) => {
      if (mounted) {
        setStats(nextStats);
        setRecords(nextRecords);
      }
    }).finally(() => mounted && setLoading(false));
    return () => {
      mounted = false;
    };
  }, [historyVersion]);

  const weeklyActivity = useMemo(() => buildWeeklyActivity(stats.activity), [stats.activity]);
  const now = new Date();
  const date = now.toLocaleDateString(undefined, { weekday: "long", day: "numeric", month: "long" });
  const greeting = now.getHours() < 12 ? "Good morning" : now.getHours() < 18 ? "Good afternoon" : "Good evening";

  return (
    <div className={`flex h-full flex-col gap-3.5 overflow-hidden px-[22px] py-5 transition-opacity ${loading ? "opacity-55" : "opacity-100"}`}>
      <header className="flex h-11 shrink-0 items-center justify-between" data-tauri-drag-region>
        <div className="flex flex-col gap-0.5">
          <p className="text-xs font-medium leading-4 text-[#747b89]">{date}</p>
          <h1 className="text-2xl font-[650] leading-7 tracking-[-.035em] text-[#11131a]">{greeting}</h1>
        </div>
        <div className="flex h-[30px] items-center gap-[7px] rounded-full bg-white/80 px-[11px] shadow-[inset_0_0_0_1px_rgba(56,62,77,.08)]">
          <span className="size-[7px] rounded-full bg-[#35b86b] shadow-[0_0_0_3px_rgba(53,184,107,.11)]" />
          <span className="text-[11px] font-[550] leading-[14px] text-[#535a68]">{formatHotkey(hotkey)} to dictate</span>
        </div>
      </header>

      <section className="flex h-[106px] shrink-0 gap-[9px]" aria-label="Dictation statistics">
        <StatCard className="w-[158px] bg-white/75 shadow-[inset_0_0_0_1px_rgba(255,255,255,.88),0_8px_20px_rgba(44,49,68,.06)]" label="TIME SAVED" value={formatDuration(stats.timeSavedMs)} detail={`${stats.streakDays}-day streak`} primary />
        <StatCard className="w-32" label="WORDS DICTATED" value={stats.totalWords.toLocaleString()} detail="lifetime" />
        <StatCard className="w-32" label="TRANSCRIPTIONS" value={stats.transcriptionCount.toLocaleString()} detail="all time" />
        <StatCard className="min-w-0 flex-1" label="SPEAKING PACE" value={Math.round(stats.speakingWpm).toLocaleString()} unit="WPM" detail={`${Math.max(0, Math.round(stats.speakingWpm / 40))}× faster than typing`} accentDetail />
      </section>

      <section className="flex h-[220px] shrink-0 gap-2.5">
        <div className="vox-paper-panel flex h-full w-[356px] shrink-0 flex-col gap-2.5 rounded-[18px] p-3.5">
          <div className="flex flex-col gap-3">
            <div className="flex items-center justify-between">
              <div>
                <h2 className="text-[13px] font-[650] leading-[17px] text-[#20232c]">Weekly activity</h2>
                <p className="text-[10px] leading-[13px] text-[#7b8290]">Words per day · 12 weeks</p>
              </div>
              <span className="flex h-6 items-center gap-[5px] rounded-full bg-white/75 px-[9px] text-[9px] font-semibold text-[#565d6b]">
                <FlameIcon /> {stats.streakDays}-day streak
              </span>
            </div>
            <div className="flex h-[104px] shrink-0 items-end gap-[7px] pt-1" aria-label="Words dictated over the last 12 weeks">
              {weeklyActivity.map((height, index) => (
                <span
                  key={index}
                  className={`w-[18px] shrink-0 rounded-md ${index === 11 ? "vox-paper-gradient shadow-[0_7px_16px_rgba(171,75,207,.18)]" : ""}`}
                  style={{ height, backgroundColor: index === 11 ? undefined : barColors[index], opacity: index === 11 ? 1 : 0.34 + index * 0.056 }}
                />
              ))}
            </div>
            <div className="flex justify-between text-[9px] leading-3 text-[#9399a5]">
              <span>12 weeks ago</span>
              <span>This week</span>
            </div>
          </div>
        </div>

        <div className="vox-paper-panel flex h-full min-w-0 flex-1 flex-col rounded-[18px] p-3.5">
          <div className="flex h-[25px] shrink-0 items-center justify-between">
            <h2 className="text-[13px] font-[650] leading-[17px] text-[#20232c]">Recent</h2>
            <button type="button" className="text-[9px] font-semibold leading-3 text-[#b84d94]" onClick={onViewHistory}>View all →</button>
          </div>
          {records.length === 0 ? (
            <div className="grid flex-1 place-items-center text-center text-[10px] text-[#8a909d]">Your recent dictations will appear here.</div>
          ) : records.map((record) => <RecentRow key={record.id ?? record.createdAt} record={record} />)}
        </div>
      </section>
    </div>
  );
}

function StatCard({ className, label, value, unit, detail, primary, accentDetail }: { className: string; label: string; value: string; unit?: string; detail: string; primary?: boolean; accentDetail?: boolean }) {
  return (
    <div className={`flex h-[106px] shrink-0 flex-col justify-between rounded-[17px] bg-white/60 p-[13px] shadow-[inset_0_0_0_1px_rgba(255,255,255,.86)] ${className}`}>
      <p className="text-[9px] font-[650] leading-[13px] tracking-[.1em] text-[#7a8190]">{label}</p>
      <div className="flex items-baseline gap-[3px]">
        <p className={`${primary ? "vox-paper-gradient-text text-[29px] leading-[34px]" : "text-[25px] leading-[30px] text-[#171922]"} font-[680] tabular-nums tracking-[-.045em]`}>{value}</p>
        {unit && <span className="text-[9px] leading-3 text-[#7c8391]">{unit}</span>}
      </div>
      <p className={`${accentDetail ? "text-[9px] font-semibold leading-3 text-[#b84d94]" : "text-[10px] leading-[13px] text-[#7b8190]"}`}>{detail}</p>
    </div>
  );
}

function RecentRow({ record }: { record: TranscriptRecord }) {
  return (
    <div className="flex h-[45px] shrink-0 items-center gap-2 border-b border-[#2a2f3c0f]">
      <span className="grid size-6 shrink-0 place-items-center rounded-lg bg-[#8b5cf617]"><MicIcon /></span>
      <div className="min-w-0 flex-1 overflow-hidden">
        <p className="truncate text-[10px] font-medium leading-[13px] text-[#343843]">{record.text}</p>
        <p className="text-[8px] leading-[11px] text-[#8a909d]">{record.wordCount} words</p>
      </div>
      <span className="w-[23px] shrink-0 text-right text-[8px] leading-[11px] text-[#9aa0ab]">{relativeDate(record.createdAt).replace(" ago", "")}</span>
    </div>
  );
}

function buildWeeklyActivity(activity: StatsSnapshot["activity"]): number[] {
  if (activity.length === 0) return [38, 52, 44, 67, 57, 76, 63, 82, 72, 92, 84, 99];
  const weeks = Array.from({ length: 12 }, () => 0);
  activity.slice(-84).forEach((day, index, days) => {
    const offset = 84 - days.length;
    weeks[Math.min(11, Math.floor((offset + index) / 7))] += day.words;
  });
  const maximum = Math.max(...weeks, 1);
  return weeks.map((words) => Math.max(12, Math.round((words / maximum) * 99)));
}

function FlameIcon() {
  return <svg width="11" height="13" viewBox="0 0 12 14" aria-hidden="true"><path d="M7.5 1.2c.4 2-1.8 2.8-.8 4.3.5.8 1.7.2 1.7-.9 1.4 1.2 2.1 2.6 2.1 4.2A4.5 4.5 0 1 1 2 6.8c.5.8 1.2 1.2 1.8.8 1.2-.9-.3-3.2 3.7-6.4Z" fill="#ff6b57" /></svg>;
}

function MicIcon() {
  return <svg width="11" height="13" viewBox="0 0 12 14" aria-hidden="true"><rect x="3.5" y="1" width="5" height="8" rx="2.5" fill="none" stroke="#9a59d7" strokeWidth="1.2" /><path d="M1.8 6.8a4.2 4.2 0 0 0 8.4 0M6 11v2" fill="none" stroke="#9a59d7" strokeWidth="1.2" strokeLinecap="round" /></svg>;
}
