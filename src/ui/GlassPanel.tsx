import type { HTMLAttributes } from "react";

export function GlassPanel({ className = "", ...props }: HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      className={`rounded-2xl border border-white/[.08] bg-white/[.045] shadow-[inset_0_1px_0_rgba(255,255,255,.035)] ${className}`}
      {...props}
    />
  );
}
