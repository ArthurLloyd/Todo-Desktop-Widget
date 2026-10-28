import type { CSSProperties } from "react";
import type { WidgetThemeConfig } from "./types";

type Rgb = { red: number; green: number; blue: number };

type ManagementCssVariableName =
  | "--management-bg"
  | "--management-surface"
  | "--management-surface-elevated"
  | "--management-text"
  | "--management-muted"
  | "--management-border"
  | "--management-accent"
  | "--management-danger";

export type ManagementThemeCssVariables = CSSProperties &
  Record<ManagementCssVariableName, string>;

const FALLBACK_BACKGROUND = "#111827";
const FALLBACK_ACCENT = "#3B82F6";
const MANAGEMENT_BACKGROUND_ALPHA = 1;

export function deriveManagementTheme(
  config?: WidgetThemeConfig | null,
): ManagementThemeCssVariables {
  const background = parseHex(config?.background) ?? parseHex(FALLBACK_BACKGROUND)!;
  const dark = relativeLuminance(background) < 0.179;
  const preferredText = parseHex(dark ? "#F8FAFC" : "#111827")!;
  const text = contrastRatio(preferredText, background) >= 4.5 ? preferredText : dark ? WHITE : BLACK;
  const surface = mix(background, dark ? WHITE : BLACK, dark ? 0.08 : 0.045);
  const elevated = mix(background, dark ? WHITE : BLACK, dark ? 0.14 : 0.08);
  const requestedAccent = parseHex(config?.accent) ?? parseHex(FALLBACK_ACCENT)!;
  const accent = ensureContrast(requestedAccent, background, 3);

  return {
    "--management-bg": rgba(background, MANAGEMENT_BACKGROUND_ALPHA),
    "--management-surface": rgba(surface, 0.985),
    "--management-surface-elevated": rgba(elevated, 0.995),
    "--management-text": toHex(text),
    "--management-muted": toHex(mix(text, background, 0.34)),
    "--management-border": rgba(text, 0.2),
    "--management-accent": toHex(accent),
    "--management-danger": dark ? "#F87171" : "#B91C1C",
  };
}

function ensureContrast(color: Rgb, background: Rgb, minimum: number): Rgb {
  if (contrastRatio(color, background) >= minimum) return color;
  const target = relativeLuminance(background) < 0.5 ? WHITE : BLACK;
  for (let amount = 0.15; amount <= 1; amount += 0.05) {
    const adjusted = mix(color, target, amount);
    if (contrastRatio(adjusted, background) >= minimum) return adjusted;
  }
  return target;
}

function parseHex(value?: string | null): Rgb | null {
  const match = /^#([0-9a-fA-F]{6})(?:[0-9a-fA-F]{2})?$/.exec(value ?? "");
  if (!match) return null;
  return {
    red: Number.parseInt(match[1].slice(0, 2), 16),
    green: Number.parseInt(match[1].slice(2, 4), 16),
    blue: Number.parseInt(match[1].slice(4, 6), 16),
  };
}

function mix(from: Rgb, to: Rgb, amount: number): Rgb {
  const weight = Math.min(1, Math.max(0, amount));
  return {
    red: Math.round(from.red + (to.red - from.red) * weight),
    green: Math.round(from.green + (to.green - from.green) * weight),
    blue: Math.round(from.blue + (to.blue - from.blue) * weight),
  };
}

function relativeLuminance(color: Rgb): number {
  const channels = [color.red, color.green, color.blue].map((channel) => {
    const value = channel / 255;
    return value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * channels[0] + 0.7152 * channels[1] + 0.0722 * channels[2];
}

function contrastRatio(left: Rgb, right: Rgb): number {
  const lighter = Math.max(relativeLuminance(left), relativeLuminance(right));
  const darker = Math.min(relativeLuminance(left), relativeLuminance(right));
  return (lighter + 0.05) / (darker + 0.05);
}

function toHex(color: Rgb): string {
  return `#${[color.red, color.green, color.blue]
    .map((channel) => channel.toString(16).padStart(2, "0"))
    .join("")}`.toUpperCase();
}

function rgba(color: Rgb, alpha: number): string {
  return `rgba(${color.red}, ${color.green}, ${color.blue}, ${alpha})`;
}

const WHITE: Rgb = { red: 255, green: 255, blue: 255 };
const BLACK: Rgb = { red: 0, green: 0, blue: 0 };
