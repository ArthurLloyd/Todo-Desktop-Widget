import assert from "node:assert/strict";
import test from "node:test";
import { deriveSettingsTheme, resolveSettingsThemeId } from "../src/lib/settingsThemes.ts";
import type { WidgetThemeConfig } from "../src/lib/types.ts";

test("settings theme migrates missing and unknown IDs to the existing window palette", () => {
  assert.equal(resolveSettingsThemeId(undefined), "follow-window");
  assert.equal(resolveSettingsThemeId("removed-theme"), "follow-window");
});

test("character themes are independent of the desktop window and its opacity", () => {
  const before = { background: "#FFFFFF", accent: "#2563EB", backgroundOpacity: 0 } as WidgetThemeConfig;
  const after = { background: "#000000", accent: "#34D399", backgroundOpacity: 1 } as WidgetThemeConfig;
  for (const id of ["gem", "feng-bujue", "chatgpt", "deepseek"] as const) {
    assert.deepEqual(deriveSettingsTheme(id, [], before), deriveSettingsTheme(id, [], after));
    assert.notEqual(deriveSettingsTheme(id, [], before)["--management-accent"], "#FF929D");
  }
  assert.notEqual(deriveSettingsTheme("gem", [])["--management-bg"], deriveSettingsTheme("feng-bujue", [])["--management-bg"]);
});

test("simple settings colors remain independent; following the window remains explicit", () => {
  const config = { background: "#153028", accent: "#34D399" } as WidgetThemeConfig;
  assert.notEqual(deriveSettingsTheme("follow-window", [], config)["--management-bg"], deriveSettingsTheme("default", [], config)["--management-bg"]);
});

test("white and blue companion themes keep body text readable on all control surfaces", () => {
  const luminance = (hex: string) => {
    const rgb = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16) / 255)
      .map((v) => v <= .04045 ? v / 12.92 : ((v + .055) / 1.055) ** 2.4);
    return rgb[0] * .2126 + rgb[1] * .7152 + rgb[2] * .0722;
  };
  for (const id of ["chatgpt", "deepseek"] as const) {
    assert.equal(resolveSettingsThemeId(id), id);
    const palette = deriveSettingsTheme(id, []);
    for (const surface of ["--management-bg", "--management-surface", "--management-surface-elevated"] as const) {
      for (const text of ["--management-text", "--management-muted"] as const) {
        const a = luminance(palette[surface]);
        const b = luminance(palette[text]);
        assert.ok((Math.max(a, b) + .05) / (Math.min(a, b) + .05) >= 4.5, `${id}: ${text} on ${surface}`);
      }
    }
  }
});
