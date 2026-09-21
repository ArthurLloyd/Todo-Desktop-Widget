# Todo Desktop Widget 0.1.0 RC2 Test Report

## Candidate

- Product: Todo Desktop Widget
- Version: 0.1.0
- Candidate: RC2
- Test date: 2026-09-22
- OS: Windows 11 Home, Chinese edition, build 10.0.26200
- Architecture: x64
- Observed system DPI: 96 (100%)
- Detected displays: 2
- Signing: Unsigned (`NotSigned`)
- Recommended installer: NSIS current-user setup
- Managed deployment installer: MSI

No Microsoft account, Client ID, OAuth token, database contents, or credential value is
recorded in this report.

## Artifact manifest

| Artifact | SHA256 | Signature |
| --- | --- | --- |
| `Todo Desktop Widget_0.1.0_x64.exe` | `14462F43C6C52BC4C2696B1588C93E4C506BED1E357771BCD2214E33CD76E99E` | NotSigned |
| `Todo Desktop Widget_0.1.0_x64-setup.exe` | `9FF6A051F8756BF605D550E0DFDD7FB9834B3B38B194B584095E64970A70D86E` | NotSigned |
| `Todo Desktop Widget_0.1.0_x64_en-US.msi` | `DA58148183FEBA8C32E94C72CB65212199A464E26677AB43E84D6A6941FB8789` | NotSigned |

The authoritative copy of these values is the `RELEASE_MANIFEST.txt` stored beside the
RC2 artifacts. Release configuration and `.env` are not copied into that directory.

## Automated validation

| Status | Validation | Result |
| --- | --- | --- |
| PASS | Version consistency | 0.1.0 across package, Cargo, Tauri, installer, and About metadata |
| PASS | Frontend build | TypeScript and Vite production build completed |
| PASS | Frontend tests | 29 passed, 0 failed |
| PASS | Rust check | Completed |
| PASS | Rust format check | Completed |
| PASS | Rust tests | 67 passed, 0 failed |
| PASS | Tauri Windows bundle | Standalone EXE, NSIS, and MSI produced |
| PASS | Release smoke | Artifact presence and EXE version metadata verified |
| PASS | Real Client ID build | Embedded value found without printing it; synthetic test GUID absent |
| PASS | About attribution | Bilingual upstream attribution, normalized repository URL, and the improvement-author credit `涂风` are present in the production frontend bundle |

## Installed-build validation

| Status | Area | Evidence |
| --- | --- | --- |
| PASS | Host NSIS install | Final process ran from `%LOCALAPPDATA%\\Todo Desktop Widget` |
| PASS | AppData location | Cache remained under `%APPDATA%\\com.local.ms-todo-desktop-widget` |
| PASS | OAuth | Real public-client loopback login completed |
| PASS | OAuth restart persistence | Quit and host restart retained login and synchronized data |
| PASS | Initial sync | Five lists and local task cache were populated; Outbox converged to zero |
| PASS | Online CRUD | Create, Task Details edit, complete, delete, and Microsoft To Do verification passed |
| PASS | Primary Task Details | Title, importance, due date, notes, save/refresh, and delete path passed |
| PASS | Same-version reinstall | OAuth and AppData survived RC1 to RC2 replacement |
| PASS | Normal Win+D | RC2 Normal window disappears with Show Desktop |
| PASS | Pinned Win+D | RC2 Pinned window remains visible |
| PASS | Pinned application coverage | Chrome covers the pinned widget |
| NOT TESTED | Full offline lifecycle | Offline mutations before restart were accepted, but final restart/reconnect/cloud assertions were interrupted |
| NOT TESTED | Multi-window / Multi-pane | Not repeated in the installed RC2 |
| NOT TESTED | Pane resize persistence | Not repeated in the installed RC2 |
| NOT TESTED | Theme and manager persistence | Not repeated in the installed RC2 |
| NOT TESTED | Installed single instance | Earlier sandbox-run evidence was discarded; host RC2 routing was not repeated |
| NOT TESTED | Autostart | No matching HKCU Run entry at closeout; sign-out/sign-in not performed |
| NOT TESTED | Multi-monitor | Two displays detected, but movement and restart restore were not exercised |
| NOT TESTED | 125% / 150% DPI | Only 100% system DPI was observed |
| NOT TESTED | Upgrade 0.1.0 to 0.1.1 | Temporary upgrade candidate was not built |
| NOT TESTED | Host NSIS uninstall options | Retain/delete AppData behavior not repeated for RC2 |
| NOT TESTED | Credential Manager retention | Credential metadata and Logout cleanup not inspected end-to-end |
| NOT TESTED | MSI lifecycle | Bundle built but not installed or uninstalled |
| NOT TESTED | SmartScreen | Prompt behavior was not recorded |
| NOT TESTED | Icon surfaces | Not visually audited across EXE, Start Menu, desktop, installer, and tray |
| NOT TESTED | About and debug UI | Metadata and upstream attribution are build-consistent; final RC2 UI was not visually rechecked |

## Issues discovered in 5C

### Environment contamination: Codex sandbox installation

The first automated NSIS launch was started by the packaged Codex process. Windows redirected
the per-user installation to the Codex package `LocalCache`, and the resulting shortcut reused
that virtualized location. This made restart behavior look like an OAuth persistence failure.
The result was discarded, the virtual installation was uninstalled, and the installer was run
manually from Explorer. The host installation then ran from the expected LocalAppData location
and OAuth persistence succeeded. This was a test-harness issue, not an application code defect.

### MAJOR fixed: Normal windows survived Win+D

RC1 created the primary window hidden and applied window mode before first visibility. Dynamic
secondary windows likewise no longer performed their pre-mode `show()`. Explorer did not treat
the resulting Normal widget like the previously validated ordinary top-level window, so Normal
and Pinned both survived Show Desktop.

RC2 restores the validated ordering:

- the statically configured primary window starts visible;
- a dynamic secondary is shown once before applying its persisted Normal/Pinned mode;
- Desktop Owner, `alwaysOnBottom`, and Normal restore algorithms remain unchanged.

RC2 manual verification passed: Normal disappears with Win+D, Pinned remains, and Chrome covers
Pinned widgets.

## Bug summary

- Blockers discovered: 0 unresolved
- Major bugs discovered: 1 application bug, fixed in RC2
- Minor bugs discovered: 0 confirmed
- Test-environment issues: 1 Codex sandbox installation virtualization, resolved by host install

## Distribution decision

RC2 is suitable for continued private/internal candidate use. It is not yet cleared for an
unqualified public release because code signing is pending and several installed lifecycle tests
remain NOT TESTED: autostart login recovery, real upgrade, MSI lifecycle, NSIS data-deletion
option, Credential Manager retention, multi-monitor restore, and non-100% DPI.
