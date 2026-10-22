# Radial Convert: notes for AI assistants

Read `README.md`, `docs/WINDOWS_HANDOFF.md`, `docs/MODULAR_INSTALL_PLAN.md` and `docs/RELEASE.md` first. They say what exists, what is unverified, and what is missing.

## Rules from the project owner

- **Preview design changes in the browser before rebuilding the app.** Run `npm run dev` in `app/` and open `http://localhost:1420/#ring-demo` (a playground with light/dark, backgrounds and every state). Rebuild the real app only after the design is approved.
- **Design: flat, minimal, clean.** No 3D bevels, no glass effects (Liquid Glass was tried and rejected), muted colors. The donut dial and its animations are liked; keep them.
- **No `setTimeout` or `setInterval`** in app code or tests. Use CSS animation events, state, or fake timers. Ask first if a timeout seems unavoidable.
- **One concern per file.** Interfaces, enums and constants live in their own files. UI components have separate `.tsx` and `.css` files. Keep files under 1000 lines of code.
- **Be honest about what is verified.** Say which routes were really run and which only compile or are covered by tests. Nothing here has run on Windows or on GitHub's runners yet.
- **Do not publish, upload or create repos/releases without being told to.**

## Commands

- `cd app && npm ci` to install; `npm run tauri dev` to run; `npm run build:local` for a signed macOS debug build.
- `cargo test` in `crates/radial-packs` and in `app/src-tauri`; `python3 -m unittest discover -s scripts`.
- Build folders (`app/src-tauri/target`, `crates/radial-packs/target`) once grew to 18 GB: Cargo keeps every old build file. Debug info is now trimmed in `Cargo.toml`; if a folder passes ~4 GB anyway, close the app and run `powershell -File scripts\trim_target.ps1` (or `-Clean`). The repository sits in OneDrive, which syncs `target` too; keep that in mind before adding large generated output.
- Use `npm ci` or `npm i --ignore-scripts <pkg>`; a plain `npm install` may be blocked by a hook in the original workspace.

## The manager window (`app/src/manager/`)

A sidebar plus one page at a time. Preview it at `http://localhost:1420/#manager-demo` (every theme, page and state).

- `ManagerView.tsx` composes `shell/Sidebar` and one page from `pages/` (`ConvertersPage`, `AppearancePage`, `StoragePage`, `ShortcutPage`). `Manager.tsx` is the container that talks to Tauri; everything else is presentational and gets its data as props.
- Shared pieces live in `parts/` (`Page`, `Row`, `Button`, `Chip`, `Meter`, `Logo`, `NavIcon`); the converter row is `packs/PackRow`. Plain data is in `sections.ts`, `packMeta.ts`, `themes.ts`, `hookStatus.ts`.
- Themes are **only** `skins/tokens.css`: one block of `--mg-*` tokens per theme (Current, Bold, Brutal). Components never mention a theme. To add a theme, add a block there and an entry in `themes.ts`; a test fails if any theme lacks a token the components use. Themes without a block (Petals, Compass, Fan, Hex) show the Current look in this window.
- `manager/ManagerSkins.css` is no longer used by the manager; it still supplies the `--m-*` tokens for the PDF editor window (`workspace/`), so leave it until the editor is redesigned.

## macOS specifics

- The Shift+drag listener needs the Input Monitoring permission. Local builds are signed with a self-signed certificate named "Radial Convert Dev" so the permission survives rebuilds. Check the log line `[hook] listen access granted:` rather than asking the user to reset the permission.
- To capture logs, launch with `open --stderr LOG --stdout LOG "<path>/Radial Convert.app"`.
- `hdiutil create`-style disk-image creation may be refused inside some sandboxes; unpacking real `.dmg` files works.
- The app is signed with the hardened runtime, which refuses to load libraries signed by anyone else. The PDF tools pack downloads PDFium (`libpdfium.dylib`), so `app/src-tauri/Entitlements.plist` grants `com.apple.security.cs.disable-library-validation` (wired in `tauri.conf.json`, so release builds get it too). Without it the editor fails with a very long "code signature ... not valid for use in process" error.
- PDFium crashes the process if two threads use it at once. Always go through `Library::load()`: it returns a guard that makes everything take turns (re-entrant on the same thread). Never keep a PDFium handle without that guard.
- The manager and `radial list` count programs already on the computer (Homebrew, Program Files) as available ("On this computer"), because the app uses them; that logic is `crates/radial-packs/src/availability.rs`. `RADIAL_NO_SYSTEM_TOOLS=1` hides them for testing.
- Some networks hang on the first address of GitHub's download host; downloads use a 15 s connect timeout shared across addresses (`fetch.rs`), and the UI says "Connecting" while waiting.


## The picture tool (resize / compress)

- Engine: `app/src-tauri/src/engine/imaging/` (`spec` job types, `dimensions` size maths, `resize`, `encode` JPG/PNG, `target_size` "stay under N KB" search, `orient` EXIF rotation, `pipeline` = open, resize, compress, write with numbered names). It uses the `image` crate only: no sips, no pack, same on Windows. `pipeline::render` is what both the preview and the save use.
- Window: `app/src/imagetool/` (presentational `ImageToolView`; `settings.ts` turns the controls into the engine's `Job`). Preview at `http://localhost:1420/#imagetool-demo`; the demo's numbers are fake.
- Not wired yet: Tauri commands, the ring wedge, the real window. Lossy WebP is not supported (the `image` crate writes lossless only).
