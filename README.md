# Radial Convert

Hold **Shift**, drag a file, and a ring of formats opens around your cursor. Drop the file on one and it converts, locally, next to the original. Nothing leaves your computer.

```
report.docx  --Shift+drag-->  [ PDF  ODT  TXT  HTML  MD ]  --drop on PDF-->  report.pdf
song.mp3     --Shift+drag-->  [ WAV  FLAC  AAC  M4A ]      --drop on WAV-->  song.wav
```

Existing files are never overwritten: a second conversion gives `report (1).pdf`.

## Status

| Platform | State |
|---|---|
| **macOS** | Works: Shift+drag, the dial, conversions, install prompts, the `radial` CLI. |
| **Windows** | **Partly working.** Shift+drag from Explorer or the desktop opens the dial, and a drop on a wedge reaches the conversion code (tried on one Windows 11 PC with a missing-pack prompt; conversions themselves, the `radial` CLI, `install.ps1` and the installer have not been run there). See `docs/WINDOWS_HANDOFF.md`. |

## Converters are optional packs

The app downloads only the converters you choose.

| Pack | Converts | Program |
|---|---|---|
| Documents | Word, Excel, PowerPoint, RTF to PDF and each other | LibreOffice |
| Text & Markup | Markdown, HTML, EPUB, TeX, plain text | pandoc |
| Audio | MP3, WAV, FLAC, AAC, M4A, OGG | ffmpeg |
| Video | MP4, MOV, MKV, WEBM, GIF, extract audio | ffmpeg |
| Images | PNG, JPG, WEBP, TIFF, BMP, GIF | ffmpeg |

Pick them with `radial install` (a multi-select in the terminal), or just drop a file: if a pack is missing, the app offers to install it on the spot.

## Install (once a release is published)

```
macOS    curl -fsSL https://github.com/OWNER/REPO/releases/latest/download/install.sh | sh
Windows  irm https://github.com/OWNER/REPO/releases/latest/download/install.ps1 | iex
```

## Develop

Needs Rust (stable) and Node 22. On Windows also the "Desktop development with C++" build tools.

```
cd app
npm ci
npm run tauri dev          # run the app (macOS: needs the Input Monitoring permission)
npm run dev                # just the UI; open http://localhost:1420/#ring-demo for the design playground
npm run build:local        # macOS debug app signed with a local dev certificate (see docs/RELEASE.md)

cd ../crates/radial-packs && cargo test      # the pack installer and `radial` CLI
cd ../../app/src-tauri && cargo test         # the app's engine
python3 -m unittest discover -s scripts      # install.sh and the mirror tool (install.sh test is macOS-only)
```

## Layout

| Path | What |
|---|---|
| `app/` | The Tauri app: React UI in `src/`, Rust in `src-tauri/` |
| `app/src-tauri/src/engine/` | Format registry, converters, jobs, naming |
| `app/src-tauri/src/hook/` | Global Shift+drag detection (`mac.rs`; `win/` mouse hook + Explorer selection) |
| `crates/radial-packs/` | The pack installer library and the `radial` CLI |
| `scripts/` | Install scripts and the engine-mirroring tool |
| `docs/` | Plans and the release guide |

## Licenses

The converter programs keep their own licenses; see `THIRD_PARTY_NOTICES.md`. Add a `LICENSE` file for this project's own code before publishing.
