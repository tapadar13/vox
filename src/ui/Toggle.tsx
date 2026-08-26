interface ToggleProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  label: string;
  description?: string;
  disabled?: boolean;
}

export function Toggle({ checked, onChange, label, description, disabled }: ToggleProps) {
  return (
    <label className="flex cursor-pointer items-center justify-between gap-4 py-2.5">
      <span>
        <span className="block text-[13px] font-medium text-white/90">{label}</span>
        {description && <span className="mt-0.5 block text-[11px] leading-4 text-white/40">{description}</span>}
      </span>
      <input
        className="peer sr-only"
        type="checkbox"
        aria-label={label}
        checked={checked}
        disabled={disabled}
        onChange={(event) => onChange(event.target.checked)}
      />
      <span className="relative h-6 w-10 shrink-0 rounded-full border border-white/10 bg-white/10 transition peer-checked:bg-[var(--vox-accent-strong)] peer-disabled:opacity-40 after:absolute after:left-[3px] after:top-[3px] after:size-4 after:rounded-full after:bg-white after:shadow-sm after:transition-transform peer-checked:after:translate-x-4" />
    </label>
  );
}
