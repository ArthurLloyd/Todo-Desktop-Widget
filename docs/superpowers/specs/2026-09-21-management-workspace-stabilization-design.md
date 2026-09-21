# Management Workspace and Transparency Stabilization Design

## Scope

Stage 4E.1 stabilizes the already implemented Widget and Theme managers. It introduces no new native windows, workspace profiles, pane resizing, theme schema, task behavior, or synchronization behavior.

## Exclusive View Routing

Each Tauri widget webview owns one local `WidgetView` state: `widget`, `settings`, `widget-manager`, or `theme-manager`. Only the selected top-level view is rendered. Task content, title bar, panes, footer, context menu, and quick-add UI do not remain mounted behind management views.

Settings opens both managers through explicit view actions. Back from either manager returns to Settings. Close from every management view returns directly to the Widget view. Manager-internal list/detail and Theme Editor selection remain local sub-state.

## Management Shell

Settings, Widget Manager, and Theme Manager share one `ManagementShell`. It owns the fixed header, optional Back action, Close action, safe native-window drag area, and the single primary vertical scroll container. Buttons and other interactive controls never initiate window dragging.

The shell fills the current webview without hiding the native window. Its header remains outside the scroll container so navigation cannot be clipped by content, borders, decoration, or scrollbars.

## Visual Isolation

The management shell defines fixed high-contrast application tokens for background, surfaces, text, muted text, borders, accent, and danger. These tokens locally override the generic UI aliases used by management controls. They do not multiply by global widget opacity or editable theme background values.

Theme Preview is the only region that applies the draft theme's full color, opacity, blur, radius, and semantic variables. Preview decoration remains clipped inside the preview, below content, and non-interactive.

## Scrolling and Responsiveness

`management-shell__body` is the sole primary vertical scroll container and reserves stable scrollbar space. Settings, Theme Manager, and Widget Manager content participate in normal flow without full-screen absolute overlays or their own competing vertical scrollbars.

At narrow widths the Theme Manager stacks library, editor, and preview. Widget Manager keeps its list/detail drill-down. At wide widths Theme Manager uses library plus editor/preview content and Widget Manager uses master/detail columns.

## Opacity

Global widget opacity remains stored as `0.0..1.0`, retains the existing `0.78` default, and is exposed as `0..100%` in Settings. Finite values are clamped to that range; non-finite values are ignored in patches and sanitized to the default when loading persisted settings.

Zero opacity only makes widget background surfaces transparent. Text, icons, tasks, inputs, and controls remain rendered. Management views remain opaque and readable at every widget and theme opacity.

## Dirty Theme Protection

Theme Manager distinguishes Back from Close but routes both through its existing dirty-draft confirmation. Back returns to Settings after discard; Close returns to Widget after discard. Escape behaves like Back. Cancel leaves the editor and draft intact.

## Testing

Pure view-reducer tests cover all top-level transitions. Rust settings tests cover valid endpoints, clamping, and non-finite sanitization. Existing Rust, Theme Runtime, Theme Editor, sync, delete, window, and Widget Manager tests remain unchanged and must continue to pass.
