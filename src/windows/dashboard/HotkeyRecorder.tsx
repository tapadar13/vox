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
      className="flex min-w-[118px] flex-col items-end gap-1 text-right"
      onClick={() => setRecording(true)}
    >
      <span className={`rounded-[7px] px-[7px] py-1 text-[10px] font-[650] leading-3 transition ${recording ? "border border-dashed border-[#ff4d8d59] bg-[#ff4d8d0f] text-[#c04c91]" : "bg-[#f5f6f8] text-[#454b58] shadow-[inset_0_0_0_1px_rgba(53,58,71,.1),0_1px_1px_rgba(25,27,33,.05)]"}`}>
        {recording ? "Press shortcut…" : formatHotkey(value)}
      </span>
      <span className={`text-[8px] leading-[11px] ${recording ? "text-[#c04c91]" : "text-[#949aa6]"}`}>
        {recording ? "listening for keys…" : "click to re-record"}
      </span>
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
