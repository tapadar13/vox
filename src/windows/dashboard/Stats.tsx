import { Clock3, Flame, Gauge, Sparkles } from "lucide-react";
import { useEffect, useState } from "react";

import { formatDuration } from "../../lib/format";
import { useHistoryVersion } from "../../lib/hooks";
import { vox } from "../../lib/tauri";
import { emptyStats, type StatsSnapshot } from "../../lib/types";
import { GlassPanel } from "../../ui/GlassPanel";
import { ActivityChart } from "./ActivityChart";

export function Stats() {
  const historyVersion = useHistoryVersion();
  const [stats, setStats] = useState<StatsSnapshot>(emptyStats);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let mounted = true;
    void vox
      .stats()
      .then((value) => mounted && setStats(value))
      .finally(() => mounted && setLoading(false));
    return () => {
      mounted = false;
    };
  }, [historyVersion]);

  const cards = [
    { label: "Words dictated", value: stats.totalWords.toLocaleString(), icon: Sparkles },
    { label: "Time saved", value: formatDuration(stats.timeSavedMs), icon: Clock3 },
    { label: "Speaking speed", value: `${Math.round(stats.speakingWpm)} WPM`, icon: Gauge },
    { label: "Current streak", value: `${stats.streakDays} day${stats.streakDays === 1 ? "" : "s"}`, icon: Flame },
  ];

  return (
    <div className={`space-y-4 transition-opacity ${loading ? "opacity-50" : "opacity-100"}`}>
      <header>
        <p className="text-[11px] font-medium uppercase tracking-[.18em] text-violet-300/65">Your voice, amplified</p>
        <h1 className="mt-1 text-[25px] font-semibold tracking-tight">Good to see you.</h1>
        <p className="mt-1 text-[12px] text-white/40">Everything here was calculated locally on this Mac.</p>
      </header>

      <div className="grid grid-cols-4 gap-2.5">
        {cards.map(({ label, value, icon: Icon }) => (
          <GlassPanel key={label} className="p-3.5">
            <Icon className="size-3.5 text-violet-300/70" />
            <p className="mt-3 text-[19px] font-semibold tracking-tight text-white/95">{value}</p>
            <p className="mt-0.5 text-[10px] text-white/35">{label}</p>
          </GlassPanel>
        ))}
      </div>

      <GlassPanel className="px-4 pb-2 pt-3.5">
        <div className="flex items-end justify-between">
          <div>
            <h2 className="text-[13px] font-medium">Activity</h2>
            <p className="mt-0.5 text-[10px] text-white/35">Last four weeks · words per day</p>
          </div>
          <span className="rounded-full bg-white/[.055] px-2.5 py-1 text-[9px] text-white/40">
            {stats.transcriptionCount.toLocaleString()} transcriptions
          </span>
        </div>
        <ActivityChart activity={stats.activity} />
      </GlassPanel>
    </div>
  );
}
