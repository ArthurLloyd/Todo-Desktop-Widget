import type { CSSProperties, ReactNode } from "react";
import type { ThemeCssVariables } from "../lib/themeRuntime";

type Props = {
  management?: boolean;
  opacity: number;
  fontFamily: string;
  fontScale: number;
  themeStyle?: ThemeCssVariables;
  children: ReactNode;
};

export function WidgetShell({ management, opacity, fontFamily, fontScale, themeStyle, children }: Props) {
  return (
    <main
      className={management ? "widget-shell settings-window" : "widget-shell"}
      data-font={fontFamily}
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
