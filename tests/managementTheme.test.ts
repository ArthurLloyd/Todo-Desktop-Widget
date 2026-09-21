import assert from "node:assert/strict";
import test from "node:test";
import { deriveManagementTheme } from "../src/lib/managementTheme.ts";
import type { WidgetThemeConfig } from "../src/lib/types.ts";

function theme(overrides: Partial<WidgetThemeConfig> = {}): WidgetThemeConfig {
  return {
    version: 1,
    background: "#111827",
    foreground: "#F8FAFC",
    secondaryText: "#CBD5E1",
    accent: "#38BDF8",
    taskOverdue: "#F87171",
    taskDueToday: "#38BDF8",
    taskImportant: "#FBBF24",
    taskPending: "#CBD5E1",
    taskCompleted: "#94A3B8",
    backgroundOpacity: 1,
    blurPx: 20,
    borderRadiusPx: 18,
    ...overrides,
  };
}

test("dark and light backgrounds receive readable management text", () => {
  assert.equal(deriveManagementTheme(theme({ background: "#07111F" }))["--management-text"], "#F8FAFC");
  assert.equal(deriveManagementTheme(theme({ background: "#F8FAFC" }))["--management-text"], "#111827");
});

test("management background stays opaque regardless of theme opacity", () => {
  const opaque = deriveManagementTheme(theme({ backgroundOpacity: 1 }));
  const transparent = deriveManagementTheme(theme({ backgroundOpacity: 0 }));
  assert.equal(opaque["--management-bg"], transparent["--management-bg"]);
  assert.match(String(transparent["--management-bg"]), /, 0\.97\)$/);
});

test("readable accents are preserved and low contrast accents are adjusted", () => {
  const preserved = deriveManagementTheme(theme({ background: "#07111F", accent: "#38BDF8" }));
  assert.equal(preserved["--management-accent"], "#38BDF8");

  const adjusted = deriveManagementTheme(theme({ background: "#FFFFFF", accent: "#FFFFFF" }));
  assert.notEqual(adjusted["--management-accent"], "#FFFFFF");
});

test("danger remains independent from theme accent", () => {
  const green = deriveManagementTheme(theme({ accent: "#22C55E" }));
  const violet = deriveManagementTheme(theme({ accent: "#8B5CF6" }));
  assert.equal(green["--management-danger"], violet["--management-danger"]);
  assert.notEqual(green["--management-danger"], green["--management-accent"]);
});
