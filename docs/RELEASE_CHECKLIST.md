# Windows Release Checklist

Use this checklist for each Windows release of Todo Desktop Widget. The current release process intentionally does not include an auto-updater.

## 0.1.0-rc2 validation status

Status values are **PASS**, **FAIL**, and **NOT TESTED**. A PASS below means the item
was observed on RC2 or verified by the final automated pipeline; earlier development-stage
validation alone is not treated as installed-build evidence.

| Status | Check |
| --- | --- |
| PASS | Version `0.1.0` is consistent across package, Cargo, Tauri, installer, and About metadata. |
| PASS | About includes bilingual attribution to the upstream `Chenzo1001/Microsoft-Todo-Desktop-Widget` project and credits `涂风` as the improvement author. |
| PASS | Git state was reviewed; generated schemas, local release configuration, and RC artifacts are excluded. |
| NOT TESTED | Repository license suitability still requires explicit owner review before public distribution. |
| PASS | A real Entra public-client ID and tenant `common` were supplied outside Git. |
| PASS | PKCE loopback OAuth, scopes, restart persistence, initial sync, and installed online CRUD passed. |
| PASS | No Client Secret, token, `.env`, certificate, database, or credential data is included in the RC directory. |
| PASS | Frontend build/tests (29/29), Rust check/fmt/tests (67/67), Windows bundles, and smoke test passed. |
| PASS | Standalone EXE, NSIS, MSI, hashes, and unsigned status are recorded in the RC2 manifest. |
| PASS | Host NSIS installation runs from `%LOCALAPPDATA%\\Todo Desktop Widget`; AppData remains under `%APPDATA%`. |
| PASS | Same-version reinstall retained OAuth and cached configuration. |
| PASS | RC2 Normal disappears with Win+D; Pinned remains and is covered by Chrome. |
| PASS | Primary Task Details and online Create/Edit/Complete/Delete passed. |
| NOT TESTED | Full offline restart/reconnect and final official-client assertions were interrupted. |
| NOT TESTED | Installed RC2 multi-window, multi-pane, pane resize, Theme Manager, and Widget Manager were not repeated. |
| NOT TESTED | Installed single-instance routing was not repeated after the host reinstall. |
| NOT TESTED | Autostart enable/disable and Windows sign-out/sign-in recovery were not exercised. |
| NOT TESTED | Multi-monitor restore was not exercised; two displays were detected. |
| NOT TESTED | Only 96 DPI / 100% was observed; 125% and 150% were not exercised. |
| NOT TESTED | A Chinese-character Windows profile was not available. |
| NOT TESTED | Temporary `0.1.1` upgrade and migration retention were not exercised. |
| NOT TESTED | Host NSIS uninstall retain/delete-AppData options and credential retention were not exercised on RC2. |
| NOT TESTED | MSI installation, CRUD, autostart, and uninstall were not exercised. |
| NOT TESTED | SmartScreen presentation and all icon surfaces were not recorded. |
| PASS | RC2 is explicitly unsigned; trusted signing remains pending for public release. |

See `docs/RC_TEST_REPORT.md` for detailed evidence, the fixed Win+D regression, and
remaining release blockers. The reusable procedural checklist follows.

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
