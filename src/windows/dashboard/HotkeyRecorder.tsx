import { useEffect, useRef, useState } from "react";

import { formatHotkey } from "../../lib/format";

interface HotkeyRecorderProps {
  value: string;
  onChange: (value: string) => void;
}

export function HotkeyRecorder({ value, onChange }: HotkeyRecorderProps) {
  const [recording, setRecording] = useState(false);
  const buttonRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    if (!recording) return;
    const onKeyDown = (event: KeyboardEvent) => {
      event.preventDefault();
      event.stopPropagation();
      if (event.key === "Escape") {
        setRecording(false);
        return;
      }
      if (["Meta", "Control", "Alt", "Shift"].includes(event.key)) return;

      const modifiers = [
        event.metaKey && "Command",
        event.ctrlKey && "Control",
        event.altKey && "Alt",
        event.shiftKey && "Shift",
      ].filter(Boolean) as string[];
      const key = normalizeKey(event);
      if (!key || (modifiers.length === 0 && !key.startsWith("F"))) return;
      onChange([...modifiers, key].join("+"));
      setRecording(false);
    };
    window.addEventListener("keydown", onKeyDown, true);
    return () => window.removeEventListener("keydown", onKeyDown, true);
  }, [onChange, recording]);

  return (
    <button
      ref={buttonRef}
      type="button"
      className={`min-w-32 rounded-lg border px-3 py-1.5 text-center font-mono text-[11px] transition ${recording ? "border-violet-300/35 bg-violet-300/10 text-violet-200" : "border-white/[.09] bg-black/15 text-white/62 hover:border-white/15"}`}
      onClick={() => setRecording(true)}
    >
      {recording ? "Press shortcut…" : formatHotkey(value)}
    </button>
  );
}

function normalizeKey(event: KeyboardEvent): string | null {
  if (event.code === "Space") return "Space";
  if (event.code.startsWith("Key")) return event.code.slice(3);
  if (event.code.startsWith("Digit")) return event.code.slice(5);
  if (/^F\d{1,2}$/.test(event.code)) return event.code;
  const aliases: Record<string, string> = {
    Enter: "Enter",
    Tab: "Tab",
    Backspace: "Backspace",
    ArrowUp: "ArrowUp",
    ArrowDown: "ArrowDown",
    ArrowLeft: "ArrowLeft",
    ArrowRight: "ArrowRight",
  };
  return aliases[event.key] ?? null;
}
