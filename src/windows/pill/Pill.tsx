import { formatDuration } from "../../lib/format";
import { useVoxState } from "../../lib/hooks";
import { vox } from "../../lib/tauri";
import { CountdownRing } from "./CountdownRing";
import { Waveform } from "./Waveform";

const pillBase = "vox-pill flex h-12 shrink-0 items-center rounded-full border border-white/10 bg-[#0f1117e6] text-white shadow-[inset_0_1px_0_rgba(255,255,255,.08),0_12px_28px_rgba(16,18,24,.18)]";

export function Pill() {
  const { state } = useVoxState();

  return (
    <main className="flex h-screen w-screen items-center justify-center overflow-hidden" aria-live="polite">
      {state.phase === "recording" && (
        <section
          className={`${pillBase} w-[300px] justify-between px-[15px] shadow-[inset_0_1px_0_rgba(255,255,255,.08),0_14px_34px_rgba(78,46,120,.24)]`}
          role="status"
          aria-label={state.partialTranscript ? `Recording. ${state.partialTranscript}` : "Recording locally"}
        >
          <Waveform level={state.audioLevel} />
          <span className="text-xs font-medium leading-4 tabular-nums text-white/80">{formatRecordingTime(state.elapsedMs)}</span>
        </section>
      )}

      {(state.phase === "transcribing" || state.phase === "delivering") && (
        <section className={`${pillBase} w-[300px] gap-3.5 px-[15px]`} role="status" aria-label={state.phase === "transcribing" ? "Transcribing locally" : "Pasting at your cursor"}>
          <span className="h-1 w-56 shrink-0 overflow-hidden rounded-full bg-[linear-gradient(90deg,oklab(0.708_0.16_0.092/.16),oklab(0.686_0.218_0.012)_52%,oklab(0.606_0.085_-0.202/.18))] shadow-[0_0_14px_rgba(255,77,141,.32)]">
            <span className="vox-pill-progress block h-full w-1/2 rounded-full bg-[var(--vox-paper-gradient)]" />
          </span>
          <span className="vox-shimmer-text text-lg leading-[18px] tracking-[.12em]">···</span>
        </section>
      )}

      {state.phase === "cancelPending" && (
        <section className={`${pillBase} w-[300px] gap-2.5 px-[13px]`} role="status">
          <CountdownRing />
          <span className="text-xs font-[520] leading-4 text-[#f7f7fa]">Cancelling… Esc to keep</span>
        </section>
      )}

      {state.phase === "success" && state.deliveryMode === "clipboard" && (
        <section className={`${pillBase} w-[272px] gap-[9px] px-[15px]`} role="status">
          <ClipboardIcon />
          <span className="text-xs font-medium leading-4 text-[#8e93a0]">Copied — press ⌘V</span>
        </section>
      )}

      {state.phase === "success" && state.deliveryMode !== "clipboard" && (
        <section className={`${pillBase} w-[210px] justify-center gap-[9px] px-4`} role="status">
          <span className="vox-paper-gradient grid size-6 shrink-0 place-items-center rounded-full"><CheckIcon /></span>
          <span className="text-[13px] font-[550] leading-[18px]">Pasted</span>
        </section>
      )}

      {state.phase === "error" && (
        <section className="flex h-12 w-[300px] shrink-0 items-center rounded-full border border-[#ff758a52] bg-[#1d1217ed] px-3.5 shadow-[inset_0_1px_0_rgba(255,255,255,.07),0_12px_28px_rgba(109,29,49,.16)]" role="status">
          <button type="button" className="flex min-w-0 flex-1 items-center gap-[9px]" onClick={() => void vox.retry()}>
            <RetryIcon />
            <span className="truncate text-xs font-medium leading-4 text-[#f9f1f3]">Couldn't transcribe — Retry</span>
          </button>
          <button type="button" className="ml-2 text-sm text-[#ff879a]" aria-label="Dismiss" onClick={() => void vox.dismiss()}>×</button>
        </section>
      )}

      {state.phase === "idle" && (
        <section className={`${pillBase} w-[210px] justify-center gap-[9px] px-4`} role="status">
          <span className="vox-paper-gradient grid size-6 place-items-center rounded-full"><MiniWave /></span>
          <span className="text-xs font-[550] text-white/85">Vox is ready</span>
        </section>
      )}
    </main>
  );
}

function formatRecordingTime(milliseconds: number): string {
  const formatted = formatDuration(milliseconds);
  return formatted.endsWith("s") ? `0:${formatted.slice(0, -1).padStart(2, "0")}` : formatted;
}

function CheckIcon() { return <svg width="14" height="14" viewBox="0 0 14 14" aria-hidden="true"><path d="m3 7.2 2.5 2.4L11 4.3" fill="none" stroke="white" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" /></svg>; }
function ClipboardIcon() { return <svg width="17" height="17" viewBox="0 0 18 18" className="shrink-0" aria-hidden="true"><rect x="4" y="4" width="10" height="12" rx="2.5" fill="none" stroke="#d7d9e0" strokeWidth="1.4" /><path d="M6.5 5V3.8c0-.7.6-1.3 1.3-1.3h2.4c.7 0 1.3.6 1.3 1.3V5" fill="none" stroke="#d7d9e0" strokeWidth="1.4" /></svg>; }
function RetryIcon() { return <svg width="16" height="16" viewBox="0 0 18 18" className="shrink-0" aria-hidden="true"><path d="M14.3 6.4A6 6 0 1 0 15 11" fill="none" stroke="#ff879a" strokeWidth="1.6" strokeLinecap="round" /><path d="M14.4 2.8v3.9h-3.9" fill="none" stroke="#ff879a" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round" /></svg>; }
function MiniWave() { return <span className="flex items-center gap-px">{[6, 12, 8].map((height) => <span key={height} className="w-0.5 rounded bg-white" style={{ height }} />)}</span>; }
