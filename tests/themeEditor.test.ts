import assert from "node:assert/strict";
import test from "node:test";
import {
  isThemeDraftValid,
  nextAvailableThemeName,
  normalizeThemeDraft,
  replaceHexRgbPreservingAlpha,
  themeDraftEquals,
  validateThemeDraft,
  type ThemeEditorDraft,
} from "../src/lib/themeEditor.ts";
import { themeToCssVariables } from "../src/lib/themeRuntime.ts";

const draft: ThemeEditorDraft = {
  name: "Study",
  config: {
    version: 1,
    background: "#152238",
    foreground: "#F3F6FA",
    secondaryText: "#94A3B8",
    accent: "#4DA3FF",
    taskOverdue: "#FF6B6B",
    taskDueToday: "#5CB8FF",
    taskImportant: "#FFC857",
    taskPending: "#94A3B8",
    taskCompleted: "#64748B",
    backgroundOpacity: 0.78,
    blurPx: 18,
    borderRadiusPx: 16,
  },
};

test("normalizeThemeDraft trims names and uppercases valid hex input", () => {
  const normalized = normalizeThemeDraft({
    name: "  My Theme  ",
    config: { ...draft.config, accent: " #4da3ff " },
  });
  assert.equal(normalized.name, "My Theme");
  assert.equal(normalized.config.accent, "#4DA3FF");
});

test("color picker replacement preserves existing alpha", () => {
  assert.equal(replaceHexRgbPreservingAlpha("#15223880", "#4da3ff"), "#4DA3FF80");
  assert.equal(replaceHexRgbPreservingAlpha("#152238", "#4da3ff"), "#4DA3FF");
});

test("draft validation rejects invalid colors and numeric bounds", () => {
  const invalid: ThemeEditorDraft = {
    name: " ",
    config: {
      ...draft.config,
      accent: "#12GG88",
      backgroundOpacity: 1.1,
      blurPx: 65,
      borderRadiusPx: -1,
    },
  };
  const errors = validateThemeDraft(invalid);
  assert.equal(errors.name, "required");
  assert.equal(errors.accent, "invalidColor");
  assert.equal(errors.backgroundOpacity, "outOfRange");
  assert.equal(errors.blurPx, "outOfRange");
  assert.equal(errors.borderRadiusPx, "outOfRange");
  assert.equal(isThemeDraftValid(invalid), false);
});

test("theme name accepts 64 characters and rejects 65", () => {
  assert.equal(isThemeDraftValid({ ...draft, name: "a".repeat(64) }), true);
  assert.equal(validateThemeDraft({ ...draft, name: "a".repeat(65) }).name, "tooLong");
});

test("dirty comparison detects config changes without mutating the initial draft", () => {
  const changed = { ...draft, config: { ...draft.config, accent: "#A78BFA" } };
  assert.equal(themeDraftEquals(draft, { ...draft, config: { ...draft.config } }), true);
  assert.equal(themeDraftEquals(draft, changed), false);
  assert.equal(draft.config.accent, "#4DA3FF");
});

test("unique theme names receive stable numeric suffixes", () => {
  assert.equal(nextAvailableThemeName("New Theme", ["Ocean"]), "New Theme");
  assert.equal(
    nextAvailableThemeName("New Theme", ["new theme", "New Theme 2"]),
    "New Theme 3",
  );
});

test("preview uses the same runtime CSS variable mapping", () => {
  const variables = themeToCssVariables(draft.config, 0.5);
  assert.equal(variables["--accent"], draft.config.accent);
  assert.equal(variables["--task-overdue"], draft.config.taskOverdue);
  assert.equal(variables["--widget-background"], "rgba(21, 34, 56, 0.39)");
});
