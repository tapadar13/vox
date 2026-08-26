import type { ButtonHTMLAttributes, ReactNode } from "react";

type Variant = "primary" | "secondary" | "ghost" | "danger";

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: Variant;
  icon?: ReactNode;
  busy?: boolean;
}

const variants: Record<Variant, string> = {
  primary: "vox-paper-gradient text-white shadow-[0_8px_18px_rgba(186,66,166,.16)] hover:brightness-[1.03]",
  secondary: "bg-[#eceef3] text-[#555b68] shadow-[inset_0_0_0_1px_rgba(55,60,75,.05)] hover:bg-[#e5e7ed]",
  ghost: "text-[#777e8d] hover:bg-white/50 hover:text-[#343843]",
  danger: "bg-[#fff0f3] text-[#b34d66] shadow-[inset_0_0_0_1px_rgba(255,117,138,.18)] hover:bg-[#ffe7ec]",
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
      className={`inline-flex h-9 items-center justify-center gap-2 rounded-xl px-3.5 text-[12px] font-semibold transition duration-150 disabled:cursor-not-allowed disabled:opacity-45 ${variants[variant]} ${className}`}
      disabled={disabled || busy}
      {...props}
    >
      {busy ? <span className="size-3.5 animate-spin rounded-full border-2 border-current border-r-transparent" /> : icon}
      {children}
    </button>
  );
}
