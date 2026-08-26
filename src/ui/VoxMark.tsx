interface VoxMarkProps {
  className?: string;
  compact?: boolean;
  variant?: "sidebar" | "hero" | "app";
}

const accent =
  "linear-gradient(135deg, oklab(0.708 0.16 0.092) 0%, oklab(0.686 0.218 0.012) 52%, oklab(0.606 0.085 -0.202) 100%)";

export function VoxMark({ className = "", compact = false, variant }: VoxMarkProps) {
  const resolved = variant ?? (compact ? "sidebar" : "sidebar");
  const hero = resolved === "hero";
  const app = resolved === "app";
  const heights = hero ? [27, 53, 38, 62, 24] : app ? [26, 48, 64, 42, 22] : [8, 15, 11, 17, 7];
  const colors = ["#ff6b57", "#ff4d8d", "#cd55c3", "#a65ce5", "#8b5cf6"];

  return (
    <div
      className={`${hero ? "size-28 rounded-[32px] shadow-[inset_0_1px_0_rgba(255,255,255,.32),0_20px_42px_rgba(196,73,181,.25)]" : app ? "size-32 rounded-[31px] shadow-[inset_0_1px_0_rgba(255,255,255,.13),0_22px_42px_rgba(37,31,48,.25)]" : "size-[34px] rounded-[11px] shadow-[0_8px_18px_rgba(215,70,146,.22)]"} grid shrink-0 place-items-center ${className}`}
      style={{
        background: app
          ? "linear-gradient(145deg, oklab(0.285 0.003 -0.024), oklab(0.188 0.0006 -0.015) 70%)"
          : accent,
      }}
      aria-hidden="true"
    >
      <div className={`flex items-center ${hero || app ? "gap-[5px]" : "gap-0.5"}`}>
        {heights.map((height, index) => (
          <span
            key={`${height}-${index}`}
            className={`${hero || app ? "w-[7px] rounded-[7px]" : "w-[3px] rounded-[3px]"} shrink-0`}
            style={{
              height,
              backgroundColor: app ? colors[index] : "white",
              opacity: hero && (index === 0 || index === 4) ? 0.92 : 1,
            }}
          />
        ))}
      </div>
    </div>
  );
}
