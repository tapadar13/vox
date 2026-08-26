import { AlertTriangle, Check, Clipboard, Mic, RotateCcw, X } from "lucide-react";

import { formatDuration } from "../../lib/format";
import { useVoxState } from "../../lib/hooks";
import { vox } from "../../lib/tauri";
import { Button } from "../../ui/Button";
import { CountdownRing } from "./CountdownRing";
import { LiveTranscript } from "./LiveTranscript";
import { Waveform } from "./Waveform";

export function Pill() {
  const { state } = useVoxState();

  return (
    <main className="flex min-h-screen items-center justify-center p-2.5" aria-live="polite">
      <section
        className={`vox-pill relative flex h-[82px] w-full items-center overflow-hidden rounded-[28px] border px-5 shadow-2xl transition-colors ${state.phase === "error" ? "border-rose-300/20" : "border-white/10"}`}
        data-phase={state.phase}
        role="status"
      >
        <div className="pointer-events-none absolute inset-x-10 top-0 h-px bg-gradient-to-r from-transparent via-white/30 to-transparent" />
        {state.phase === "recording" && (
          <div className="flex w-full items-center gap-4">
            <span className="grid size-9 shrink-0 place-items-center rounded-full bg-rose-400/12 text-rose-300">
              <Mic className="size-4" />
            </span>
            <div className="min-w-0 flex-1">
              <Waveform className="w-full" level={state.audioLevel} />
              {state.partialTranscript ? (
                <LiveTranscript text={state.partialTranscript} stableWords={state.stableWords} />
              ) : (
                <p className="truncate text-[10px] leading-4 text-white/24">Listening locally…</p>
              )}
            </div>
            <span className="ml-auto min-w-11 font-mono text-[11px] tabular-nums text-white/42">
              {formatDuration(state.elapsedMs)}
            </span>
          </div>
        )}

        {state.phase === "cancelPending" && (
          <div className="flex w-full items-center gap-3.5">
            <CountdownRing />
            <div className="min-w-0 flex-1">
              <p className="text-[13px] font-medium text-white/90">Cancelling…</p>
              <p className="mt-0.5 truncate text-[11px] text-white/42">Press Esc again to keep recording</p>
            </div>
          </div>
        )}

        {(state.phase === "transcribing" || state.phase === "delivering") && (
          <div className="flex w-full items-center gap-4">
            <Waveform level={0.32} processing />
            <div className="min-w-0">
              <p className="vox-shimmer-text text-[13px] font-medium">
                {state.phase === "transcribing" ? "Transcribing locally" : "Pasting at your cursor"}
              </p>
              <p className="mt-0.5 text-[11px] text-white/38">Audio never leaves this Mac</p>
            </div>
          </div>
        )}

        {state.phase === "success" && (
          <div className="flex w-full items-center gap-3.5">
            <span className="grid size-9 place-items-center rounded-full bg-emerald-300/12 text-emerald-300">
              {state.deliveryMode === "clipboard" ? <Clipboard className="size-4" /> : <Check className="size-4" />}
            </span>
            <div className="min-w-0">
              <p className="text-[13px] font-medium text-white/90">{state.message ?? "Done"}</p>
              {state.lastTranscript && <p className="mt-0.5 truncate text-[11px] text-white/38">{state.lastTranscript}</p>}
            </div>
          </div>
        )}

        {state.phase === "error" && (
          <div className="flex w-full items-center gap-3">
            <AlertTriangle className="size-5 shrink-0 text-rose-300" />
            <p className="line-clamp-2 min-w-0 flex-1 text-[11px] leading-4 text-white/65">
              {state.message ?? "Vox could not finish this dictation."}
            </p>
            <Button className="size-8 !px-0" variant="ghost" icon={<RotateCcw className="size-3.5" />} onClick={() => void vox.retry()} aria-label="Retry" />
            <Button className="size-8 !px-0" variant="ghost" icon={<X className="size-3.5" />} onClick={() => void vox.dismiss()} aria-label="Dismiss" />
          </div>
        )}

        {state.phase === "idle" && (
          <div className="flex w-full items-center gap-3 text-white/45">
            <Mic className="size-4" />
            <span className="text-xs">Vox is ready</span>
          </div>
        )}
      </section>
    </main>
  );
}
