# Resizable Pane Layout Design

## Scope

Stage 4F adds user-resizable pane proportions for `columns` and `rows` widget layouts. `single` and `grid` retain their current behavior. The feature stores only relative pane weights; it does not change window geometry, task data, synchronization, themes, or desktop-window behavior.

## Data model and persistence

`widget_panes` gains `size_weight REAL NOT NULL DEFAULT 1.0`. Existing databases receive the column through an idempotent migration, so every existing pane starts equally sized. Rust and TypeScript pane models expose the value as `sizeWeight`.

Valid weights are finite values from `0.05` through `100.0`. A dedicated bulk repository operation validates the complete request before starting a transaction, verifies that every referenced pane belongs to the target widget, rejects duplicate or missing pane IDs, and then updates every weight atomically. A reset operation atomically returns every pane in one widget to `1.0`.

Reordering changes only `sort_order`; deletion removes only the selected pane; duplication copies the source weights; newly created panes start at `1.0`. Switching layout modes does not modify stored weights.

## Runtime layout

Columns and rows use a flex layout. Each pane item receives `flex-grow: sizeWeight` and a zero basis, with practical minimum sizes (200 px wide for columns and 160 px high for rows). Existing outer scrolling remains available when the window is too small to satisfy all minimums.

Adjacent panes are separated by a reusable `PaneDivider`. Pointer Events and pointer capture keep dragging reliable outside the divider hit area. During a drag, React updates only transient local weights, so pointer movement never writes to SQLite. On pointer release, cancellation, or capture loss, the complete ordered weight set is persisted once. A failed write restores the last persisted values and shows a localized error.

The resize calculation preserves the pair's total weight. It derives the new pair from the measured pixel sizes, pointer delta, and minimum pixel size. Only the panes immediately before and after the active divider change.

## Keyboard and accessibility

Each divider is focusable and uses `role="separator"` with the correct orientation. Arrow keys resize the adjacent pair in the layout axis by 5% of its measured span; Shift increases the step to 10%. Keyboard changes use the same pure resize math and are persisted through one 250 ms debounce rather than one write per key repeat.

## APIs and UI

The active window uses a dedicated current-widget command, so a window cannot resize panes owned by another widget instance. The Widget Manager gets a separate reset command and displays “Reset pane sizes” only when the selected widget contains at least two panes. Both commands emit the existing widget-configuration change event after successful persistence.

## Error handling and tests

Validation or ownership failures make no database changes. Frontend persistence failures roll back transient weights. Tests cover migration defaults, valid and invalid weights, foreign panes, transactional failure, reset, reorder/delete preservation, and pure pointer/keyboard resize calculations.

## Explicit non-goals

No changes are made to window ownership, tray behavior, pinning, the SyncCoordinator, task CRUD/delete, theme configuration, grid sizing, or the Theme Editor.
