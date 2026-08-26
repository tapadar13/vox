import { useEffect, useRef, useState } from "react";

import { vox } from "../../lib/tauri";
import type { LanguageHint, Settings as VoxSettings } from "../../lib/types";
import { Toggle } from "../../ui/Toggle";
import { HotkeyRecorder } from "./HotkeyRecorder";
import { ModelManager } from "./ModelManager";
import { UpdateCard } from "./UpdateCard";

interface SettingsProps {
  value: VoxSettings;
  onSaved: (settings: VoxSettings) => void;
}

const languages = [
  ["auto", "Auto-detect"], ["en", "English"], ["hi", "हिन्दी"],
  ["es", "Español"], ["fr", "Français"], ["ar", "العربية"],
] as const;

export function Settings({ value, onSaved }: SettingsProps) {
  const [draft, setDraft] = useState(value);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const saveEpoch = useRef(0);

  useEffect(() => setDraft(value), [value]);

  const persist = <Key extends keyof VoxSettings>(key: Key, nextValue: VoxSettings[Key]) => {
    const next = { ...draft, [key]: nextValue };
    const epoch = ++saveEpoch.current;
    setDraft(next);
    setSaving(true);
    setError(null);
    void vox.updateSettings(next).then(() => {
      if (epoch === saveEpoch.current) {
        onSaved(next);
        setSaving(false);
      }
    }).catch((caught) => {
      if (epoch === saveEpoch.current) {
        setSaving(false);
        setError(caught instanceof Error ? caught.message : String(caught));
      }
    });
  };

  const selectModel = (id: string) => {
    persist("modelId", id);
    void vox.selectModel(id).catch((caught) => setError(String(caught)));
  };

  const languageValue = draft.language.mode === "auto" ? "auto" : draft.language.language;
  const setLanguage = (next: string) => {
    const language: LanguageHint = next === "auto" ? { mode: "auto" } : { mode: "pinned", language: next };
    persist("language", language);
  };

  return (
    <div className="relative flex h-full flex-col gap-2.5 overflow-hidden px-5 py-[18px]">
      <header className="flex h-[30px] shrink-0 items-end justify-between" data-tauri-drag-region>
        <h1 className="text-2xl font-[650] leading-7 tracking-[-.035em] text-[#11131a]">Settings</h1>
        <p className="text-[9px] leading-3 text-[#8a909d]">{saving ? "Saving locally…" : "Vox v0.1.0"}</p>
      </header>

      {error && <p className="absolute inset-x-5 top-[52px] z-20 rounded-lg bg-[#fff0f3] px-2.5 py-1.5 text-[9px] text-[#a9445d] shadow-sm">{error}</p>}

      <div className="flex h-[326px] shrink-0 gap-2.5">
        <div className="flex h-full w-[285px] shrink-0 flex-col gap-2">
          <section className="flex h-[199px] shrink-0 flex-col rounded-[17px] bg-white/65 px-[13px] py-3 shadow-[inset_0_0_0_1px_rgba(255,255,255,.88)]">
            <h2 className="h-[22px] shrink-0 text-[11px] font-bold leading-[15px] tracking-[.09em] text-[#606775]">DICTATION</h2>

            <div className="flex h-[45px] shrink-0 items-center justify-between border-b border-[#343a480f]">
              <span className="text-[10px] font-[550] leading-[14px] text-[#343843]">Hotkey</span>
              <HotkeyRecorder value={draft.hotkey} onChange={(hotkey) => persist("hotkey", hotkey)} />
            </div>

            <label className="flex h-[88px] shrink-0 items-start justify-between border-b border-[#343a480f] pt-[11px]">
              <span className="flex flex-col gap-0.5 pt-1">
                <span className="text-[10px] font-[550] leading-[14px] text-[#343843]">Language</span>
                <span className="text-[8px] leading-[11px] text-[#8a909d]">{languageValue === "auto" ? "Auto-detect" : languageValue.toUpperCase()}</span>
              </span>
              <select className="h-7 w-[132px] rounded-[10px] bg-white/95 px-2 text-[9px] text-[#535966] shadow-[inset_0_0_0_1px_rgba(55,60,75,.07),0_8px_18px_rgba(42,47,63,.11)] outline-none" value={languageValue} onChange={(event) => setLanguage(event.target.value)}>
                {languages.map(([id, label]) => <option key={id} value={id}>{label}</option>)}
              </select>
            </label>

            <div className="flex h-8 shrink-0 items-center">
              <Toggle checked={draft.autoPaste} onChange={(checked) => persist("autoPaste", checked)} label="Auto-paste" />
            </div>
          </section>

          <section className="flex h-[119px] shrink-0 flex-col rounded-[17px] bg-white/60 px-[13px] py-[11px] shadow-[inset_0_0_0_1px_rgba(255,255,255,.86)]">
            <h2 className="h-[19px] shrink-0 text-[10px] font-bold leading-[14px] tracking-[.09em] text-[#666d7a]">GENERAL</h2>
            <div className="flex h-7 shrink-0 items-center border-b border-[#343a480f]">
              <Toggle checked={draft.launchAtLogin} onChange={(checked) => persist("launchAtLogin", checked)} label="Launch at login" />
            </div>
            <div className="flex h-[31px] shrink-0 items-center justify-between border-b border-[#343a480f]">
              <span className="text-[9px] text-[#3d424e]">Appearance</span>
              <div className="flex h-[21px] rounded-[7px] bg-[#eceef2] p-0.5" aria-label="Appearance follows macOS">
                <span className="rounded-[5px] bg-white px-1.5 py-0.5 text-[8px] leading-[13px] text-[#3f4450] shadow-[0_1px_3px_rgba(40,44,56,.1)]">Auto</span>
                <span className="px-[5px] py-0.5 text-[8px] leading-[13px] text-[#8a909d]">Light</span>
                <span className="px-[5px] py-0.5 text-[8px] leading-[13px] text-[#8a909d]">Dark</span>
              </div>
            </div>
            <UpdateCard />
          </section>
        </div>

        <section className="flex h-full min-w-0 flex-1 flex-col rounded-[17px] bg-white/65 px-[13px] py-3 shadow-[inset_0_0_0_1px_rgba(255,255,255,.88)]">
          <ModelManager selected={draft.modelId} onSelect={selectModel} />
        </section>
      </div>

      <footer className="flex h-[26px] shrink-0 items-center justify-center gap-[7px] text-[9px] font-[550] leading-3 text-[#626977]">
        <ShieldIcon /> 100% local. Your voice never leaves this Mac.
      </footer>
    </div>
  );
}

function ShieldIcon() {
  return <svg width="12" height="14" viewBox="0 0 14 16" aria-hidden="true"><path d="M7 1.5 12 3.4v3.8c0 3.1-2.1 5.8-5 7.3-2.9-1.5-5-4.2-5-7.3V3.4L7 1.5Z" fill="none" stroke="#7d61b0" strokeWidth="1.2" /><path d="m4.8 7.6 1.4 1.5 3.3-3.3" fill="none" stroke="#7d61b0" strokeWidth="1.2" strokeLinecap="round" strokeLinejoin="round" /></svg>;
}
