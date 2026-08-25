interface VoxMarkProps {
  className?: string;
  compact?: boolean;
}

export function VoxMark({ className = "", compact = false }: VoxMarkProps) {
  return (
    <div
      className={`grid place-items-center rounded-xl bg-gradient-to-br from-[#292447] to-[#0a0c14] shadow-[inset_0_0_0_1px_rgba(255,255,255,.1)] ${compact ? "size-8" : "size-10"} ${className}`}
      aria-hidden="true"
    >
      <div className="flex h-4 items-center gap-[2px]">
        {[7, 13, 18, 13, 7].map((height, index) => (
          <span
            key={`${height}-${index}`}
            className="w-[3px] rounded-full bg-gradient-to-b from-[#b9aeff] to-[#62d9bf]"
            style={{ height: compact ? height * 0.75 : height }}
          />
        ))}
      </div>
    </div>
  );
}
