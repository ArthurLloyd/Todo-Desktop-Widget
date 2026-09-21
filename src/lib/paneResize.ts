export type PaneResizeAxis = "columns" | "rows";

export type PaneResizeInput = {
  beforeWeight: number;
  afterWeight: number;
  beforePx: number;
  afterPx: number;
  deltaPx: number;
  minPx: number;
};

export type PaneResizeResult = {
  beforeWeight: number;
  afterWeight: number;
};

export function calculatePaneResize(input: PaneResizeInput): PaneResizeResult {
  const pairTotalWeight = input.beforeWeight + input.afterWeight;
  const pairTotalPx = input.beforePx + input.afterPx;
  if (!Number.isFinite(pairTotalWeight) || pairTotalWeight <= 0 || pairTotalPx <= 0) {
    return { beforeWeight: input.beforeWeight, afterWeight: input.afterWeight };
  }

  const effectiveMin = Math.min(Math.max(input.minPx, 0), pairTotalPx / 2);
  const minimumWeight = Math.max(0.05, pairTotalWeight - 100);
  const maximumWeight = Math.min(100, pairTotalWeight - 0.05);
  const minimumBeforePx = Math.max(effectiveMin, pairTotalPx * minimumWeight / pairTotalWeight);
  const maximumBeforePx = Math.min(pairTotalPx - effectiveMin, pairTotalPx * maximumWeight / pairTotalWeight);
  const newBeforePx = clamp(input.beforePx + input.deltaPx, minimumBeforePx, maximumBeforePx);
  const beforeWeight = pairTotalWeight * newBeforePx / pairTotalPx;
  return {
    beforeWeight,
    afterWeight: pairTotalWeight - beforeWeight,
  };
}

export function keyboardResizeDelta(
  axis: PaneResizeAxis,
  key: string,
  pairTotalPx: number,
  shiftKey: boolean,
): number | null {
  const step = pairTotalPx * (shiftKey ? 0.1 : 0.05);
  if (axis === "columns") {
    if (key === "ArrowLeft") return -step;
    if (key === "ArrowRight") return step;
  } else {
    if (key === "ArrowUp") return -step;
    if (key === "ArrowDown") return step;
  }
  return null;
}

function clamp(value: number, min: number, max: number) {
  return Math.min(Math.max(value, min), max);
}
