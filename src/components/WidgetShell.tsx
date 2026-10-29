import type { CSSProperties, ReactNode } from "react";
import type { ThemeCssVariables } from "../lib/themeRuntime";
import type { SettingsThemeId } from "../lib/types";

type Props = {
  management?: boolean;
  settingsTheme?: SettingsThemeId;
  opacity: number;
  fontFamily: string;
  fontScale: number;
  themeStyle?: ThemeCssVariables;
  children: ReactNode;
};

export function WidgetShell({ management, settingsTheme, opacity, fontFamily, fontScale, themeStyle, children }: Props) {
  return (
    <main
      className={management ? "widget-shell settings-window" : "widget-shell"}
      data-font={fontFamily}
      data-settings-theme={management ? settingsTheme : undefined}
      style={
        {
          "--widget-opacity": opacity,
          "--font-scale": fontScale,
          ...themeStyle,
        } as CSSProperties
      }
    >
      {children}
    </main>
  );
}
