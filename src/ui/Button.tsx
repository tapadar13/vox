import type { ButtonHTMLAttributes, ReactNode } from "react";

type Variant = "primary" | "secondary" | "ghost" | "danger";

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: Variant;
  icon?: ReactNode;
  busy?: boolean;
}

const variants: Record<Variant, string> = {
  primary: "bg-[var(--vox-accent-strong)] text-white shadow-[0_8px_24px_rgba(124,103,255,.25)] hover:bg-[#8b79ff]",
  secondary: "border border-white/10 bg-white/[.07] text-white hover:bg-white/[.11]",
  ghost: "text-white/60 hover:bg-white/[.07] hover:text-white",
  danger: "border border-rose-400/20 bg-rose-400/10 text-rose-200 hover:bg-rose-400/15",
};

export function Button({
  variant = "secondary",
  icon,
  busy,
  className = "",
  children,
  disabled,
  ...props
}: ButtonProps) {
  return (
    <button
      className={`inline-flex h-9 items-center justify-center gap-2 rounded-xl px-3.5 text-[13px] font-medium transition duration-150 disabled:cursor-not-allowed disabled:opacity-45 ${variants[variant]} ${className}`}
      disabled={disabled || busy}
      {...props}
    >
      {busy ? <span className="size-3.5 animate-spin rounded-full border-2 border-current border-r-transparent" /> : icon}
      {children}
    </button>
  );
}
