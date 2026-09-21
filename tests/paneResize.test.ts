import assert from "node:assert/strict";
import test from "node:test";
import { calculatePaneResize, keyboardResizeDelta } from "../src/lib/paneResize.ts";

test("50/50 drag right preserves pair total", () => {
  const result = calculatePaneResize({ beforeWeight: 1, afterWeight: 1, beforePx: 300, afterPx: 300, deltaPx: 60, minPx: 200 });
  assert.equal(result.beforeWeight, 1.2);
  assert.equal(result.afterWeight, 0.8);
  assert.equal(result.beforeWeight + result.afterWeight, 2);
});

test("70/30 drag left changes only the adjacent pair", () => {
  const weights = [7, 3, 5];
  const result = calculatePaneResize({ beforeWeight: weights[0], afterWeight: weights[1], beforePx: 700, afterPx: 300, deltaPx: -100, minPx: 180 });
  assert.equal(result.beforeWeight, 6);
  assert.equal(result.afterWeight, 4);
  assert.equal(weights[2], 5);
});

test("before and after sizes clamp to the minimum", () => {
  const before = calculatePaneResize({ beforeWeight: 1, afterWeight: 1, beforePx: 300, afterPx: 300, deltaPx: -500, minPx: 200 });
  const after = calculatePaneResize({ beforeWeight: 1, afterWeight: 1, beforePx: 300, afterPx: 300, deltaPx: 500, minPx: 200 });
  assert.ok(Math.abs(before.beforeWeight - 2 / 3) < Number.EPSILON);
  assert.ok(Math.abs(after.afterWeight - 2 / 3) < Number.EPSILON);
});

test("keyboard steps use five percent and shift uses ten percent", () => {
  assert.equal(keyboardResizeDelta("columns", "ArrowRight", 800, false), 40);
  assert.equal(keyboardResizeDelta("columns", "ArrowLeft", 800, true), -80);
  assert.equal(keyboardResizeDelta("rows", "ArrowDown", 500, false), 25);
  assert.equal(keyboardResizeDelta("rows", "ArrowUp", 500, true), -50);
  assert.equal(keyboardResizeDelta("rows", "ArrowLeft", 500, false), null);
});

test("resize output remains inside the persisted weight range", () => {
  const minimumPair = calculatePaneResize({ beforeWeight: 0.05, afterWeight: 0.05, beforePx: 200, afterPx: 200, deltaPx: 100, minPx: 100 });
  assert.equal(minimumPair.beforeWeight, 0.05);
  assert.equal(minimumPair.afterWeight, 0.05);

  const maximumPair = calculatePaneResize({ beforeWeight: 100, afterWeight: 100, beforePx: 300, afterPx: 300, deltaPx: 200, minPx: 100 });
  assert.ok(maximumPair.beforeWeight <= 100);
  assert.ok(maximumPair.afterWeight >= 0.05);
});
