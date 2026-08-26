import { Keyboard, Languages, Mic2, Save, SlidersHorizontal } from "lucide-react";
import { useEffect, useState } from "react";

import { vox } from "../../lib/tauri";
import type { LanguageHint, Settings as VoxSettings } from "../../lib/types";
import { Button } from "../../ui/Button";
import { GlassPanel } from "../../ui/GlassPanel";
import { Toggle } from "../../ui/Toggle";
import { HotkeyRecorder } from "./HotkeyRecorder";
import { ModelManager } from "./ModelManager";

interface SettingsProps {
  value: VoxSettings;
  onSaved: (settings: VoxSettings) => void;
}

const languages = [
  ["auto", "Auto-detect"],
  ["en", "English"],
  ["hi", "Hindi"],
  ["es", "Spanish"],
  ["it", "Italian"],
  ["fr", "French"],
  ["ar", "Arabic"],
  ["de", "German"],
  ["pt", "Portuguese"],
  ["ja", "Japanese"],
  ["zh", "Chinese"],
] as const;

export function Settings({ value, onSaved }: SettingsProps) {
  const [draft, setDraft] = useState(value);
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => setDraft(value), [value]);

  const patch = <Key extends keyof VoxSettings>(key: Key, next: VoxSettings[Key]) => {
    setDraft((current) => ({ ...current, [key]: next }));
    setSaved(false);
  };

  const save = async () => {
    setSaving(true);
    setError(null);
    try {
      await vox.updateSettings(draft);
      setSaved(true);
      onSaved(draft);
      window.setTimeout(() => setSaved(false), 1_400);
    } catch (caught) {
      setError(typeof caught === "object" && caught && "message" in caught ? String(caught.message) : String(caught));
    } finally {
      setSaving(false);
    }
  };

  const selectModel = (id: string) => {
    patch("modelId", id);
    void vox.selectModel(id).catch((caught) => setError(String(caught)));
  };

  const languageValue = draft.language.mode === "auto" ? "auto" : draft.language.language;
  const setLanguage = (value: string) => {
    const language: LanguageHint = value === "auto" ? { mode: "auto" } : { mode: "pinned", language: value };
    patch("language", language);
  };

  return (
    <div className="pb-4">
      <header className="flex items-end justify-between">
        <div>
          <h1 className="text-[24px] font-semibold tracking-tight">Settings</h1>
          <p className="mt-1 text-[11px] text-white/38">Tune Vox to the way you speak and work.</p>
        </div>
        <Button variant="primary" icon={<Save className="size-3.5" />} busy={saving} onClick={() => void save()}>
          {saved ? "Saved" : "Save changes"}
        </Button>
      </header>

      {error && <p className="mt-3 rounded-xl border border-rose-300/10 bg-rose-400/10 px-3 py-2 text-[10px] text-rose-200/80">{error}</p>}

      <div className="mt-4 grid gap-3">
        <GlassPanel className="p-4">
          <div className="mb-2 flex items-center gap-2 text-[11px] font-medium uppercase tracking-[.12em] text-white/38">
            <Keyboard className="size-3.5" /> Dictation
          </div>
          <div className="flex items-center justify-between border-b border-white/[.055] py-2.5">
            <div>
              <p className="text-[13px] font-medium text-white/88">Global hotkey</p>
              <p className="mt-0.5 text-[10px] text-white/34">Click, then press your preferred shortcut.</p>
            </div>
            <HotkeyRecorder value={draft.hotkey} onChange={(hotkey) => patch("hotkey", hotkey)} />
          </div>
          <Toggle
            checked={draft.autoPaste}
            onChange={(checked) => patch("autoPaste", checked)}
            label="Paste at the cursor"
            description="Falls back to the clipboard when Accessibility access is unavailable."
          />
          <div className="flex items-center justify-between border-t border-white/[.055] py-2.5">
            <div>
              <p className="text-[13px] font-medium text-white/88">Recording limit</p>
              <p className="mt-0.5 text-[10px] text-white/34">Automatically finish forgotten recordings.</p>
            </div>
            <select
              className="rounded-lg border border-white/[.09] bg-[#11131d] px-2.5 py-1.5 text-[11px] text-white/65 outline-none"
              value={draft.maxRecordingSeconds}
              onChange={(event) => patch("maxRecordingSeconds", Number(event.target.value))}
            >
              <option value={60}>1 minute</option>
              <option value={180}>3 minutes</option>
              <option value={300}>5 minutes</option>
              <option value={600}>10 minutes</option>
            </select>
          </div>
        </GlassPanel>

        <GlassPanel className="p-4">
          <div className="mb-2 flex items-center gap-2 text-[11px] font-medium uppercase tracking-[.12em] text-white/38">
            <Languages className="size-3.5" /> Language & text
          </div>
          <div className="flex items-center justify-between border-b border-white/[.055] py-2.5">
            <div>
              <p className="text-[13px] font-medium text-white/88">Language</p>
              <p className="mt-0.5 text-[10px] text-white/34">Pinning a language is a little faster and more accurate.</p>
            </div>
            <select
              className="rounded-lg border border-white/[.09] bg-[#11131d] px-2.5 py-1.5 text-[11px] text-white/65 outline-none"
              value={languageValue}
              onChange={(event) => setLanguage(event.target.value)}
            >
              {languages.map(([id, name]) => <option key={id} value={id}>{name}</option>)}
            </select>
          </div>
          <Toggle
            checked={draft.trimFillerWords}
            onChange={(checked) => patch("trimFillerWords", checked)}
            label="Trim filler words"
            description="Remove standalone “um”, “uh”, “erm”, and “hmm”. Off by default."
          />
        </GlassPanel>

        <GlassPanel className="p-4">
          <div className="mb-3 flex items-center gap-2 text-[11px] font-medium uppercase tracking-[.12em] text-white/38">
            <Mic2 className="size-3.5" /> Local model
          </div>
          <ModelManager selected={draft.modelId} onSelect={selectModel} />
        </GlassPanel>

        <GlassPanel className="p-4">
          <div className="mb-2 flex items-center gap-2 text-[11px] font-medium uppercase tracking-[.12em] text-white/38">
            <SlidersHorizontal className="size-3.5" /> System
          </div>
          <Toggle
            checked={draft.launchAtLogin}
            onChange={(checked) => patch("launchAtLogin", checked)}
            label="Launch Vox at login"
            description="Keep the menubar app ready without opening a dashboard window."
          />
          <div className="flex items-center justify-between border-t border-white/[.055] py-2.5">
            <div>
              <p className="text-[13px] font-medium text-white/88">Typing speed baseline</p>
              <p className="mt-0.5 text-[10px] text-white/34">Used only to estimate time saved.</p>
            </div>
            <label className="flex items-center gap-2 text-[11px] text-white/45">
              <input
                className="w-14 rounded-lg border border-white/[.09] bg-[#11131d] px-2 py-1.5 text-right text-white/65 outline-none"
                type="number"
                min={20}
                max={200}
                value={draft.typingWpm}
                onChange={(event) => patch("typingWpm", Number(event.target.value))}
              />
              WPM
            </label>
          </div>
        </GlassPanel>
      </div>
    </div>
  );
}
