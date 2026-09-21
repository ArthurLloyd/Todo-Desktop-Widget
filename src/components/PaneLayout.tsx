import clsx from "clsx";
import type { CSSProperties, ReactNode } from "react";
import type { WidgetLayoutMode, WidgetPane } from "../lib/types";

type Props = {
  layoutMode: WidgetLayoutMode;
  panes: WidgetPane[];
  renderPane: (pane: WidgetPane) => ReactNode;
};

export function PaneLayout({ layoutMode, panes, renderPane }: Props) {
  const visiblePanes = layoutMode === "single" ? panes.slice(0, 1) : panes;

  return (
    <section
      className={clsx("pane-layout", `pane-layout--${layoutMode}`)}
      style={{ "--pane-count": Math.max(visiblePanes.length, 1) } as CSSProperties}
    >
      {visiblePanes.map((pane) => renderPane(pane))}
    </section>
  );
}
