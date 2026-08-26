import type { ReactNode } from "react";

import { vox } from "../../lib/tauri";
import { VoxMark } from "../../ui/VoxMark";

export type DashboardPage = "home" | "history" | "settings";

interface DashboardShellProps {
  page: DashboardPage;
  onNavigate: (page: DashboardPage) => void;
  children: ReactNode;
}

const navigation: Array<{ id: DashboardPage; label: string }> = [
  { id: "home", label: "Home" },
  { id: "history", label: "History" },
  { id: "settings", label: "Settings" },
];

export function DashboardShell({ page, onNavigate, children }: DashboardShellProps) {
  return (
    <main className="vox-paper-background h-screen w-screen overflow-hidden p-4">
      <section className="vox-paper-window flex h-full w-full overflow-hidden rounded-3xl">
        <aside className="flex h-full w-[58px] shrink-0 flex-col items-center border-r border-[#353b4a14] bg-white/35 py-3.5">
          <div className="flex h-[18px] shrink-0 items-center gap-1" data-tauri-drag-region>
            <button
              type="button"
              className="size-[7px] rounded-full bg-[#ff6a5f]"
              aria-label="Hide Vox"
              onClick={() => void vox.hideDashboard()}
            />
            <span className="size-[7px] rounded-full bg-[#f2be4f]" />
            <span className="size-[7px] rounded-full bg-[#58c35a]" />
          </div>

          <VoxMark compact className="mt-[18px]" />

          <nav className="flex flex-col items-center gap-[13px] pt-[34px]" aria-label="Dashboard">
            {navigation.map((item) => (
              <button
                key={item.id}
                type="button"
                className={`grid size-[34px] shrink-0 place-items-center rounded-xl transition ${page === item.id ? "bg-white/80 shadow-[0_5px_14px_rgba(52,59,79,.08)]" : "hover:bg-white/45"}`}
                aria-label={item.label}
                aria-current={page === item.id ? "page" : undefined}
                onClick={() => onNavigate(item.id)}
              >
                <NavigationIcon page={item.id} active={page === item.id} />
              </button>
            ))}
          </nav>

          <div className="mt-auto grid size-7 shrink-0 place-items-center rounded-full bg-[#e4e8ef] text-[10px] font-[650] text-[#687080]">
            AJ
          </div>
        </aside>

        <div className="min-w-0 flex-1">{children}</div>
      </section>
    </main>
  );
}

function NavigationIcon({ page, active }: { page: DashboardPage; active: boolean }) {
  const stroke = active ? "#ff4d8d" : "#7c8290";
  if (page === "home") {
    return (
      <svg width="17" height="17" viewBox="0 0 20 20" aria-hidden="true">
        <path d="M3 9.2 10 3l7 6.2v7a1 1 0 0 1-1 1h-4v-5H8v5H4a1 1 0 0 1-1-1v-7Z" fill="none" stroke={stroke} strokeWidth="1.7" strokeLinejoin="round" />
      </svg>
    );
  }
  if (page === "history") {
    return (
      <svg width="17" height="17" viewBox="0 0 20 20" aria-hidden="true">
        <path d="M4 4h12M4 9h12M4 14h8" fill="none" stroke={stroke} strokeWidth="1.7" strokeLinecap="round" />
      </svg>
    );
  }
  return (
    <svg width="17" height="17" viewBox="0 0 20 20" aria-hidden="true">
      <path d="M10 6.6a3.4 3.4 0 1 0 0 6.8 3.4 3.4 0 0 0 0-6.8Z" fill="none" stroke={stroke} strokeWidth="1.6" />
      <path d="M10 2.5v1.4m0 12.2v1.4M17.5 10h-1.4M3.9 10H2.5m12.8-5.3-1 1m-8.6 8.6-1 1m10.6 0-1-1M5.7 5.7l-1-1" fill="none" stroke={stroke} strokeWidth="1.5" strokeLinecap="round" />
    </svg>
  );
}
