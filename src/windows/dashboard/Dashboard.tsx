import { Clock3, History as HistoryIcon, Minus, Settings2, Sparkles } from "lucide-react";
import { lazy, Suspense, useEffect, useState } from "react";

import { formatHotkey } from "../../lib/format";
import { useVoxState } from "../../lib/hooks";
import { vox } from "../../lib/tauri";
import { defaultSettings, type Settings as VoxSettings } from "../../lib/types";
import { VoxMark } from "../../ui/VoxMark";

const History = lazy(() => import("./History").then(({ History: component }) => ({ default: component })));
const Onboarding = lazy(() =>
  import("./Onboarding").then(({ Onboarding: component }) => ({ default: component })),
);
const Settings = lazy(() =>
  import("./Settings").then(({ Settings: component }) => ({ default: component })),
);
const Stats = lazy(() => import("./Stats").then(({ Stats: component }) => ({ default: component })));

type Page = "home" | "history" | "settings";

const navigation = [
  { id: "home", label: "Overview", icon: Sparkles },
  { id: "history", label: "History", icon: HistoryIcon },
  { id: "settings", label: "Settings", icon: Settings2 },
] as const;

export function Dashboard() {
  const [page, setPage] = useState<Page>("home");
  const [settings, setSettings] = useState<VoxSettings>(defaultSettings);
  const [loaded, setLoaded] = useState(false);
  const { state } = useVoxState();

  useEffect(() => {
    let mounted = true;
    void vox
      .settings()
      .then((next) => mounted && setSettings(next))
      .finally(() => mounted && setLoaded(true));
    return () => {
      mounted = false;
    };
  }, []);

  if (!loaded) {
    return <main className="vox-dashboard min-h-screen" />;
  }

  if (!settings.onboardingComplete) {
    return (
      <Suspense fallback={<main className="vox-dashboard min-h-screen" />}>
        <Onboarding settings={settings} onComplete={setSettings} />
      </Suspense>
    );
  }

  return (
    <main className="vox-dashboard flex h-screen overflow-hidden text-white">
      <aside className="flex w-[178px] shrink-0 flex-col border-r border-white/[.07] bg-black/15 p-3.5">
        <div className="flex items-center gap-2.5 px-1.5 py-1" data-tauri-drag-region>
          <VoxMark compact />
          <div>
            <p className="text-[14px] font-semibold tracking-tight">Vox</p>
            <p className="text-[8px] uppercase tracking-[.16em] text-white/25">Local dictation</p>
          </div>
        </div>

        <nav className="mt-7 space-y-1" aria-label="Dashboard">
          {navigation.map(({ id, label, icon: Icon }) => (
            <button
              key={id}
              type="button"
              className={`flex h-9 w-full items-center gap-2.5 rounded-xl px-3 text-[11px] transition ${page === id ? "bg-white/[.08] text-white shadow-[inset_0_0_0_1px_rgba(255,255,255,.045)]" : "text-white/38 hover:bg-white/[.045] hover:text-white/70"}`}
              onClick={() => setPage(id)}
            >
              <Icon className={`size-3.5 ${page === id ? "text-violet-300" : ""}`} />
              {label}
            </button>
          ))}
        </nav>

        <div className="mt-auto">
          <div className="rounded-xl border border-white/[.065] bg-white/[.03] p-3">
            <div className="flex items-center gap-2">
              <span className={`size-1.5 rounded-full ${state.modelReady ? "bg-emerald-300 shadow-[0_0_8px_rgba(85,214,169,.6)]" : "bg-amber-300"}`} />
              <span className="text-[9px] font-medium text-white/48">{state.modelReady ? "Model ready" : "Model needed"}</span>
            </div>
            <p className="mt-2 text-[8px] uppercase tracking-[.12em] text-white/22">Press to dictate</p>
            <p className="mt-1 font-mono text-[10px] text-white/48">{formatHotkey(settings.hotkey)}</p>
          </div>
          <p className="mt-3 flex items-center justify-center gap-1 text-[8px] text-white/20">
            <Clock3 className="size-2.5" /> Audio never leaves this Mac
          </p>
        </div>
      </aside>

      <section className="relative min-w-0 flex-1 overflow-hidden">
        <header className="absolute inset-x-0 top-0 z-10 flex h-8 items-center justify-end px-3" data-tauri-drag-region>
          <button
            type="button"
            className="grid size-6 place-items-center rounded-lg text-white/25 transition hover:bg-white/[.07] hover:text-white/70"
            aria-label="Hide dashboard"
            onClick={() => void vox.hideDashboard()}
          >
            <Minus className="size-3.5" />
          </button>
        </header>
        <Suspense fallback={<div className="h-full px-6 pb-6 pt-10 text-[11px] text-white/30">Loading…</div>}>
          <div className="h-full overflow-y-auto px-6 pb-6 pt-10">
            {page === "home" && <Stats />}
            {page === "history" && <History />}
            {page === "settings" && <Settings value={settings} onSaved={setSettings} />}
          </div>
        </Suspense>
      </section>
    </main>
  );
}
