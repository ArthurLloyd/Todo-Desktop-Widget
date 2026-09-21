type Props = {
  label: string;
  value: number;
  min: number;
  max: number;
  step: number;
  displayValue: string;
  disabled?: boolean;
  error?: string;
  onChange: (value: number) => void;
};

export function SliderField({
  label,
  value,
  min,
  max,
  step,
  displayValue,
  disabled = false,
  error,
  onChange,
}: Props) {
  return (
    <label className="theme-slider-field">
      <span>{label}</span>
      <strong>{displayValue}</strong>
      <input
        type="range"
        value={value}
        min={min}
        max={max}
        step={step}
        disabled={disabled}
        aria-invalid={Boolean(error)}
        onChange={(event) => onChange(Number(event.target.value))}
      />
      {error ? <small className="theme-field-error">{error}</small> : null}
    </label>
  );
}
