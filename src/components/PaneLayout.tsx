import clsx from "clsx";
import {
  useEffect,
  useRef,
  useState,
  type CSSProperties,
  type KeyboardEvent,
  type PointerEvent,
  type ReactNode,
} from "react";
import { useI18n } from "../lib/i18n";
import { calculatePaneResize, keyboardResizeDelta } from "../lib/paneResize";
import type { WidgetLayoutMode, WidgetPane, WidgetPaneWeightInput } from "../lib/types";
import { PaneDivider } from "./PaneDivider";

const MIN_COLUMN_PX = 200;
const MIN_ROW_PX = 160;
const KEYBOARD_PERSIST_DELAY_MS = 250;

type Props = {
  layoutMode: WidgetLayoutMode;
  panes: WidgetPane[];
  renderPane: (pane: WidgetPane) => ReactNode;
  onPersistWeights: (weights: WidgetPaneWeightInput[]) => Promise<void>;
};

type DragState = {
  pointerId: number;
  startCoordinate: number;
  beforeId: string;
  afterId: string;
  beforePx: number;
  afterPx: number;
  beforeWeight: number;
  afterWeight: number;
};

export function PaneLayout({ layoutMode, panes, renderPane, onPersistWeights }: Props) {
  const { t } = useI18n();
  const visiblePanes = layoutMode === "single" ? panes.slice(0, 1) : panes;
  const resizable = layoutMode === "columns" || layoutMode === "rows";
  const [weights, setWeights] = useState(() => weightsFromPanes(panes));
  const [persistError, setPersistError] = useState<string | null>(null);
  const weightsRef = useRef(weights);
  const dragRef = useRef<DragState | null>(null);
  const dirtyRef = useRef(false);
  const persistingRef = useRef(false);
  const panesRef = useRef(panes);
  const persistWeightsRef = useRef(onPersistWeights);
  const paneElements = useRef(new Map<string, HTMLDivElement>());
  const keyboardTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    panesRef.current = panes;
    persistWeightsRef.current = onPersistWeights;
    if (!dragRef.current && !keyboardTimer.current && !persistingRef.current) {
      applyWeights(weightsFromPanes(panes));
    }
  }, [onPersistWeights, panes]);

  useEffect(() => () => {
    if (keyboardTimer.current) clearTimeout(keyboardTimer.current);
    if (dirtyRef.current) {
      const payload = panesRef.current.map((pane) => ({
        paneId: pane.id,
        sizeWeight: weightsRef.current[pane.id] ?? pane.sizeWeight,
      }));
      dirtyRef.current = false;
      void persistWeightsRef.current(payload).catch(() => undefined);
    }
  }, []);

  function applyWeights(next: Record<string, number>) {
    weightsRef.current = next;
    setWeights(next);
  }

  async function persistCurrentWeights() {
    if (!dirtyRef.current) return;
    const payload = panes.map((pane) => ({
      paneId: pane.id,
      sizeWeight: weightsRef.current[pane.id] ?? pane.sizeWeight,
    }));
    dirtyRef.current = false;
    persistingRef.current = true;
    try {
      await onPersistWeights(payload);
      setPersistError(null);
    } catch {
      applyWeights(weightsFromPanes(panes));
      setPersistError(t("pane.resizeFailed"));
    } finally {
      persistingRef.current = false;
    }
  }

  function dividerMetrics(before: WidgetPane, after: WidgetPane) {
    const beforeElement = paneElements.current.get(before.id);
    const afterElement = paneElements.current.get(after.id);
    if (!beforeElement || !afterElement) return null;
    const beforeRect = beforeElement.getBoundingClientRect();
    const afterRect = afterElement.getBoundingClientRect();
    return {
      beforePx: layoutMode === "columns" ? beforeRect.width : beforeRect.height,
      afterPx: layoutMode === "columns" ? afterRect.width : afterRect.height,
    };
  }

  function resizePair(
    before: WidgetPane,
    after: WidgetPane,
    beforePx: number,
    afterPx: number,
    beforeWeight: number,
    afterWeight: number,
    deltaPx: number,
  ) {
    const result = calculatePaneResize({
      beforeWeight,
      afterWeight,
      beforePx,
      afterPx,
      deltaPx,
      minPx: layoutMode === "columns" ? MIN_COLUMN_PX : MIN_ROW_PX,
    });
    applyWeights({
      ...weightsRef.current,
      [before.id]: result.beforeWeight,
      [after.id]: result.afterWeight,
    });
    dirtyRef.current = true;
  }

  function startPointerResize(event: PointerEvent<HTMLDivElement>, before: WidgetPane, after: WidgetPane) {
    if (keyboardTimer.current) {
      clearTimeout(keyboardTimer.current);
      keyboardTimer.current = null;
    }
    const metrics = dividerMetrics(before, after);
    if (!metrics) return;
    dragRef.current = {
      pointerId: event.pointerId,
      startCoordinate: layoutMode === "columns" ? event.clientX : event.clientY,
      beforeId: before.id,
      afterId: after.id,
      beforePx: metrics.beforePx,
      afterPx: metrics.afterPx,
      beforeWeight: weightsRef.current[before.id] ?? before.sizeWeight,
      afterWeight: weightsRef.current[after.id] ?? after.sizeWeight,
    };
    setPersistError(null);
    event.preventDefault();
  }

  function movePointerResize(event: PointerEvent<HTMLDivElement>) {
    const drag = dragRef.current;
    if (!drag || drag.pointerId !== event.pointerId) return;
    const before = panes.find((pane) => pane.id === drag.beforeId);
    const after = panes.find((pane) => pane.id === drag.afterId);
    if (!before || !after) return;
    const coordinate = layoutMode === "columns" ? event.clientX : event.clientY;
    resizePair(before, after, drag.beforePx, drag.afterPx, drag.beforeWeight, drag.afterWeight, coordinate - drag.startCoordinate);
  }

  function endPointerResize(event: PointerEvent<HTMLDivElement>) {
    const drag = dragRef.current;
    if (!drag || drag.pointerId !== event.pointerId) return;
    dragRef.current = null;
    void persistCurrentWeights();
  }

  function keyboardResize(event: KeyboardEvent<HTMLDivElement>, before: WidgetPane, after: WidgetPane) {
    const metrics = dividerMetrics(before, after);
    if (!metrics) return;
    const delta = keyboardResizeDelta(
      layoutMode === "columns" ? "columns" : "rows",
      event.key,
      metrics.beforePx + metrics.afterPx,
      event.shiftKey,
    );
    if (delta === null) return;
    event.preventDefault();
    resizePair(
      before,
      after,
      metrics.beforePx,
      metrics.afterPx,
      weightsRef.current[before.id] ?? before.sizeWeight,
      weightsRef.current[after.id] ?? after.sizeWeight,
      delta,
    );
    if (keyboardTimer.current) clearTimeout(keyboardTimer.current);
    keyboardTimer.current = setTimeout(() => {
      keyboardTimer.current = null;
      void persistCurrentWeights();
    }, KEYBOARD_PERSIST_DELAY_MS);
  }

  if (!resizable) {
    return (
      <section
        className={clsx("pane-layout", `pane-layout--${layoutMode}`)}
        style={{ "--pane-count": Math.max(visiblePanes.length, 1) } as CSSProperties}
      >
        {visiblePanes.map((pane) => renderPane(pane))}
      </section>
    );
  }

  return (
    <section className={clsx("pane-layout", `pane-layout--${layoutMode}`)}>
      {visiblePanes.map((pane, index) => {
        const nextPane = visiblePanes[index + 1];
        const pairTotal = nextPane
          ? (weights[pane.id] ?? pane.sizeWeight) + (weights[nextPane.id] ?? nextPane.sizeWeight)
          : 1;
        return (
          <div className="pane-layout__group" key={pane.id}>
            <div
              className="pane-layout__item"
              data-pane-id={pane.id}
              ref={(element) => {
                if (element) paneElements.current.set(pane.id, element);
                else paneElements.current.delete(pane.id);
              }}
              style={{ flexGrow: weights[pane.id] ?? pane.sizeWeight }}
            >
              {renderPane(pane)}
            </div>
            {nextPane ? (
              <PaneDivider
                orientation={layoutMode === "columns" ? "vertical" : "horizontal"}
                label={t("pane.resizeDivider")}
                valueNow={100 * (weights[pane.id] ?? pane.sizeWeight) / pairTotal}
                onPointerStart={(event) => startPointerResize(event, pane, nextPane)}
                onPointerMove={movePointerResize}
                onPointerEnd={endPointerResize}
                onKeyResize={(event) => keyboardResize(event, pane, nextPane)}
              />
            ) : null}
          </div>
        );
      })}
      {persistError ? <div className="pane-layout__error" role="alert">{persistError}</div> : null}
    </section>
  );
}

function weightsFromPanes(panes: WidgetPane[]) {
  return Object.fromEntries(panes.map((pane) => [pane.id, pane.sizeWeight]));
}
