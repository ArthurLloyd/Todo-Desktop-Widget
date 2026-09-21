# Full Widget / Window Manager Design

## Scope

Stage 4E turns the existing multi-window, multi-pane, per-window pin, geometry, list binding, and theme runtime into one manageable system. It does not add pane resizing, workspace profiles, theme import/export, theme cloud sync, multiple Microsoft accounts, custom CSS, or a replacement main dashboard.

All widgets remain views over the same task cache, outbox, Microsoft session, and `SyncCoordinator`. Widget management is local UI configuration and never triggers Graph synchronization.

## Boundaries

- `widget_config.rs` owns transactional WidgetInstance and WidgetPane configuration changes.
- `widget_window.rs` remains the only native window runtime for open, show, close, geometry persistence, and existing desktop-mode cleanup.
- `commands.rs` exposes narrow ID-scoped manager commands and emits existing configuration events after successful changes.
- `WidgetManager.tsx` owns the global widget list and list/detail navigation.
- `WidgetManagerDetail.tsx` owns one widget's display name, runtime state, pin preference, layout, theme, and pane configuration.
- `WidgetPaneEditor.tsx` owns one pane's title override, list, theme, inherited settings, and order controls.
- `SettingsPanel.tsx` keeps current-widget shortcuts and provides entry points to Widget Manager and the independent Theme Manager.

No component or command introduces a second window runtime, theme model, pane CRUD system, task database, or sync coordinator.

## Runtime and Data Flow

Manager actions target stable WidgetInstance IDs. Successful configuration changes emit `widget-config-changed(widgetInstanceId)`. Every open widget refreshes the global widget list; only the matching widget reloads its `WidgetContext`. Theme definition CRUD continues to emit `theme-definitions-changed`, which refreshes theme selectors and runtime theme maps.

The Manager refreshes its own selected detail after each mutation rather than depending only on events from the current window.

## Widget Creation

Both quick-create paths use one Rust transaction:

- Blank widget: unique friendly display name, one pane with `list_id=NULL`, `is_primary=false`, `is_open=true`, and `pinned=false`.
- From list: friendly name derived from the cached list display name, one pane bound to the selected list, and the same runtime defaults.

Both paths save a cascaded position relative to the invoking window when available and immediately open the new window through `widget_window::open_widget_window`. Multiple widgets may reference the same list.

## Widget Duplication

`duplicate_widget_instance` runs in one database transaction and creates a secondary instance with a new stable ID and new pane IDs. It copies layout mode, default theme ID, and every pane's list ID, title override, theme ID, sort order, show-completed override, and max-task limit.

It does not copy primary identity, runtime ownership state, pin state, window geometry, timestamps, tasks, lists, or theme definitions. The result is `is_primary=false`, `is_open=true`, and `pinned=false`. After commit, the command stores cascaded geometry and opens the new window. If opening fails, the command resets `is_open=false`, keeps the complete config recoverable as a closed secondary widget, and reports the error.

## Open, Show, Close, and Delete

- Open/show calls the existing `OpeningRegistry`-protected window runtime.
- Closing a secondary widget persists geometry, performs existing desktop detach/close cleanup, sets `is_open=false`, and keeps all configuration.
- Primary widgets cannot be permanently closed or deleted.
- Deleting a secondary widget requires confirmation. If open, the command persists geometry and closes/detaches the native window first, then transactionally deletes panes and the WidgetInstance.
- If native close succeeds but database deletion fails, the closed configuration remains available for recovery.
- Widget deletion never touches cached lists, tasks, pending operations, or Graph data.

## Pin Semantics

For an open widget, Manager pin changes reuse the existing per-window desktop-mode function and persistence path. For a closed widget, only the `pinned` preference changes; the existing restore/open flow applies it next time. Manager code never directly manipulates HWND, owner relationships, or AlwaysOnBottom.

## Pane Management

Pane editing supports:

- `title_override`, with blank input normalized to NULL and fallback to the list display name.
- list binding, preserving unavailable list IDs until the user chooses another list.
- pane theme assignment or inheritance from the widget theme.
- nullable show-completed and max-task overrides.
- add pane and delete any non-final pane.
- up/down ordering controls with disabled boundaries and accessible labels.

Pane deletion and reordering are transactional. Add, delete, and reorder leave sort orders normalized to `0..N-1`.

`reorder_widget_panes(widget_id, ordered_ids)` rejects duplicate IDs, missing IDs, extra IDs, and IDs owned by another widget. It then writes all sort orders in one transaction. Pane order never changes task ordering and panes cannot move across widgets in 4E.

## Manager UI

The list displays name, Primary/Secondary, current-window marker, Open/Closed, Pinned/Normal, layout mode, pane count, and resolved default theme name.

Primary actions are Show, Manage, and Duplicate. Open secondary actions are Show, Manage, Close, Duplicate, and Delete. Closed secondary actions are Open, Manage, Duplicate, and Delete.

The detail view supports name, runtime state, pin preference, layout, window theme, pane management, open/show, close, duplicate, delete, and an entry to the existing Theme Manager.

At narrow widths the Manager uses list-to-detail drill-down with a clear Back action and vertical scrolling. At wider widths it renders master/detail columns. It uses local `managerView` and `selectedWidgetId` state and adds no router or native manager window.

## Validation and Safety

- Widget names are trimmed, non-empty, and at most 64 Unicode characters. Names are display-only and need not be unique.
- Default generated names use a shared helper to avoid collisions where practical.
- IDs remain authoritative and never change on rename or configuration updates.
- Primary delete protection exists in both UI and Rust.
- Create, duplicate, delete, open, and close controls have loading guards.
- Delete Widget copy explicitly says Microsoft To Do lists and tasks are unaffected.
- Missing lists and themes remain visible as unavailable choices; runtime fallback behavior is preserved.
- All icon controls have descriptive accessible labels. Pane order remains keyboard-operable.

## Error Handling

Database transactions roll back on failure. Commands only emit configuration events after successful persistence. Runtime failures are returned with their original configuration preserved wherever possible. Manager-local errors remain visible in the Manager and do not masquerade as task, sync, or Microsoft account failures.

## Testing

Rust tests cover complex and primary duplication, copied pane configuration, reset pin/open/geometry semantics, secondary deletion, primary deletion rejection, widget-name validation, pane title override, valid reorder, duplicate/missing/foreign reorder rejection, delete normalization, create-from-list, and repeated list references.

Frontend pure-helper tests cover generated display names and Manager state helpers if introduced. Existing Theme Runtime and Theme Editor tests continue to run. Final verification includes frontend build, Rust formatting and compilation, the complete Rust suite, and manual checks for narrow/wide UI, multi-window operations, restart persistence, and separation of Close from Delete.
