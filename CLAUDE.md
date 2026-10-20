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
- Build folders (`app/src-tauri/target`, `crates/radial-packs/target`) once grew to 18 GB: Cargo keeps every old build file. Debug info is now trimmed in `Cargo.toml`; if a folder passes ~4 GB anyway, close the app and run `powershell -File scripts	rim_target.ps1` (or `-Clean`). The repository sits in OneDrive, which syncs `target` too; keep that in mind before adding large generated output.
- Use `npm ci` or `npm i --ignore-scripts <pkg>`; a plain `npm install` may be blocked by a hook in the original workspace.

## macOS specifics

- The Shift+drag listener needs the Input Monitoring permission. Local builds are signed with a self-signed certificate named "Radial Convert Dev" so the permission survives rebuilds. Check the log line `[hook] listen access granted:` rather than asking the user to reset the permission.
- To capture logs, launch with `open --stderr LOG --stdout LOG "<path>/Radial Convert.app"`.
- `hdiutil create`-style disk-image creation may be refused inside some sandboxes; unpacking real `.dmg` files works.
