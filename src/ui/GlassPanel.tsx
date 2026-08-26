import type { HTMLAttributes } from "react";

export function GlassPanel({ className = "", ...props }: HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      className={`rounded-[17px] bg-white/60 shadow-[inset_0_0_0_1px_rgba(255,255,255,.86)] ${className}`}
      {...props}
    />
  );
}
