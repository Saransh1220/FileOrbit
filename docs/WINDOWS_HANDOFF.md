# Windows handoff

Everything here was written and tested on a Mac. This lists what is known to work, what has never run on Windows, and the one big piece that does not exist yet.

## How the Windows Shift+drag works (built, lightly tested)

Tried by hand on one Windows 11 PC: the dial opens on Shift+drag from an Explorer window, the drop is received, and a missing pack triggers the install prompt. Not yet tried: a full conversion, the desktop, Explorer tabs beyond one, several monitors, mixed DPI.

- **Mouse hook** (`hook/win/mouse_hook.rs`): `WH_MOUSE_LL` on its own thread. It only forwards events over a channel; `hook/win/mod.rs` runs the shared `DragTrigger` on a worker thread.
- **Is it a file drag?** (`hook/win/explorer.rs`): Windows cannot show what is inside an in-flight drag, so the press must start on an Explorer window or the desktop, and the files are that window's selection, read through the shell COM API (`IShellWindows`, the visible tab). The ring's wedges therefore know the file type up front, as on macOS.
- **Overlay** (`overlay_native.rs`): shown as a non-activating tool window so the drag is not cancelled. Positioned in physical pixels using the monitor's scale.
- **Receiving the drop** (`overlay_catcher.rs`, `overlay_drop.rs`): the webview's own drop handling never saw Explorer's drag (its windows belong to the WebView2 process), so an invisible native window is laid over the ring and carries our own OLE drop target. It always answers "copy", so the source file is never moved, and it sends `ring-drag` events to the page. `useRingDrag.ts` uses those events on Windows and Tauri's drop events on macOS.
- **Debugging a dead drag:** a plain WinForms window received the same Explorer drag, which is how the webview was ruled out as the receiver.

Still to do for Windows: elevated Explorer windows (UIPI), multi-monitor and DPI checks, edge clamping of the ring, and a keyboard Esc to cancel.

## Things that exist but have never run on Windows

| Item | Where | How to check |
|---|---|---|
| Unpacking the LibreOffice `.msi` with `msiexec /a` | `crates/radial-packs/src/extract.rs` | `cargo test --test real_downloads windows_x86_64_all_packs -- --ignored --nocapture` (about 600 MB). If `soffice.exe` is not at `LibreOffice/program/soffice.exe` the installer searches for a unique file of that name. |
| Windows ffmpeg (shared build with DLLs) and pandoc | same test | Same command. The test only checks files are in place; also run `radial install --packs audio,text` then `radial doctor`. |
| The whole `radial` CLI | `crates/radial-packs/src/main.rs` | `cargo run -- install` in a Windows terminal; check the multi-select works in Windows Terminal and `cmd`. |
| Tests that are skipped on Windows | `installer.rs` tests are `#[cfg(unix)]` because they use shell scripts as fake engines | Write Windows equivalents (a tiny `.bat` or `.exe` fixture). |
| App engine on Windows | `app/src-tauri/src/engine/` | `cargo test` in `app/src-tauri`. Conversions that need an engine skip when it is missing. |
| Revealing a file | `app/src-tauri/src/reveal.rs` uses `explorer /select,` | Click **Show** on a finished toast. |
| `install.ps1` | `scripts/install.ps1` | **Never executed.** Read it, then run it against a test release. |
| NSIS installer build | `release.yml`, job `app-windows` | Run the workflow by hand from the Actions tab. |
| CI itself | `.github/workflows/*.yml` | Valid YAML only. Never run on GitHub. |

## Order I would do things in

1. Install Rust (MSVC toolchain), Node 22, Git, the GitHub CLI. Windows 11 already has WebView2.
2. `cd app && npm ci && npm run tauri dev`. Start it from your own terminal, not from an automated or sandboxed shell. Install the "Desktop development with C++" build tools and the Windows SDK first.
3. Run both Rust test suites and the real-download test above. Fix whatever Windows-specific problems show up.
4. Create the GitHub repo, add a `LICENSE`, replace the app icons (`npm run tauri icon`), push, and watch the `ci` workflow.
5. Harden the Windows hook and overlay (see "Still to do" above).
6. Then follow `docs/RELEASE.md`: mirror the engines, add signing secrets, tag, review the draft release, publish.

## Remaining work (kept here so it survives a new context)

Windows behaviour
- [ ] Elevated Explorer windows: the hook cannot see them unless the app is elevated too (UIPI). Decide: document it, or offer an elevated mode.
- [ ] Several monitors and mixed DPI: check ring placement, size and wedge hit-testing.
- [ ] Keep the ring inside the screen edges (it can currently spill off).
- [ ] Esc cancels the ring.
- [ ] Desktop and Explorer tabs: only lightly tried; also try network folders and "Search results" views.
- [ ] Download speed: resumable and parallel (range request) downloads; maybe a smaller audio-only ffmpeg. The wait is network-bound (measured 0.3 to 1.4 MB/s here), not our code.

Tests and release
- [ ] Windows versions of the `installer.rs` tests that are `#[cfg(unix)]` (fake engines are shell scripts).
- [ ] Run `real_downloads` with `--ignored` for pandoc and LibreOffice on Windows (ffmpeg is done and works).
- [ ] Try the Documents pack on Windows (LibreOffice `.msi` unpack is still unverified).
- [ ] Add a `LICENSE`, replace the scaffold icons (`npm run tauri icon`).
- [ ] First CI run on GitHub (`ci` workflow), then the release workflow by hand.
- [ ] Run and fix `scripts/install.ps1`; build and try the NSIS installer.
- [ ] Mirror the engine downloads (`docs/RELEASE.md`) before any release.
- [ ] Windows code signing (optional, SmartScreen warning otherwise).

Design
- [x] Themes: Current, Bold, Brutal, Petals, Compass, Fan and Hex, chosen from the tray icon menu under Theme (saved in `settings.json`, applied live to the ring, toast and status window). Add a design by adding a dial in `app/src/ring/variants/`, a registry line in `ringVariants.ts`, toast and status skins in `ToastSkins.css` / `StatusSkins.css`, and an entry in `app/src-tauri/src/design.rs` (a test checks the two lists match).
- [ ] More design options were requested after Bold and Brutal. Five earlier designs (Orbit, Blueprint, Halo, Chips, Clock) are kept in git under the tag `designs-archive-8`.
- [ ] Check every theme on macOS (the UI is shared, but only Windows has been run).

## Context worth keeping

- The design is deliberately flat and minimal; preview UI changes in the browser (`npm run dev`, then `#ring-demo`) before rebuilding the app.
- macOS-only pieces: `hook/mac.rs`, `hdiutil`/`ditto` in `extract.rs`, `sips` for HEIC, `LSUIElement` in `app/src-tauri/Info.plist`.
- Environment switches for testing like a clean machine: `RADIAL_NO_SYSTEM_TOOLS=1`, `RADIAL_HOME=<empty folder>`, `RADIAL_MANIFEST=<file>` (see `docs/MODULAR_INSTALL_PLAN.md`).
- The pinned download links point at upstream sites today and will rot; mirror them before releasing (`docs/RELEASE.md`).
