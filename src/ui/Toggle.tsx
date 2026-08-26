interface ToggleProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  label: string;
  description?: string;
  disabled?: boolean;
}

export function Toggle({ checked, onChange, label, description, disabled }: ToggleProps) {
  return (
    <label className="flex cursor-pointer items-center justify-between gap-4 py-2">
      <span>
        <span className="block text-[10px] font-[550] leading-[14px] text-[#343843]">{label}</span>
        {description && <span className="mt-0.5 block text-[8px] leading-[11px] text-[#8a909d]">{description}</span>}
      </span>
      <input
        className="peer sr-only"
        type="checkbox"
        aria-label={label}
        checked={checked}
        disabled={disabled}
        onChange={(event) => onChange(event.target.checked)}
      />
      <span className="relative h-[18px] w-[31px] shrink-0 rounded-full bg-[#c7cbd3] transition peer-checked:bg-[linear-gradient(90deg,oklab(0.708_0.16_0.092),oklab(0.686_0.218_0.012)_56%,oklab(0.606_0.085_-0.202))] peer-disabled:opacity-40 after:absolute after:left-0.5 after:top-0.5 after:size-3.5 after:rounded-full after:bg-white after:shadow-[0_1px_3px_rgba(36,26,47,.24)] after:transition-transform peer-checked:after:translate-x-[13px]" />
    </label>
  );
}
