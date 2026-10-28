import type { CSSProperties } from "react";
import type { WidgetTheme, WidgetThemeConfig } from "./types";

export const DEFAULT_THEME_ID = "default";

type ThemeCssVariableName =
  | "--control-background"
  | "--control-text"
  | "--widget-background"
  | "--widget-blur"
  | "--widget-border-radius"
  | "--text"
  | "--text-muted"
  | "--accent"
  | "--completed"
  | "--surface"
  | "--surface-strong"
  | "--border"
  | "--task-overdue"
  | "--task-due-today"
  | "--task-important"
  | "--task-pending"
  | "--task-completed"
  | "--task-completed-accent";

export type ThemeCssVariables = CSSProperties & Record<ThemeCssVariableName, string>;

export function hexWithOpacity(hex: string, opacity: number): string {
  const match = /^#([0-9a-fA-F]{6})([0-9a-fA-F]{2})?$/.exec(hex);
  if (!match) throw new Error("Theme colors must use #RRGGBB or #RRGGBBAA");

  const rgb = match[1];
  const red = Number.parseInt(rgb.slice(0, 2), 16);
  const green = Number.parseInt(rgb.slice(2, 4), 16);
  const blue = Number.parseInt(rgb.slice(4, 6), 16);
  const sourceAlpha = match[2] ? Number.parseInt(match[2], 16) / 255 : 1;
  const finalAlpha = roundAlpha(sourceAlpha * clampOpacity(opacity));
  return `rgba(${red}, ${green}, ${blue}, ${finalAlpha})`;
}

export function themeToCssVariables(
  config: WidgetThemeConfig,
  windowOpacity: number,
): ThemeCssVariables {
  const opacityMultiplier = clampOpacity(windowOpacity);
  const backgroundOpacity = clampOpacity(config.backgroundOpacity);
  const surfaceOpacity = backgroundOpacity * opacityMultiplier;
  const strongSurfaceOpacity = Math.min(1, backgroundOpacity + 0.18) * opacityMultiplier;

  return {
    "--control-background": hexWithOpacity(config.background.slice(0, 7), 0.5),
    "--control-text": readableControlText(config.background),
    "--widget-background": hexWithOpacity(config.background, surfaceOpacity),
    "--widget-blur": `${config.blurPx}px`,
    "--widget-border-radius": `${config.borderRadiusPx}px`,
    "--text": config.foreground,
    "--text-muted": config.secondaryText,
    "--accent": config.accent,
    "--completed": config.taskCompleted,
    "--surface": hexWithOpacity(config.background, surfaceOpacity),
    "--surface-strong": hexWithOpacity(config.background, strongSurfaceOpacity),
    "--border": hexWithOpacity(config.foreground, 0.16),
    "--task-overdue": config.taskOverdue,
    "--task-due-today": config.taskDueToday,
    "--task-important": config.taskImportant,
    "--task-pending": config.taskPending,
    "--task-completed": config.taskCompleted,
    "--task-completed-accent": config.taskCompleted,
  };
}

export function createThemeMap(themes: WidgetTheme[]): Map<string, WidgetTheme> {
  return new Map(themes.map((theme) => [theme.id, theme]));
}

export function resolveWidgetTheme(
  themes: Map<string, WidgetTheme>,
  themeId?: string | null,
): WidgetTheme | null {
  return (
    (themeId ? themes.get(themeId) : undefined) ??
    themes.get(DEFAULT_THEME_ID) ??
    [...themes.values()].find((theme) => theme.isBuiltin) ??
    null
  );
}

export function resolvePaneTheme(
  themes: Map<string, WidgetTheme>,
  paneThemeId: string | null | undefined,
  widgetTheme: WidgetTheme | null,
): WidgetTheme | null {
  return (paneThemeId ? themes.get(paneThemeId) : undefined) ?? widgetTheme;
}

export function resolvePaneThemeOverride(
  themes: Map<string, WidgetTheme>,
  paneThemeId?: string | null,
): WidgetTheme | null {
  return paneThemeId ? themes.get(paneThemeId) ?? null : null;
}

function clampOpacity(value: number): number {
  if (!Number.isFinite(value)) return 1;
  return Math.min(1, Math.max(0, value));
}

function roundAlpha(value: number): number {
  return Number(value.toFixed(4));
}

// Essential controls stay legible even when the widget background is transparent.
function readableControlText(background: string): string {
  const channels = [1, 3, 5].map((start) => Number.parseInt(background.slice(start, start + 2), 16) / 255)
    .map((value) => value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4);
  const luminance = channels[0] * 0.2126 + channels[1] * 0.7152 + channels[2] * 0.0722;
  return (luminance + 0.05) / 0.05 >= 1.05 / (luminance + 0.05) ? "#000000" : "#FFFFFF";
}
