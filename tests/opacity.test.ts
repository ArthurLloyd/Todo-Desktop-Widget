import assert from "node:assert/strict";
import test from "node:test";
import { parseOpacityPercent } from "../src/lib/opacity.ts";

test("manual opacity accepts integer percentages, including transparent and opaque endpoints", () => {
  for (const [draft, expected] of [["0", 0], ["55", .55], ["100", 1], [" 25 ", .25]] as const) {
    assert.equal(parseOpacityPercent(draft), expected);
  }
});

test("incomplete, non-numeric, fractional and out of range drafts cannot change saved opacity", () => {
  for (const draft of ["", " ", "-1", "101", "1.5", "5e1", "50%", "abc", "Infinity"]) {
    assert.equal(parseOpacityPercent(draft), null, draft);
  }
});
