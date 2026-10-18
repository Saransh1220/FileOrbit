# Windows handoff

Everything here was written and tested on a Mac. This lists what is known to work, what has never run on Windows, and the one big piece that does not exist yet.

## The big gap: the Windows app cannot open the dial yet

`app/src-tauri/src/hook/win.rs` is a stub that only reports "unsupported". Without the global Shift+drag detection, a Windows build starts, shows a menu-bar/tray icon and the status window, and **does nothing else**. Do not publish a Windows installer as "working" until this exists.

What to build (the trigger rules are already written and tested; they are OS-independent):

- **Mouse hook:** a low-level hook (`SetWindowsHookEx` with `WH_MOUSE_LL`) on its own thread, feeding `DragTrigger` in `hook/trigger.rs` (button down, move, button up). Shift state: `GetAsyncKeyState(VK_SHIFT)`. The `windows` crate is the usual binding. Mirror what `hook/mac.rs` does.
- **Is it a file drag?** macOS reads the drag pasteboard; Windows has no way to read drag data while it is in flight. Use a heuristic (the window under the cursor belongs to Explorer or the desktop) to *open* the ring, then rely on the drop: the overlay window receives the real file paths through Tauri's drag-drop event, and if there are none it just closes. The ring's wedges need the file type up front, so either show a neutral ring first and fill it when the drag enters, or read the selection from Explorer (shell COM). Decide this early; it shapes the UX.
- **Overlay window:** the Tauri window defined in `tauri.conf.json` (transparent, always on top, no decorations). It must not take focus (`WS_EX_NOACTIVATE`), or the drag is cancelled. Position uses physical pixels on Windows (the macOS code uses logical points). Check `app/src/ring/platform.ts` too: it already treats drag positions as physical pixels on Windows.
- **Known traps:** Shift+drag in Explorer means *move*, so the overlay must accept the drop as copy and never move or delete the source. The hook cannot see windows running as administrator unless the app also does (UIPI). Multi-monitor and DPI scaling need testing.

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
2. `cd app && npm ci && npm run tauri dev`. The status window should say the hook is unsupported.
3. Run both Rust test suites and the real-download test above. Fix whatever Windows-specific problems show up.
4. Create the GitHub repo, add a `LICENSE`, replace the app icons (`npm run tauri icon`), push, and watch the `ci` workflow.
5. Build the Windows hook and overlay behaviour (the gap above).
6. Then follow `docs/RELEASE.md`: mirror the engines, add signing secrets, tag, review the draft release, publish.

## Context worth keeping

- The design is deliberately flat and minimal; preview UI changes in the browser (`npm run dev`, then `#ring-demo`) before rebuilding the app.
- macOS-only pieces: `hook/mac.rs`, `hdiutil`/`ditto` in `extract.rs`, `sips` for HEIC, `LSUIElement` in `app/src-tauri/Info.plist`.
- Environment switches for testing like a clean machine: `RADIAL_NO_SYSTEM_TOOLS=1`, `RADIAL_HOME=<empty folder>`, `RADIAL_MANIFEST=<file>` (see `docs/MODULAR_INSTALL_PLAN.md`).
- The pinned download links point at upstream sites today and will rot; mirror them before releasing (`docs/RELEASE.md`).
