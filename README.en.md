# Todo Desktop Widget

[简体中文](README.md) | **English**

A lightweight widget that brings your Microsoft To Do lists to the Windows desktop. It supports multiple independent windows, multiple list panes within each window, offline task operations, and a shared settings window.

[Download for Windows](https://github.com/ArthurLloyd/Todo-Desktop-Widget/releases/latest) · [Release history](https://github.com/ArthurLloyd/Todo-Desktop-Widget/releases) · [Report an issue](https://github.com/ArthurLloyd/Todo-Desktop-Widget/issues)

Adapted from [Chenzo1001/Microsoft-Todo-Desktop-Widget](https://github.com/Chenzo1001/Microsoft-Todo-Desktop-Widget) and independently maintained by **涂风 (Tu Feng)**. This is an unofficial app and is not affiliated with Microsoft.

## Features

- **Microsoft To Do sync**: Sign in with your Microsoft account, view and create lists, and manage task dates, reminders, importance, and notes.
- **Multiple desktop windows**: Create, duplicate, and manage independent widgets, each with multiple list panes.
- **Flexible layouts**: Use a single pane, columns, rows, or a grid. Drag dividers to resize adjacent panes.
- **Offline task operations**: Add, edit, complete, and delete tasks locally, then sync when your connection returns.
- **Independent appearance settings**: Each widget saves its own theme and background opacity. Settings and management pages have a separate choice of four simple color schemes and four illustrated scenes.
- **Windows integration**: Pin widgets to the desktop, use the system tray, launch at startup, and add tasks with `Ctrl + Alt + T`.

## Screenshots

These screenshots show the **settings interface in this project's customized v0.2.1 release**, using sample data. The four personalization scenes apply independently to settings and management pages; desktop lists retain their own themes and layouts.

| ChatGPT · White dragon girl | DeepSeek · Blue whale girl |
| --- | --- |
| ![ChatGPT settings theme with sample data](assets/Theme-ChatGPT.jpg) | ![DeepSeek settings theme with sample data](assets/Theme-DeepSeek.jpg) |

| G.E.M. · Violet stage | Feng Bujue · Noir theater |
| --- | --- |
| ![G.E.M. settings theme with sample data](assets/Theme-GEM.jpg) | ![Feng Bujue settings theme with sample data](assets/Theme-Feng.jpg) |

## Download and installation

The current version is **v0.2.1**, with installers for **Windows x64**. Choose a file from [Releases](https://github.com/ArthurLloyd/Todo-Desktop-Widget/releases/latest):

| File | Purpose |
| --- | --- |
| `Todo.Desktop.Widget_0.2.1_x64-setup.exe` | Recommended per-user installer (NSIS) |
| `Todo.Desktop.Widget_0.2.1_x64_en-US.msi` | For environments with an existing MSI installation |
| `SHA256SUMS.txt` | SHA-256 checksums for the installers |

Run the installer, launch the app, and follow the sign-in steps below. The installers are unsigned, so Windows may display an unknown publisher warning. When upgrading, keep the same installer type and installation directory; see [Windows packaging and upgrades](#windows-packaging-and-upgrades). The repository includes macOS source code and a WidgetKit extension, but no macOS installer has completed this project's validation.

## What's new in 0.2.1

- **Four complete personalization scenes**: Added **G.E.M. · Violet stage**, **Feng Bujue · Noir theater**, **ChatGPT · White dragon girl** (white hair, dragon horns, and a dragon tail), and **DeepSeek · Blue whale girl** (blue hair, a whale tail, and a maid outfit). Each scene styles the settings and management pages independently of desktop list appearance and layout.
- **Dedicated Personalization tab**: Settings now has three tabs: **Window appearance**, **Personalization**, and **General & account**. The Default / Ocean / Violet / Forest color schemes and the option to follow the selected window remain available. Theme selections save automatically.
- **Rounded corners and headings**: Fixed square remnants at widget corners and removed duplicate headings while keeping a clear page and section hierarchy.
- **Direct opacity input**: The slider remains available. Double-click the percentage, or focus it and press Enter / Space, to enter an integer from 0 to 100. Press Enter or move focus away to save; press Esc to cancel.

All four backgrounds are complete horizontal scenes with environmental details on both sides. They fill the window, with local backdrops behind text and controls to keep them readable. Decorative layers do not intercept mouse input. The ChatGPT and DeepSeek scenes expand community character designs; attribution and CC BY-NC-SA 4.0 details are recorded in the [theme asset notices](src/assets/themes/NOTICE.md).

Settings previews with sample data: [ChatGPT · White dragon girl](assets/Theme-ChatGPT.jpg) · [DeepSeek · Blue whale girl](assets/Theme-DeepSeek.jpg) · [G.E.M. · Violet stage](assets/Theme-GEM.jpg) · [Feng Bujue · Noir theater](assets/Theme-Feng.jpg). See the [0.2.1 validation record](docs/VALIDATION-0.2.1.md) (Chinese) for the scope of verification.

## What's new in 0.2.0

- **Separate settings window**: Open it from any widget's settings button. Version 0.2.0 grouped settings under **Window appearance** and **General & account**. Select the window to edit under Window appearance while keeping your widgets visible.
- **Per-window opacity**: Changing one widget's background opacity does not change the others. Values persist after closing and reopening a window or restarting the app. Duplicating a widget copies its opacity, after which the two can be adjusted independently.
- **Create Microsoft To Do lists directly**: Click **New list** beside the widget's list selector, enter a name, and click **Create and use**. The new list is saved to your Microsoft account and assigned to the current pane.
- **Startup edge fix**: Disabled system shadows for transparent, borderless widgets and refreshed the native frame after restoring the window mode to address a residual edge on the right at startup. Dragging, resizing, and desktop pinning remain available.
- **Typography and interaction updates**: Settings uses 14px body text, 13px help text, 17px section headings, and 20px page titles. Control heights, alignment, and spacing were standardized, with save feedback, keyboard focus indicators, error messages, and reduced-motion support.
- **List creation entry point**: The new-list button sits to the right of the list selector. Its background has 50% opacity while its text and icon remain fully opaque.

## Getting started

1. Launch the app, open **Settings → General & account**, and sign in to Microsoft.
2. Return to a widget and select an existing list from the dropdown, or click **New list**.
3. Enter a task title and press Enter. Use the checkbox to complete a task, or open its details to edit dates, reminders, importance, and notes.
4. To add more widgets, open **Settings → Window appearance → Manage desktop widgets**. You can create, duplicate, show, or close additional windows and add list panes.

**Window names and list names are separate.** Renaming a window changes only its local title. **New list** creates a list in Microsoft To Do. The same list can appear in multiple windows or panes.

### Appearance and window behavior

| Setting | Scope | Behavior |
| --- | --- | --- |
| Background opacity | Selected widget | Adjust the slider, or double-click the percentage to enter an integer from 0 to 100. You can also focus the percentage and press Enter / Space. Enter or moving focus away saves; Esc cancels. 0% is more transparent; 100% is more opaque. Text is unaffected, and the theme's own opacity also applies. |
| Pin to desktop | Selected widget | Keeps the widget visible after Win + D while allowing normal apps to cover it. This differs from keeping it always on top of other apps. |
| Layout and window theme | Selected widget | Supports a single pane, columns, rows, and grids. More pane options are available in the widget manager. |
| Pane theme and list selection | Individual pane | A pane can override the window theme. Drag dividers to resize adjacent panes. |
| Personalization theme | Settings and management pages | Saved separately. Choose a simple color scheme, an illustrated theme, or the option to follow the selected window. Desktop list appearance and layout stay unchanged. |
| Language, font, and text size | All windows | Supports Simplified Chinese, English, and following the system language. |
| Sync interval, completed-task visibility, and launch at startup | All windows | Account and sync services are shared. Each pane can override completed-task visibility. |

Window appearance changes save automatically. The global opacity from older versions remains a fallback for windows without an individual value, so upgrading does not reset existing appearance settings. The settings window itself has a solid background so it remains usable when widget opacity is low.

**Personalization** offers the Default / Ocean / Violet / Forest color schemes and four complete scenes: **G.E.M. · Violet stage**, **Feng Bujue · Noir theater**, **ChatGPT · White dragon girl**, and **DeepSeek · Blue whale girl**. These themes apply to settings and management pages. You can also explicitly choose to follow the selected window; older configurations retain that behavior by default.

### Sync and offline use

Task additions, edits, completions, and deletions are written to local SQLite first, then uploaded through a shared outbox and sync coordinator. Pending operations are retained while offline and resume syncing when the connection returns.

**Creating a list requires sign-in and an internet connection.** Offline list creation is not supported. To avoid duplicate lists, a list-creation POST is not automatically retried after a network timeout. If the app cannot confirm the result, sync your lists first and check whether the list was created before trying again.

The system tray can show or hide widgets, sync, open settings, and quit the app. The global quick-add shortcut is `Ctrl + Alt + T`. If another app has already registered that shortcut, the other features remain available.

## Local development

Stack: Tauri v2, Rust, React, TypeScript, Vite, SQLite, and Microsoft Graph.

Requirements: Node.js 22.12+, Rust, Windows C++ build tools, and WebView2. Frontend tests use Node.js's `--experimental-strip-types` option. Clone the current default branch and install frontend dependencies:

```powershell
git clone --branch my-widget https://github.com/ArthurLloyd/Todo-Desktop-Widget.git
cd Todo-Desktop-Widget
npm ci
```

Sign-in requires the Application (client) ID of a Microsoft Entra public desktop client. Configure the mobile and desktop application redirect URI as `http://localhost`, with the delegated permissions `User.Read`, `Tasks.ReadWrite`, and `offline_access`. No client secret is required.

```powershell
$env:MICROSOFT_CLIENT_ID = "your application client id"
$env:MICROSOFT_TENANT = "common"
npm run tauri:dev
```

`npm run dev` starts only the frontend. Use the full Tauri app to verify multiple windows, the database, sign-in, and system integration.

```powershell
npm run build
npm run test:frontend
npm run version:check
cargo check --manifest-path src-tauri/Cargo.toml
cargo fmt --check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
```

See the [0.2.1 validation record](docs/VALIDATION-0.2.1.md) for current check results and native verification coverage. The [0.2.0 validation record](docs/VALIDATION-0.2.0.md) is retained for reference. Both records are in Chinese.

Key code: `src/App.tsx` handles window routing and data connections; `src/components/` contains UI components; `src-tauri/src/widget_config.rs` manages window configuration; `desktop_window.rs` handles desktop layering; `commands.rs` exposes app commands; and `graph.rs` / `sync.rs` handle remote APIs and synchronization.

## Windows packaging and upgrades

```powershell
$env:MICROSOFT_CLIENT_ID = "your application client id"
$env:MICROSOFT_TENANT = "common"
npm run release:windows
```

The release script can also read build configuration from a Git-ignored `.env` file. It checks version consistency, runs frontend tests and Rust checks and tests, then builds the application EXE, NSIS installer, and MSI package and verifies artifact metadata.

```text
src-tauri/target/release/ms-todo-desktop-widget.exe
src-tauri/target/release/bundle/nsis/Todo Desktop Widget_0.2.1_x64-setup.exe
src-tauri/target/release/bundle/msi/Todo Desktop Widget_0.2.1_x64_en-US.msi
```

NSIS is recommended for personal installations and installs for the current user. Version 0.2.1 retains the application identifier and data directory from 0.2.0, so an in-place upgrade preserves your account, tasks, and settings. Before upgrading, quit the old version from the system tray, then run the installer using the same package type and installation directory. If prompted about the old version, choose to upgrade without uninstalling first, and leave any option to delete application data unchecked. Check the launch-at-startup setting after upgrading. Installers are distributed through Releases rather than Git source history. Current builds are unsigned, so Windows may display an unknown publisher warning.

See the [0.2.1 validation record](docs/VALIDATION-0.2.1.md) (Chinese) for completed checks and areas that still need native testing. Startup on other computers, multiple monitors, and different DPI settings need testing in those environments. Older release-candidate reports in the repository are historical and do not describe the current version's validation status.

## Data and privacy

The default Windows data location is:

```text
%APPDATA%\com.local.ms-todo-desktop-widget\cache.sqlite3
```

The database contains cached tasks, pending sync operations, widget configuration, and themes. Installing the app on another drive does not change the data location; portable data mode is not available. To back up your data, quit the app first, then copy the data directory.

Refresh tokens are stored in Windows Credential Manager where possible; a compatibility fallback may retain a value in SQLite settings. Deleting the installation directory or AppData does not necessarily remove system credentials. Do not commit `.env`, local build configuration, databases, tokens, keys, or certificates. An application client ID is a public client identifier, but this repository still keeps local configuration outside Git.

## macOS and project credits

The repository retains macOS build support and a WidgetKit extension, which must be built on macOS. Windows desktop behavior and installation validation do not establish macOS compatibility. See the [macOS Widget README](macos-widget/README.md) for extension configuration.

- Upstream: [Chenzo1001/Microsoft-Todo-Desktop-Widget](https://github.com/Chenzo1001/Microsoft-Todo-Desktop-Widget).
- Customization and improvements: **涂风 (Tu Feng)**.
- The repository does not currently include a code `LICENSE` file. Upstream attribution and author credits are retained. Theme asset sources, attribution, and licenses are recorded separately in the [theme asset notices](src/assets/themes/NOTICE.md) (Chinese); those terms do not imply the same license for other code or assets in this repository.
- UI references: [Uiverse](https://uiverse.io/), [React Bits](https://www.reactbits.dev/), [Mobbin](https://mobbin.com/), and [VibeHub](https://vibe-hub.org/). Control states, grouping, and lightweight feedback informed the interface. The implementation uses the project's existing React/CSS and Lucide icons without adding component dependencies from those sites.
- List creation API: [Microsoft Graph — Create todoTaskList](https://learn.microsoft.com/en-us/graph/api/todo-post-lists?view=graph-rest-1.0).
