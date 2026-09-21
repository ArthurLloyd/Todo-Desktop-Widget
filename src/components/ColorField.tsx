import {
  isValidHexColor,
  normalizeHexColor,
  replaceHexRgbPreservingAlpha,
} from "../lib/themeEditor";

type Props = {
  label: string;
  value: string;
  error?: string;
  disabled?: boolean;
  onChange: (value: string) => void;
};

export function ColorField({ label, value, error, disabled = false, onChange }: Props) {
  const normalized = normalizeHexColor(value);
  const pickerValue = isValidHexColor(normalized) ? normalized.slice(0, 7) : "#000000";

  return (
    <label className="theme-color-field">
      <span>{label}</span>
      <span className="theme-color-field__controls">
        <input
          className="theme-color-field__picker"
          type="color"
          value={pickerValue}
          disabled={disabled}
          aria-label={label}
          onChange={(event) => onChange(replaceHexRgbPreservingAlpha(value, event.target.value))}
        />
        <input
          className="theme-color-field__hex"
          type="text"
          value={value}
          disabled={disabled}
          spellCheck={false}
          autoCapitalize="characters"
          aria-invalid={Boolean(error)}
          onChange={(event) => onChange(event.target.value)}
          onBlur={() => {
            if (isValidHexColor(value)) onChange(normalizeHexColor(value));
          }}
        />
      </span>
      {error ? <small className="theme-field-error">{error}</small> : null}
    </label>
  );
}
