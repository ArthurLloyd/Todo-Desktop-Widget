# Windows Release Checklist

Use this checklist for each Windows release of Todo Desktop Widget. The current release process intentionally does not include an auto-updater.

## Version and source state

- [ ] Confirm the intended version in `package.json`.
- [ ] Run `npm run version:check`; Cargo, package-lock, Tauri, installer, and About metadata must agree.
- [ ] Review `git status`, `git diff`, and `git diff --cached`.
- [ ] Confirm generated schema line-ending artifacts and all unrelated files are excluded from the release change.
- [ ] Confirm the repository has an appropriate license before public distribution.

## OAuth and secrets

- [ ] Export `MICROSOFT_CLIENT_ID` for the Microsoft Entra public desktop application.
- [ ] Use `MICROSOFT_TENANT=common` unless this release intentionally targets a narrower account type.
- [ ] Confirm the Entra registration permits public client/native PKCE flows and loopback redirect URIs.
- [ ] Confirm delegated scopes remain `User.Read`, `Tasks.ReadWrite`, and `offline_access`.
- [ ] Confirm there is no Client Secret, access token, refresh token, `.env`, PFX, PEM, or private key in Git or bundle resources.
- [ ] Run `npm run release:preflight`; it reports only whether the Client ID is configured and never prints its value.

## Automated validation and build

- [ ] Run `npm run release:windows`.
- [ ] Confirm frontend build and all frontend helper tests pass.
- [ ] Confirm `cargo check`, `cargo fmt --check`, and all Rust tests pass.
- [ ] Confirm the Tauri release EXE has no console window.
- [ ] Confirm `npm run release:smoke` finds the EXE, NSIS installer, and MSI with matching version metadata.

## Artifact locations

- [ ] EXE: `src-tauri/target/release/ms-todo-desktop-widget.exe`
- [ ] Recommended personal installer: `src-tauri/target/release/bundle/nsis/Todo Desktop Widget_<version>_x64-setup.exe`
- [ ] Managed deployment installer: `src-tauri/target/release/bundle/msi/Todo Desktop Widget_<version>_x64_en-US.msi`
- [ ] Do not publish PDBs, build intermediates, `.env`, runtime config, databases, or credentials.

## Clean install and first run

- [ ] Back up any existing `%APPDATA%\com.local.ms-todo-desktop-widget` data before destructive testing.
- [ ] Install NSIS as the current user without an administrator prompt.
- [ ] Confirm Start menu search finds **Todo Desktop Widget** and the optional desktop shortcut targets the installed EXE.
- [ ] Confirm the app uses the product icon in the installer, Start menu, executable, tray, and task switcher where applicable.
- [ ] Confirm First Run explains Microsoft connection and Settings remains reachable before login.
- [ ] Confirm a release built without a Client ID clearly reports that Microsoft connection is not configured.
- [ ] Confirm Microsoft login, loopback callback, token persistence, initial sync, Task Details, task CRUD, and offline delete.

## Runtime behavior

- [ ] Launch the installed EXE twice; only one application instance and one SyncCoordinator remain.
- [ ] With Primary Normal, second launch and tray click show, focus, and foreground the Primary window.
- [ ] With Primary Pinned, second launch and tray click keep it desktop-bottom and below normal applications.
- [ ] Restore multiple open widgets, geometry, pinned state, pane layout, resize weights, themes, and task details.
- [ ] Confirm `Win+D`, Normal/Pinned switching, multi-monitor placement, and Chinese-character user paths.
- [ ] Enable autostart and confirm the Run entry targets the installed release EXE, not a development `target/debug` path.
- [ ] Disable autostart and confirm the OS entry is removed.

## Upgrade and persistence

- [ ] Install the previous test build, create representative widget/theme/pane/task state, and log in.
- [ ] Install the newer build over it without deleting AppData.
- [ ] Confirm `%APPDATA%\com.local.ms-todo-desktop-widget\cache.sqlite3` remains readable and migrations run once.
- [ ] Confirm widget bootstrap records are not duplicated and all tokens remain in Windows Credential Manager.
- [ ] Confirm About and installer metadata report the new version.

## MSI and uninstall

- [ ] Separately test the MSI on a clean machine or VM; it uses machine scope and may require elevation.
- [ ] Confirm MSI and NSIS shortcuts launch the same stable identifier and do not create duplicate application data roots.
- [ ] Uninstall and confirm program files and shortcuts are removed.
- [ ] Confirm NSIS removes the **Todo Desktop Widget** autostart Run entry when uninstalling (not when updating).
- [ ] Leave **Delete app data** unchecked and confirm roaming AppData is retained; repeat with it checked only after backing up test data.
- [ ] Confirm Windows Credential Manager data and the current SQLite refresh-token fallback follow the documented retention policy; use in-app Logout before uninstall when a full account reset is required.
- [ ] Reinstall and confirm retained data/login behavior matches the documented policy.

## Signing and publishing

- [ ] For private testing, document that artifacts are unsigned and SmartScreen warnings are expected.
- [ ] For public distribution, configure a trusted code-signing certificate, SHA-256 digest, and provider-recommended timestamp URL outside Git.
- [ ] Verify signatures on the EXE, NSIS installer, MSI, and uninstaller before publishing.
- [ ] Publish NSIS as the recommended personal installer; describe MSI as the managed deployment option rather than presenting both as equivalent choices.
