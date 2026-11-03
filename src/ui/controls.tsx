import { type CSSProperties, useId } from "react";

/** A Windows 11 style switch. The whole row is the hit target, not just the knob. */
export function SwitchRow({
  label,
  hint,
  checked,
  disabled = false,
  onChange,
}: {
  label: string;
  hint?: string | undefined;
  checked: boolean;
  disabled?: boolean;
  onChange: (checked: boolean) => void;
}) {
  const hintId = useId();
  return (
    <button
      type="button"
      role="switch"
      className="setting"
      aria-checked={checked}
      aria-describedby={hint ? hintId : undefined}
      disabled={disabled}
      onClick={() => onChange(!checked)}
    >
      <span className="setting__text">
        <span className="setting__label">{label}</span>
        {hint && (
          <span id={hintId} className="setting__hint">
            {hint}
          </span>
        )}
      </span>
      <span className="switch" data-on={checked || undefined} aria-hidden="true">
        <span className="switch__knob" />
      </span>
    </button>
  );
}

/** A small set of mutually exclusive choices, shown side by side. */
export function Segmented<T extends string>({
  label,
  value,
  options,
  onChange,
}: {
  label: string;
  value: T;
  options: readonly { value: T; label: string }[];
  onChange: (value: T) => void;
}) {
  const name = useId();
  const index = Math.max(
    0,
    options.findIndex((option) => option.value === value),
  );
  // Native radios keep arrow-key navigation and announcements for free.
  return (
    <fieldset
      className="segmented"
      style={{ "--segments": options.length, "--active": index } as CSSProperties}
    >
      <legend className="sr-only">{label}</legend>
      <span className="segmented__thumb" aria-hidden="true" />
      {options.map((option) => (
        <label key={option.value} className="segmented__option">
          <input
            type="radio"
            name={name}
            value={option.value}
            checked={option.value === value}
            onChange={() => onChange(option.value)}
          />
          {option.label}
        </label>
      ))}
    </fieldset>
  );
}
