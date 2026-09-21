# Management Theme, Tray Activation, and Preview Containment Design

## Scope

This validation fix keeps the existing exclusive React views and native window model. It adds no
window, database field, theme-schema field, or synchronization behavior.

## Management theme

A pure `managementTheme` helper derives opaque management CSS tokens from the currently resolved
window-default theme. It ignores global widget opacity, theme background opacity, pane overrides,
and unsaved editor drafts. Background luminance selects readable light or dark text. Accent is
preserved when it has usable contrast and adjusted toward a safe contrast color otherwise. Danger
remains an independent red semantic color. `App` computes the tokens once and passes them to the
shared `ManagementShell`, so Settings, Widget Manager, and Theme Manager use one policy.

## Preview containment

The blue and brown shapes are the `ThemePreview` stage pseudo-elements. The stage becomes a paint
containment boundary with `position: relative`, `isolation: isolate`, and `overflow: hidden`.
Decorations use local `z-index: 0` and never receive pointer input; the preview window uses
`position: relative; z-index: 1`. This removes the negative stacking level that could paint below
the preview stacking context and appear behind neighboring editor content.

## Tray activation

Tray left-click targets only the primary window. A single high-level desktop-window activation
helper reads the existing authoritative pinned state. Normal windows are shown, unminimized,
restored through the existing normal-mode/owner/Z-order path, and focused. Pinned windows are shown,
unminimized, and re-enforced as desktop-bottom without focus or foreground promotion. The tray
Settings command uses the same activation path and then emits an event that changes only the
primary React view to Settings. Existing multi-widget Show/Hide menu commands remain unchanged.

## Verification

Pure tests cover management contrast/opacity rules and pinned-versus-normal tray activation
selection. Existing theme, view reducer, Rust, and build checks remain mandatory; Windows focus,
desktop-bottom behavior, and geometric containment receive explicit manual validation.
