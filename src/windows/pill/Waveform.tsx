interface WaveformProps {
  level: number;
  processing?: boolean;
  className?: string;
}

const baseHeights = [9, 16, 25, 9, 21, 14, 8];
const colors = ["#ff6b57", "#ff5c71", "#ff4d8d", "#f867a9", "#c660ca", "#a25deb", "#8b5cf6"];

export function Waveform({ level, processing = false, className = "" }: WaveformProps) {
  const energy = 0.72 + Math.min(1, level) * 0.42;
  return (
    <div className={`flex h-7 items-center gap-1 ${processing ? "animate-pulse" : ""} ${className}`} aria-label="Live microphone level">
      {baseHeights.map((height, index) => (
        <span
          key={`${height}-${index}`}
          className={`${index === 3 ? "size-[9px] rounded-full shadow-[0_0_12px_rgba(255,77,141,.85)]" : "w-[5px] rounded-lg"} shrink-0 transition-[height] duration-100`}
          style={{ height: index === 3 ? 9 : Math.max(6, Math.round(height * energy)), backgroundColor: colors[index] }}
        />
      ))}
    </div>
  );
}
