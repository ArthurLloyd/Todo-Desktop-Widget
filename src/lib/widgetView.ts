export type WidgetView = "widget" | "settings" | "widget-manager" | "theme-manager";

export type WidgetViewAction =
  | { type: "open-settings" }
  | { type: "open-widget-manager" }
  | { type: "open-theme-manager" }
  | { type: "back" }
  | { type: "close" };

export function reduceWidgetView(view: WidgetView, action: WidgetViewAction): WidgetView {
  if (action.type === "open-settings") return "settings";
  if (action.type === "open-widget-manager") return "widget-manager";
  if (action.type === "open-theme-manager") return "theme-manager";
  if (action.type === "close") return "widget";
  if (view === "widget-manager" || view === "theme-manager") return "settings";
  return "widget";
}
