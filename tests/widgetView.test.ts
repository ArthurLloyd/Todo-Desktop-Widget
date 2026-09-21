import assert from "node:assert/strict";
import test from "node:test";
import { reduceWidgetView } from "../src/lib/widgetView.ts";

test("widget view navigation keeps management pages mutually exclusive", () => {
  assert.equal(reduceWidgetView("widget", { type: "open-settings" }), "settings");
  assert.equal(reduceWidgetView("settings", { type: "open-theme-manager" }), "theme-manager");
  assert.equal(reduceWidgetView("theme-manager", { type: "back" }), "settings");
  assert.equal(reduceWidgetView("theme-manager", { type: "close" }), "widget");
  assert.equal(reduceWidgetView("settings", { type: "open-widget-manager" }), "widget-manager");
  assert.equal(reduceWidgetView("widget-manager", { type: "back" }), "settings");
  assert.equal(reduceWidgetView("settings", { type: "close" }), "widget");
});
