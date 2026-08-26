import { openUrl } from "@tauri-apps/plugin-opener";
import { useEffect, useMemo, useRef, useState } from "react";

import { formatBytes, formatHotkey } from "../../lib/format";
import { isTauri, onModelProgress, vox } from "../../lib/tauri";
import type { ManagedModel, ModelDownloadProgress, Settings } from "../../lib/types";
import { VoxMark } from "../../ui/VoxMark";

interface OnboardingProps {
  settings: Settings;
  onComplete: (settings: Settings) => void;
}

export function Onboarding({ settings, onComplete }: OnboardingProps) {
  const [step, setStep] = useState(0);
  const [micGranted, setMicGranted] = useState(false);
  const [models, setModels] = useState<ManagedModel[]>([]);
  const [downloading, setDownloading] = useState(false);
  const [downloaded, setDownloaded] = useState(false);
  const [progress, setProgress] = useState<ModelDownloadProgress | null>(null);
  const [error, setError] = useState<string | null>(null);
  const attemptedDownload = useRef(false);

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
    () => models.find((model) => model.id === settings.modelId) ?? models[0],
    [models, settings.modelId],
  );
  const modelReady = Boolean(selectedModel?.installed || downloaded);

  useEffect(() => {
    if (step !== 2 || !selectedModel || selectedModel.installed || attemptedDownload.current) return;
    attemptedDownload.current = true;
    setDownloading(true);
    setError(null);
    void vox.downloadModel(selectedModel.id)
      .then(() => vox.selectModel(selectedModel.id))
      .then(() => setDownloaded(true))
      .catch((caught) => setError(caught instanceof Error ? caught.message : String(caught)))
      .finally(() => setDownloading(false));
  }, [selectedModel, step]);

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

  const finish = async () => {
    const complete = { ...settings, onboardingComplete: true };
    setError(null);
    try {
      await vox.updateSettings(complete);
      onComplete(complete);
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : String(caught));
    }
  };

  return (
    <main className="vox-paper-background h-screen w-screen overflow-hidden">
      <section className="relative flex h-full w-full flex-col items-center overflow-hidden rounded-[26px] border border-white/90 bg-white/70 px-7 py-6 text-[#191b23] shadow-[inset_0_1px_0_rgba(255,255,255,.9),0_24px_54px_rgba(51,57,78,.14)]">
        <header className="flex h-[22px] w-full shrink-0 items-center justify-between" data-tauri-drag-region>
          <span className="text-[10px] font-[650] leading-[14px] tracking-[.12em] text-[#7e8491]">VOX SETUP</span>
          <span className="text-[10px] font-semibold leading-[14px] text-[#a0a5af]">0{step + 1} / 03</span>
        </header>

        {error && <p className="absolute inset-x-7 top-12 z-10 rounded-lg bg-[#fff0f3] px-2.5 py-1.5 text-center text-[9px] text-[#a9445d] shadow-sm">{error}</p>}

        {step === 0 && <WelcomeStep onContinue={() => setStep(1)} />}
        {step === 1 && (
          <PermissionsStep
            micGranted={micGranted}
            onMicrophone={() => void askForMicrophone()}
            onAccessibility={() => void openAccessibility()}
            onContinue={() => setStep(2)}
          />
        )}
        {step === 2 && (
          <DownloadStep
            hotkey={settings.hotkey}
            model={selectedModel}
            downloading={downloading}
            ready={modelReady}
            progress={progress}
            onFinish={() => void finish()}
          />
        )}
      </section>
    </main>
  );
}

function WelcomeStep({ onContinue }: { onContinue: () => void }) {
  return (
    <>
      <div className="grid h-[188px] w-full shrink-0 place-items-center"><VoxMark variant="hero" /></div>
      <div className="flex h-[110px] w-full shrink-0 flex-col items-center gap-2.5 text-center">
        <h1 className="text-[29px] font-[680] leading-[34px] tracking-[-.045em] text-[#161820]">Type with your voice.<br />Everywhere.</h1>
        <p className="text-xs leading-[18px] text-[#6f7684]">Press one hotkey, speak naturally, and Vox pastes polished text wherever your cursor is.</p>
      </div>
      <div className="flex h-[45px] w-full shrink-0 items-center justify-center gap-2.5 text-[9px] font-semibold leading-3 text-[#616876]">
        <span>ANY APP</span><Dot /><span>100% LOCAL</span><Dot /><span>INSTANT</span>
      </div>
      <div className="flex h-[70px] w-full shrink-0 items-center justify-center">
        <button type="button" className="vox-paper-gradient flex h-12 w-full items-center justify-center gap-2 rounded-[15px] text-[13px] font-[650] text-white shadow-[0_12px_24px_rgba(201,69,171,.22)]" onClick={onContinue}>Continue <span className="text-[17px]">→</span></button>
      </div>
      <StepDots step={0} className="h-9" />
    </>
  );
}

function PermissionsStep({ micGranted, onMicrophone, onAccessibility, onContinue }: { micGranted: boolean; onMicrophone: () => void; onAccessibility: () => void; onContinue: () => void }) {
  return (
    <>
      <div className="flex h-[158px] w-full shrink-0 flex-col items-center justify-center gap-[11px] text-center">
        <div className="grid size-[78px] shrink-0 place-items-center rounded-[26px] bg-[linear-gradient(145deg,oklab(0.708_0.16_0.092/.18),oklab(0.686_0.218_0.012/.18)_48%,oklab(0.606_0.085_-0.202/.2))] shadow-[inset_0_0_0_1px_rgba(255,255,255,.75)]"><PermissionHero /></div>
        <h1 className="text-2xl font-[670] leading-[29px] tracking-[-.035em]">Two small permissions</h1>
        <p className="text-[11px] leading-4 text-[#757c89]">Vox needs only what makes dictation work.</p>
      </div>
      <div className="flex h-[178px] w-full shrink-0 flex-col gap-[9px]">
        <PermissionRow type="microphone" title="Microphone" detail="To hear you speak" granted={micGranted} action="Grant access" onClick={onMicrophone} />
        <PermissionRow type="accessibility" title="Accessibility" detail="So Vox can paste for you" action="Grant access" onClick={onAccessibility} tall />
      </div>
      <div className="flex h-[114px] w-full shrink-0 flex-col items-center justify-end gap-[17px]">
        <button type="button" className="text-[10px] font-[550] leading-[14px] text-[#8b6bab]" onClick={onContinue}>Skip for now</button>
        <StepDots step={1} />
      </div>
    </>
  );
}

function DownloadStep({ hotkey, model, downloading, ready, progress, onFinish }: { hotkey: string; model?: ManagedModel; downloading: boolean; ready: boolean; progress: ModelDownloadProgress | null; onFinish: () => void }) {
  const fraction = ready ? 1 : progress?.fraction ?? 0;
  const percentage = Math.round(fraction * 100);
  const downloadedBytes = progress?.downloadedBytes ?? 0;
  const totalBytes = progress?.totalBytes ?? model?.sizeBytes ?? 600_000_000;
  return (
    <>
      <div className="grid h-[190px] w-full shrink-0 place-items-center">
        <div className="grid size-[132px] shrink-0 place-items-center rounded-full shadow-[0_18px_40px_rgba(149,76,199,.17)]" style={{ background: `conic-gradient(#ff4d8d 0%, #d74692 ${percentage / 2}%, #8b5cf6 ${percentage}%, #eceaf0 ${percentage}%, #eceaf0 100%)` }}>
          <div className="flex size-28 flex-col items-center justify-center rounded-full bg-white/95"><DownloadIcon /><span className="text-base font-[680] leading-5 text-[#343843]">{percentage}%</span></div>
        </div>
      </div>
      <div className="flex h-[92px] w-full shrink-0 flex-col items-center gap-2 text-center">
        <h1 className="text-[23px] font-[670] leading-7 tracking-[-.035em]">{ready ? "Whisper Turbo is ready" : "Downloading Whisper Turbo…"}</h1>
        <p className="text-[11px] font-[550] leading-[15px] text-[#6e7582]">{formatBytes(downloadedBytes)} of {formatBytes(totalBytes)}</p>
        <p className="text-[10px] leading-[15px] text-[#8a909d]">One-time download. After this, everything is offline.</p>
      </div>
      <button type="button" className="flex h-[122px] w-full shrink-0 items-center justify-between gap-3 rounded-[18px] bg-white/75 px-[15px] py-3.5 text-left shadow-[inset_0_0_0_1px_rgba(255,255,255,.88)] disabled:opacity-55" disabled={!ready || downloading} onClick={onFinish}>
        <span className="flex flex-col gap-[5px]"><span className="text-[13px] font-[650] leading-[17px] text-[#30343e]">Try it now</span><span className="text-[9px] leading-[13px] text-[#7e8592]">Press the hotkey and say hello.</span></span>
        <span className="grid h-12 place-items-center rounded-[13px] bg-[#f5f6f8] px-[15px] text-base font-[670] text-[#454b58] shadow-[inset_0_0_0_1px_rgba(53,58,71,.11),0_3px_8px_rgba(25,27,33,.06)]">{formatHotkey(hotkey)}</span>
      </button>
      <StepDots step={2} className="h-[46px]" />
    </>
  );
}

function PermissionRow({ type, title, detail, action, granted, tall, onClick }: { type: "microphone" | "accessibility"; title: string; detail: string; action: string; granted?: boolean; tall?: boolean; onClick: () => void }) {
  return (
    <div className={`flex w-full shrink-0 items-center gap-[11px] rounded-2xl bg-white/75 px-3 py-[11px] shadow-[inset_0_0_0_1px_rgba(255,255,255,.88)] ${tall ? "h-[88px]" : "h-[74px]"}`}>
      <span className={`grid size-9 shrink-0 place-items-center rounded-xl ${type === "microphone" ? "bg-[#42b0671a]" : "bg-[#8b5cf617]"}`}>{type === "microphone" ? <MicrophoneIcon /> : <AccessibilityIcon />}</span>
      <span className="min-w-0 flex-1"><span className="block text-xs font-[620] leading-4 text-[#30343e]">{title}</span><span className="mt-[3px] block text-[9px] leading-3 text-[#838996]">{detail}</span></span>
      {granted ? <span className="text-[9px] font-[650] text-[#3a8d58]">Granted ✓</span> : <button type="button" className="vox-paper-gradient h-7 rounded-[9px] px-2.5 text-[9px] font-[650] text-white" onClick={onClick}>{action}</button>}
    </div>
  );
}

function StepDots({ step, className = "" }: { step: number; className?: string }) {
  return <div className={`flex w-full shrink-0 items-center justify-center gap-1.5 ${className}`}>{[0, 1, 2].map((index) => <span key={index} className={`${index === step ? "h-1.5 w-5 bg-[#cf56c4]" : "size-1.5 bg-[#d8dbe1]"} rounded-full`} />)}</div>;
}

function Dot() { return <span className="size-[3px] rounded-full bg-[#c3c7cf]" />; }
function PermissionHero() { return <svg width="37" height="42" viewBox="0 0 38 43" aria-hidden="true"><path d="M19 3 33 8.3v10.6c0 8.7-5.8 16.3-14 20.5C10.8 35.2 5 27.6 5 18.9V8.3L19 3Z" fill="none" stroke="#b34dc2" strokeWidth="2" /><rect x="15" y="11" width="8" height="14" rx="4" fill="none" stroke="#ff5a83" strokeWidth="2" /><path d="M11 20a8 8 0 0 0 16 0M19 28v4" fill="none" stroke="#8b5cf6" strokeWidth="2" strokeLinecap="round" /></svg>; }
function MicrophoneIcon() { return <svg width="16" height="21" viewBox="0 0 17 22" aria-hidden="true"><rect x="5" y="1" width="7" height="13" rx="3.5" fill="none" stroke="#3a9e60" strokeWidth="1.5" /><path d="M2.5 10.5a6 6 0 0 0 12 0M8.5 16.5V20" fill="none" stroke="#3a9e60" strokeWidth="1.5" strokeLinecap="round" /></svg>; }
function AccessibilityIcon() { return <svg width="20" height="20" viewBox="0 0 21 21" aria-hidden="true"><circle cx="10.5" cy="4" r="2.2" fill="none" stroke="#8f5cd1" strokeWidth="1.5" /><path d="M3 8h15M10.5 8v10M6 18l4.5-6 4.5 6" fill="none" stroke="#8f5cd1" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" /></svg>; }
function DownloadIcon() { return <svg width="25" height="27" viewBox="0 0 26 28" aria-hidden="true"><path d="M13 3v14m0 0-5-5m5 5 5-5M4 22v3h18v-3" fill="none" stroke="#a257d3" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" /></svg>; }
