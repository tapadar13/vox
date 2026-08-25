import { openUrl } from "@tauri-apps/plugin-opener";
import {
  ArrowLeft,
  ArrowRight,
  Check,
  Download,
  Keyboard,
  LockKeyhole,
  Mic,
  ShieldCheck,
  Sparkles,
} from "lucide-react";
import { useEffect, useMemo, useState, type ReactNode } from "react";

import { formatBytes, formatHotkey } from "../../lib/format";
import { isTauri, onModelProgress, vox } from "../../lib/tauri";
import type { ManagedModel, ModelDownloadProgress, Settings } from "../../lib/types";
import { Button } from "../../ui/Button";
import { VoxMark } from "../../ui/VoxMark";
import { HotkeyRecorder } from "./HotkeyRecorder";

interface OnboardingProps {
  settings: Settings;
  onComplete: (settings: Settings) => void;
}

const steps = ["Welcome", "Microphone", "Accessibility", "Model", "Ready"] as const;

export function Onboarding({ settings, onComplete }: OnboardingProps) {
  const [step, setStep] = useState(0);
  const [draft, setDraft] = useState(settings);
  const [micGranted, setMicGranted] = useState(false);
  const [models, setModels] = useState<ManagedModel[]>([]);
  const [downloading, setDownloading] = useState(false);
  const [progress, setProgress] = useState<ModelDownloadProgress | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void vox.models().then(setModels);
    let mounted = true;
    let unlisten: () => void = () => undefined;
    void onModelProgress((next) => mounted && setProgress(next)).then((stop) => {
      if (mounted) unlisten = stop;
      else stop();
    });
    return () => {
      mounted = false;
      unlisten();
    };
  }, []);

  const selectedModel = useMemo(
    () => models.find((model) => model.id === draft.modelId),
    [draft.modelId, models],
  );

  const askForMicrophone = async () => {
    setError(null);
    try {
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
      stream.getTracks().forEach((track) => track.stop());
      setMicGranted(true);
    } catch {
      setError("Microphone access was not granted. You can enable it later in System Settings.");
    }
  };

  const openAccessibility = async () => {
    setError(null);
    if (!isTauri()) return;
    try {
      await openUrl("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility");
    } catch {
      setError("Open System Settings → Privacy & Security → Accessibility, then enable Vox.");
    }
  };

  const chooseModel = (modelId: string) => {
    setDraft((current) => ({ ...current, modelId }));
  };

  const downloadSelectedModel = async () => {
    if (!selectedModel) return;
    setDownloading(true);
    setError(null);
    try {
      await vox.downloadModel(selectedModel.id);
      await vox.selectModel(selectedModel.id);
      setModels(await vox.models());
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setDownloading(false);
      setProgress(null);
    }
  };

  const finish = async () => {
    setError(null);
    const complete = { ...draft, onboardingComplete: true };
    try {
      await vox.updateSettings(complete);
      onComplete(complete);
    } catch (caught) {
      setError(errorMessage(caught));
    }
  };

  const advance = () => setStep((current) => Math.min(steps.length - 1, current + 1));
  const back = () => setStep((current) => Math.max(0, current - 1));

  return (
    <main className="vox-dashboard grid min-h-screen place-items-center overflow-hidden p-8">
      <div className="w-full max-w-[570px]">
        <div className="mb-6 flex items-center justify-center gap-2" aria-label={`Step ${step + 1} of ${steps.length}`}>
          {steps.map((label, index) => (
            <span
              key={label}
              className={`h-1 rounded-full transition-all ${index === step ? "w-8 bg-violet-300" : index < step ? "w-4 bg-violet-300/45" : "w-4 bg-white/10"}`}
            />
          ))}
        </div>

        <section className="rounded-[28px] border border-white/[.09] bg-[#10131de6] p-8 shadow-[0_30px_100px_rgba(0,0,0,.48)] backdrop-blur-2xl">
          {step === 0 && (
            <OnboardingStep
              icon={<VoxMark />}
              eyebrow="Welcome to Vox"
              title="Turn your voice into text. Privately."
              body="Vox records only while you hold the shortcut, transcribes entirely on this Mac, and never uploads your audio."
            >
              <div className="mt-6 grid grid-cols-3 gap-2.5 text-center text-[10px] text-white/42">
                <Feature icon={<LockKeyhole />} label="100% local" />
                <Feature icon={<Keyboard />} label="Works anywhere" />
                <Feature icon={<Sparkles />} label="Ready in seconds" />
              </div>
            </OnboardingStep>
          )}

          {step === 1 && (
            <OnboardingStep
              icon={<Mic />}
              eyebrow="Microphone"
              title="Vox needs to hear you."
              body="macOS will ask for microphone access. Vox processes the recording locally and discards the audio after transcription."
            >
              <Button
                className="mt-6 w-full"
                variant={micGranted ? "secondary" : "primary"}
                icon={micGranted ? <Check className="size-4 text-emerald-300" /> : <Mic className="size-4" />}
                onClick={() => void askForMicrophone()}
              >
                {micGranted ? "Microphone allowed" : "Allow microphone access"}
              </Button>
            </OnboardingStep>
          )}

          {step === 2 && (
            <OnboardingStep
              icon={<ShieldCheck />}
              eyebrow="Accessibility"
              title="Paste straight into any app."
              body="Accessibility access lets Vox paste at your cursor. It is optional—without it, your transcription is copied to the clipboard."
            >
              <Button
                className="mt-6 w-full"
                variant="primary"
                icon={<ShieldCheck className="size-4" />}
                onClick={() => void openAccessibility()}
              >
                Open Accessibility settings
              </Button>
              <p className="mt-3 text-center text-[10px] text-white/28">You can continue without granting access.</p>
            </OnboardingStep>
          )}

          {step === 3 && (
            <OnboardingStep
              icon={<Download />}
              eyebrow="Local speech model"
              title="Choose your balance."
              body="Download one verified Whisper model. The balanced model is a good first choice; Turbo gives the best multilingual accuracy."
            >
              <div className="mt-5 space-y-2">
                {models.map((model) => (
                  <button
                    key={model.id}
                    type="button"
                    className={`flex w-full items-center rounded-xl border p-3 text-left transition ${model.id === draft.modelId ? "border-violet-300/35 bg-violet-300/[.08]" : "border-white/[.07] bg-black/10 hover:border-white/15"}`}
                    onClick={() => chooseModel(model.id)}
                  >
                    <span className="min-w-0 flex-1">
                      <span className="block text-[12px] font-medium text-white/82">{model.name}</span>
                      <span className="mt-0.5 block text-[9px] text-white/32">{formatBytes(model.sizeBytes)} · {model.speed}</span>
                    </span>
                    {model.installed && <span className="text-[9px] text-emerald-300/70">Installed</span>}
                    {model.id === draft.modelId && !model.installed && <span className="size-2 rounded-full bg-violet-300" />}
                  </button>
                ))}
              </div>
              {selectedModel && !selectedModel.installed && (
                <div className="mt-3">
                  <Button className="w-full" variant="primary" busy={downloading} onClick={() => void downloadSelectedModel()}>
                    {downloading && progress
                      ? `Downloading ${Math.round(progress.fraction * 100)}%`
                      : `Download ${formatBytes(selectedModel.sizeBytes)}`}
                  </Button>
                  {downloading && progress && (
                    <div className="mt-2 h-1 overflow-hidden rounded-full bg-white/10">
                      <div className="h-full rounded-full bg-violet-300 transition-[width]" style={{ width: `${progress.fraction * 100}%` }} />
                    </div>
                  )}
                </div>
              )}
            </OnboardingStep>
          )}

          {step === 4 && (
            <OnboardingStep
              icon={<Check />}
              eyebrow="You’re ready"
              title="Speak naturally. Vox handles the rest."
              body="Press your shortcut once to start, speak, then press it again to transcribe and paste. Press Escape during recording to cancel."
            >
              <div className="mt-6 rounded-2xl border border-violet-300/15 bg-violet-300/[.06] p-4 text-center">
                <p className="text-[9px] uppercase tracking-[.18em] text-white/30">Your shortcut</p>
                <div className="mt-2 flex items-center justify-center gap-3">
                  <HotkeyRecorder value={draft.hotkey} onChange={(hotkey) => setDraft((current) => ({ ...current, hotkey }))} />
                  <span className="text-[10px] text-white/28">{formatHotkey(draft.hotkey)}</span>
                </div>
              </div>
            </OnboardingStep>
          )}

          {error && <p className="mt-4 rounded-xl bg-rose-400/10 px-3 py-2 text-[10px] text-rose-200/80">{error}</p>}

          <footer className="mt-7 flex items-center justify-between border-t border-white/[.06] pt-5">
            <Button variant="ghost" icon={<ArrowLeft className="size-3.5" />} onClick={back} disabled={step === 0}>
              Back
            </Button>
            {step < steps.length - 1 ? (
              <Button variant="primary" icon={<ArrowRight className="size-3.5" />} onClick={advance}>
                Continue
              </Button>
            ) : (
              <Button variant="primary" icon={<Check className="size-3.5" />} onClick={() => void finish()}>
                Start using Vox
              </Button>
            )}
          </footer>
        </section>
      </div>
    </main>
  );
}

function OnboardingStep({
  icon,
  eyebrow,
  title,
  body,
  children,
}: {
  icon: ReactNode;
  eyebrow: string;
  title: string;
  body: string;
  children: ReactNode;
}) {
  return (
    <div>
      <div className="mx-auto grid size-14 place-items-center rounded-2xl border border-white/10 bg-white/[.055] text-violet-200 [&>svg]:size-6">
        {icon}
      </div>
      <p className="mt-5 text-center text-[10px] font-medium uppercase tracking-[.19em] text-violet-300/65">{eyebrow}</p>
      <h1 className="mx-auto mt-2 max-w-md text-center text-[26px] font-semibold leading-tight tracking-tight">{title}</h1>
      <p className="mx-auto mt-3 max-w-md text-center text-[12px] leading-relaxed text-white/42">{body}</p>
      {children}
    </div>
  );
}

function Feature({ icon, label }: { icon: ReactNode; label: string }) {
  return (
    <div className="rounded-xl border border-white/[.07] bg-white/[.035] p-3">
      <div className="mx-auto mb-2 w-fit text-violet-200/70 [&>svg]:size-4">{icon}</div>
      {label}
    </div>
  );
}

function errorMessage(caught: unknown): string {
  return typeof caught === "object" && caught && "message" in caught
    ? String(caught.message)
    : String(caught);
}
