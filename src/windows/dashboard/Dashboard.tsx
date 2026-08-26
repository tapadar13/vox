import { lazy, Suspense, useEffect, useState } from "react";

import { onDashboardNavigation, vox } from "../../lib/tauri";
import { defaultSettings, type Settings as VoxSettings } from "../../lib/types";
import { resizeVoxWindow } from "../../lib/window";
import { DashboardShell, type DashboardPage } from "./DashboardShell";

const History = lazy(() => import("./History").then(({ History: component }) => ({ default: component })));
const Onboarding = lazy(() =>
  import("./Onboarding").then(({ Onboarding: component }) => ({ default: component })),
);
const Settings = lazy(() =>
  import("./Settings").then(({ Settings: component }) => ({ default: component })),
);
const Stats = lazy(() => import("./Stats").then(({ Stats: component }) => ({ default: component })));

export function Dashboard() {
  const [page, setPage] = useState<DashboardPage>("home");
  const [settings, setSettings] = useState<VoxSettings>(defaultSettings);
  const [loaded, setLoaded] = useState(false);

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

  useEffect(() => {
    if (loaded) void resizeVoxWindow(settings.onboardingComplete ? 720 : 413, settings.onboardingComplete ? 480 : 520);
  }, [loaded, settings.onboardingComplete]);

  useEffect(() => {
    let mounted = true;
    let unlisten: () => void = () => undefined;
    void onDashboardNavigation((next) => mounted && setPage(next)).then((stop) => {
      if (mounted) unlisten = stop;
      else stop();
    });
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.metaKey && event.key === ",") {
        event.preventDefault();
        setPage("settings");
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => {
      mounted = false;
      unlisten();
      window.removeEventListener("keydown", onKeyDown);
    };
  }, []);

  if (!loaded) {
    return <main className="vox-paper-background min-h-screen" />;
  }

  if (!settings.onboardingComplete) {
    return (
      <Suspense fallback={<main className="vox-paper-background min-h-screen" />}>
        <Onboarding settings={settings} onComplete={setSettings} />
      </Suspense>
    );
  }

  return (
    <DashboardShell page={page} onNavigate={setPage}>
      <Suspense fallback={<div className="h-full p-5 text-[11px] text-[#7a8190]">Loading…</div>}>
        {page === "home" && <Stats hotkey={settings.hotkey} onViewHistory={() => setPage("history")} />}
        {page === "history" && <History />}
        {page === "settings" && <Settings value={settings} onSaved={setSettings} />}
      </Suspense>
    </DashboardShell>
  );
}
