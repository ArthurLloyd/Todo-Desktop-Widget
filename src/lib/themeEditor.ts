import type { WidgetTheme, WidgetThemeConfig } from "./types";

export const THEME_COLOR_KEYS = [
  "background",
  "foreground",
  "secondaryText",
  "accent",
  "taskOverdue",
  "taskDueToday",
  "taskImportant",
  "taskPending",
  "taskCompleted",
] as const;

export type ThemeColorKey = (typeof THEME_COLOR_KEYS)[number];
export type ThemeDraftField =
  | "name"
  | ThemeColorKey
  | "backgroundOpacity"
  | "blurPx"
  | "borderRadiusPx";
export type ThemeDraftError = "required" | "tooLong" | "invalidColor" | "outOfRange";
export type ThemeDraftErrors = Partial<Record<ThemeDraftField, ThemeDraftError>>;

export type ThemeEditorDraft = {
  name: string;
  config: WidgetThemeConfig;
};

export function draftFromTheme(theme: WidgetTheme): ThemeEditorDraft {
  return {
    name: theme.name,
    config: { ...theme.config },
  };
}

export function cloneThemeDraft(draft: ThemeEditorDraft): ThemeEditorDraft {
  return {
    name: draft.name,
    config: { ...draft.config },
  };
}

export function normalizeHexColor(value: string): string {
  return value.trim().toUpperCase();
}

export function isValidHexColor(value: string): boolean {
  return /^#[0-9A-Fa-f]{6}([0-9A-Fa-f]{2})?$/.test(value.trim());
}

export function replaceHexRgbPreservingAlpha(current: string, nextRgb: string): string {
  const normalizedRgb = normalizeHexColor(nextRgb);
  if (!/^#[0-9A-F]{6}$/.test(normalizedRgb)) return current;
  const normalizedCurrent = normalizeHexColor(current);
  const alpha = /^#[0-9A-F]{8}$/.test(normalizedCurrent)
    ? normalizedCurrent.slice(7, 9)
    : "";
  return `${normalizedRgb}${alpha}`;
}

export function normalizeThemeDraft(draft: ThemeEditorDraft): ThemeEditorDraft {
  const config = { ...draft.config };
  for (const key of THEME_COLOR_KEYS) {
    config[key] = normalizeHexColor(config[key]);
  }
  return {
    name: draft.name.trim(),
    config,
  };
}

export function validateThemeDraft(draft: ThemeEditorDraft): ThemeDraftErrors {
  const errors: ThemeDraftErrors = {};
  const name = draft.name.trim();
  if (!name) errors.name = "required";
  else if ([...name].length > 64) errors.name = "tooLong";

  for (const key of THEME_COLOR_KEYS) {
    if (!isValidHexColor(draft.config[key])) errors[key] = "invalidColor";
  }

  if (!isInRange(draft.config.backgroundOpacity, 0, 1)) {
    errors.backgroundOpacity = "outOfRange";
  }
  if (!isInRange(draft.config.blurPx, 0, 64)) errors.blurPx = "outOfRange";
  if (!isInRange(draft.config.borderRadiusPx, 0, 64)) {
    errors.borderRadiusPx = "outOfRange";
  }
  return errors;
}

export function isThemeDraftValid(draft: ThemeEditorDraft): boolean {
  return Object.keys(validateThemeDraft(draft)).length === 0;
}

export function themeDraftEquals(left: ThemeEditorDraft, right: ThemeEditorDraft): boolean {
  return left.name === right.name && JSON.stringify(left.config) === JSON.stringify(right.config);
}

export function nextAvailableThemeName(base: string, existingNames: string[]): string {
  const normalizedBase = base.trim() || "New Theme";
  const used = new Set(existingNames.map((name) => name.trim().toLocaleLowerCase()));
  if (!used.has(normalizedBase.toLocaleLowerCase())) return normalizedBase;
  for (let suffix = 2; suffix < 10_000; suffix += 1) {
    const candidate = `${normalizedBase} ${suffix}`;
    if (!used.has(candidate.toLocaleLowerCase())) return candidate;
  }
  return `${normalizedBase} ${Date.now()}`;
}

function isInRange(value: number, min: number, max: number): boolean {
  return Number.isFinite(value) && value >= min && value <= max;
}
