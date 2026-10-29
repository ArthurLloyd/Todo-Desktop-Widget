// Keep empty, partial and non-numeric drafts out of the saved configuration.
export function parseOpacityPercent(value: string): number | null {
  if (!/^\d{1,3}$/.test(value.trim())) return null;
  const percent = Number(value);
  return percent >= 0 && percent <= 100 ? percent / 100 : null;
}
