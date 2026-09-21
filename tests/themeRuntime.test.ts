import assert from "node:assert/strict";
import test from "node:test";
import {
  createThemeMap,
  hexWithOpacity,
  resolvePaneTheme,
  resolveWidgetTheme,
  themeToCssVariables,
} from "../src/lib/themeRuntime.ts";
import type { WidgetTheme, WidgetThemeConfig } from "../src/lib/types.ts";

const config: WidgetThemeConfig = {
  version: 1,
  background: "#336699",
  foreground: "#F8FAFC",
  secondaryText: "#94A3B8",
  accent: "#38BDF8",
  taskOverdue: "#FB7185",
  taskDueToday: "#22D3EE",
  taskImportant: "#FBBF24",
  taskPending: "#A1A1AA",
  taskCompleted: "#64748B",
  backgroundOpacity: 0.8,
  blurPx: 20,
  borderRadiusPx: 16,
};

function theme(id: string, isBuiltin = true): WidgetTheme {
  return {
    id,
    name: id,
    config,
    isBuiltin,
    createdAt: "2026-01-01T00:00:00Z",
    updatedAt: "2026-01-01T00:00:00Z",
  };
}

test("hexWithOpacity handles six-digit colors", () => {
  assert.equal(hexWithOpacity("#336699", 0.5), "rgba(51, 102, 153, 0.5)");
});

test("hexWithOpacity multiplies embedded alpha", () => {
  assert.equal(hexWithOpacity("#33669980", 0.5), "rgba(51, 102, 153, 0.251)");
});

test("themeToCssVariables maps runtime and semantic values", () => {
  const variables = themeToCssVariables(config, 0.5);

  assert.equal(variables["--widget-background"], "rgba(51, 102, 153, 0.4)");
  assert.equal(variables["--widget-blur"], "20px");
  assert.equal(variables["--widget-border-radius"], "16px");
  assert.equal(variables["--task-overdue"], config.taskOverdue);
  assert.equal(variables["--task-due-today"], config.taskDueToday);
  assert.equal(variables["--task-important"], config.taskImportant);
  assert.equal(variables["--task-pending"], config.taskPending);
  assert.equal(variables["--task-completed"], config.taskCompleted);
  assert.deepEqual(
    Object.keys(variables).filter((key) => !key.startsWith("--")),
    [],
  );
});

test("themeToCssVariables preserves supported blur and radius extremes", () => {
  assert.equal(
    themeToCssVariables({ ...config, blurPx: 0, borderRadiusPx: 0 }, 1)["--widget-blur"],
    "0px",
  );
  const maximums = themeToCssVariables({ ...config, blurPx: 64, borderRadiusPx: 32 }, 1);
  assert.equal(maximums["--widget-blur"], "64px");
  assert.equal(maximums["--widget-border-radius"], "32px");
});

test("pane without override inherits widget theme", () => {
  const ocean = theme("ocean");
  const themes = createThemeMap([theme("default"), ocean]);
  assert.equal(resolvePaneTheme(themes, null, ocean)?.id, "ocean");
});

test("valid pane override wins over widget theme", () => {
  const ocean = theme("ocean");
  const violet = theme("violet");
  const themes = createThemeMap([theme("default"), ocean, violet]);
  assert.equal(resolvePaneTheme(themes, "violet", ocean)?.id, "violet");
});

test("invalid pane override falls back to widget theme", () => {
  const ocean = theme("ocean");
  const themes = createThemeMap([theme("default"), ocean]);
  assert.equal(resolvePaneTheme(themes, "missing", ocean)?.id, "ocean");
});

test("invalid widget theme falls back to Default", () => {
  const themes = createThemeMap([theme("default"), theme("ocean")]);
  assert.equal(resolveWidgetTheme(themes, "missing")?.id, "default");
});

test("invalid pane and widget themes converge on Default", () => {
  const themes = createThemeMap([theme("default"), theme("ocean")]);
  const widgetTheme = resolveWidgetTheme(themes, "missing");
  assert.equal(resolvePaneTheme(themes, "missing-pane", widgetTheme)?.id, "default");
});
