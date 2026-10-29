import { deriveManagementTheme, type ManagementThemeCssVariables } from "./managementTheme.ts";
import type { SettingsThemeId, WidgetTheme, WidgetThemeConfig } from "./types";

export const SETTINGS_THEME_IDS: readonly SettingsThemeId[] = [
  "follow-window", "default", "ocean", "violet", "forest", "gem", "feng-bujue", "chatgpt", "deepseek",
];

const companionPalettes: Partial<Record<SettingsThemeId, ManagementThemeCssVariables>> = {
  chatgpt: {
    "--management-bg": "#FAF8FC",
    "--management-surface": "#FFFFFF",
    "--management-surface-elevated": "#F2EDF7",
    "--management-text": "#30263D",
    "--management-muted": "#675874",
    "--management-border": "#9984A9",
    "--management-accent": "#785495",
    "--management-danger": "#B42335",
  },
  deepseek: {
    "--management-bg": "#0B1735",
    "--management-surface": "#102247",
    "--management-surface-elevated": "#1B3565",
    "--management-text": "#F0F6FF",
    "--management-muted": "#B6CAED",
    "--management-border": "#6A89B9",
    "--management-accent": "#A6C4FF",
    "--management-danger": "#FFA2AF",
  },
};

export function resolveSettingsThemeId(value?: string): SettingsThemeId {
  return SETTINGS_THEME_IDS.includes(value as SettingsThemeId) ? value as SettingsThemeId : "follow-window";
}

export function deriveSettingsTheme(
  id: SettingsThemeId | undefined,
  themes: WidgetTheme[],
  windowConfig?: WidgetThemeConfig,
): ManagementThemeCssVariables {
  const themeId = resolveSettingsThemeId(id);
  const companionPalette = companionPalettes[themeId];
  if (companionPalette) return { ...companionPalette };
  if (themeId === "gem" || themeId === "feng-bujue") {
    const gem = themeId === "gem";
    // Controls are solid surfaces. Only the decorative image sits behind them.
    return {
      "--management-bg": gem ? "#140F20" : "#101113",
      "--management-surface": gem ? "#20182E" : "#1B1C20",
      "--management-surface-elevated": gem ? "#332541" : "#2B2C31",
      "--management-text": gem ? "#F7F0FF" : "#F1EDE7",
      "--management-muted": gem ? "#C9B8DC" : "#C2BEB8",
      "--management-border": gem ? "#615071" : "#5D5D62",
      "--management-accent": gem ? "#C6A1FF" : "#CEC4E0",
      "--management-danger": "#FF929D",
    };
  }
  return deriveManagementTheme(themeId === "follow-window"
    ? windowConfig
    : themes.find((theme) => theme.id === themeId)?.config);
}
