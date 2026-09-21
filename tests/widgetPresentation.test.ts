import test from "node:test";
import assert from "node:assert/strict";
import {
  classifyPanePresentation,
  classifyStartupPresentation,
  classifySyncPresentation,
} from "../src/lib/widgetPresentation.ts";

test("startup distinguishes loading, failure, first run, and cached/offline readiness", () => {
  assert.equal(classifyStartupPresentation({ loading: true, loadError: null, loggedIn: false, taskCount: 0 }), "loading");
  assert.equal(classifyStartupPresentation({ loading: false, loadError: "db", loggedIn: false, taskCount: 0 }), "load-error");
  assert.equal(classifyStartupPresentation({ loading: false, loadError: null, loggedIn: false, taskCount: 0 }), "first-run");
  assert.equal(classifyStartupPresentation({ loading: false, loadError: null, loggedIn: false, taskCount: 2 }), "ready");
});

test("pane state keeps unbound, missing list, and valid empty states distinct", () => {
  assert.equal(classifyPanePresentation({ paneListId: null, listExists: false, taskCount: 0 }), "unbound");
  assert.equal(classifyPanePresentation({ paneListId: "missing", listExists: false, taskCount: 0 }), "missing-list");
  assert.equal(classifyPanePresentation({ paneListId: "list", listExists: true, taskCount: 0 }), "empty");
  assert.equal(classifyPanePresentation({ paneListId: "list", listExists: true, taskCount: 1 }), "tasks");
});

test("sync presentation does not confuse offline, auth, and errors", () => {
  assert.equal(classifySyncPresentation({ state: "offline" }), "offline");
  assert.equal(classifySyncPresentation({ state: "auth_required" }), "auth-required");
  assert.equal(classifySyncPresentation({ state: "error" }), "error");
});
