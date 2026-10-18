> **Historical.** This is the first plan written before any code. It has been superseded: the modular pack installer is in `docs/MODULAR_INSTALL_PLAN.md`, the release process is in `docs/RELEASE.md`, and what is built vs. missing is in `docs/WINDOWS_HANDOFF.md`. SUPERSEDED.

# Radial Convert — Plan (working name)

Hold **Shift**, drag any file, and a ring of target formats opens around your cursor. Drop on a wedge and it converts locally. Heavy targets (PDF, video) open a small options sheet first (page reorder, quality, trim).

Targets: macOS now, Windows from the same codebase (Linux is possible later but out of scope).

## 1. Tech stack

**Tauri 2 (Rust core) + TypeScript UI (React + Vite)**, with conversion engines run as bundled sidecar binaries.

| Concern | Choice | Why |
|---|---|---|
| App shell | Tauri 2 | One codebase for macOS and Windows. About 10 MB installer, against about 150 MB for Electron. It has transparent, always-on-top windows and file-drop events that carry real paths. |
| Native glue | Rust | Global input hook, overlay window and process supervision are the only native parts, so the Rust surface stays small. |
| UI | React + TS + Vite, Framer Motion or plain SVG/CSS for the ring | You already know TS. The radial menu is just SVG with CSS transitions. |
| Job queue | Rust (tokio) | Spawns sidecars, parses progress, supports cancel, runs a few jobs in parallel. |
| PDF page tools | `pdf-lib` + `pdf.js` in the UI | Thumbnails, drag-to-reorder, delete and rotate pages with no native dependency. |

Not chosen:
- **Electron**: heavy, but you already know Node. Fallback if Rust becomes a blocker.
- **Go + Wails**: fine, but the global-hook and drop-target ecosystem is weaker.
- **Python (PyQt)**: slow startup, awkward packaging on Windows.
- **Swift/SwiftUI**: great on Mac, but does not compile on Windows.

Installed here: Node 24, Python 3.14, Go, pandoc. **Missing: Rust toolchain** (`rustup`), ffmpeg, LibreOffice, ImageMagick/vips, Ghostscript. Rust has to be installed before step 1 of the build.

## 2. The hard part: detecting "Shift + file drag" system-wide

A normal app only sees drags that land on its own window. We need to react while the drag is still in flight over Finder or Explorer.

**Trigger flow**
1. A global listener (Rust, `CGEventTap` on macOS and `SetWindowsHookEx(WH_MOUSE_LL)` on Windows, wrapped via `rdev` or direct bindings) watches for **Shift held + left button down + mouse moved more than ~6 px**.
2. Confirm it is a *file* drag, not a text selection or window drag.
   - macOS: read the drag pasteboard (`NSPasteboardNameDrag`) for file URLs.
   - Windows: there is no clean way to read in-flight drag data. Use a heuristic (foreground app is Explorer or Desktop, plus button held) and validate when the drop lands.
3. Show a **transparent, borderless, always-on-top overlay window** centered on the cursor. It is a real drop target, so the OS delivers the drop to it with file paths.
4. Each wedge is an HTML element listening for drag-over and drop. On drop, the path goes to the job queue.
5. Esc, or dropping on the center or outside the ring, cancels. The overlay closes.

**Known gotchas (to prototype first, in phase 0)**
- **Shift changes the drop effect.** Explorer treats Shift+drag as *move*, and Finder may treat it as an alias or move. The overlay must accept the drop and *copy* only. The source file must never be moved or deleted.
- **macOS permissions.** A global hook needs Input Monitoring or Accessibility permission. Needs a first-run onboarding screen that deep-links to System Settings.
- **Overlay must not steal focus** or it cancels the drag. Use a non-activating panel (`NSPanel` style on Mac, `WS_EX_NOACTIVATE` on Windows).
- **Windows UIPI.** The hook does not see elevated windows unless our app is elevated too. Document it.
- **Multi-monitor and HiDPI.** The ring must clamp to screen edges and use the right scale factor.

Phase 0 exists to prove or kill this one risk before anything else is built.

## 3. Conversion engine design

A `Converter` trait (Rust) with a **registry**, which is a graph of `format → format` edges. Each edge names an engine, its cost and its options schema. If there is no direct edge, the router can chain (for example `docx → pdf → png`).

| Category | Engine | Examples |
|---|---|---|
| Audio / video | **FFmpeg** sidecar | mp3↔wav↔flac↔aac↔ogg, mp4↔mov↔mkv↔webm↔gif, extract audio |
| Images | **libvips** or ImageMagick sidecar (HEIC support is the deciding factor) | png↔jpg↔webp↔avif↔heic↔tiff↔bmp, svg→png |
| Office docs | **LibreOffice headless** | doc/docx/odt/rtf↔pdf, xls/xlsx/ods↔csv/pdf, ppt/pptx→pdf |
| Text / markup | **pandoc** | md↔docx↔html↔epub↔txt, tex |
| PDF ops | `pdf-lib` and Ghostscript or qpdf | merge, reorder, split, compress, pdf→png |
| Archives (later) | native libs | zip↔tar.gz↔7z |

**Packaging.** FFmpeg, vips and pandoc are small enough to bundle per OS. LibreOffice is about 300 MB, so it is an optional on-demand download the first time a document is converted (or reuse an existing install if found).

**Ring contents are contextual.** The wedges come from the registry, filtered by the dragged file's extension and MIME type. A `.docx` shows PDF, ODT, TXT, HTML, MD. An `.mp3` shows WAV, FLAC, AAC, OGG. With more than 8 targets, the ring pages (Tab or scroll to rotate) or groups into a second ring.

## 4. UX

- **Ring:** glassy translucent wedges with icons and labels, a spring animation on open, and the hovered wedge scaling up. The center shows the file name, type and size.
- **Options sheet** (after dropping on formats with options):
  - PDF target: page thumbnails with drag-to-reorder, delete, rotate, page range, then Convert.
  - Video / audio: quality preset, trim, bitrate.
  - Image: quality slider, resize.
  - A "skip options, use defaults" setting for one-gesture conversions.
- **Progress:** a small toast with a progress bar and cancel. Finished: "Reveal in Finder / Explorer".
- **Output:** next to the source by default, collision-safe naming (`report (1).pdf`), configurable folder.
- **Tray / menu-bar icon** with settings: trigger key (Shift by default), enabled formats, start at login, output folder.
- Fully **offline**. Nothing leaves the machine.

## 5. Repo layout

```
radial-convert/
  src-tauri/
    src/
      hook/          # global input listener (mac.rs, win.rs, mod.rs)
      overlay/       # overlay window lifecycle + positioning
      engine/        # Converter trait, registry, router, job queue
      engines/       # ffmpeg.rs, libreoffice.rs, pandoc.rs, vips.rs, pdf.rs
      settings/
    binaries/        # sidecars per target triple
  src/               # React UI
    ring/            # radial menu
    options/         # per-format option sheets, pdf reorder
    toast/ settings/
  docs/
```

Following your workspace rules: one concern per file, interfaces/enums/constants in their own files, separate `.ts`/`.html`/`.css` equivalents (React: `.tsx` + `.css`), no `setTimeout`, files under 1000 code lines.

## 6. Phases

| Phase | Deliverable | Done when |
|---|---|---|
| **0. Risk spike** (2-3 days) | Rust + Tauri hello-world. Shift+drag from Finder opens an overlay ring, and a drop returns the file path. Same spike on Windows. | Drop path is received without moving the source file, on both OSes |
| **1. Core engine** | Registry, router, job queue, FFmpeg + pandoc + vips engines, CLI harness | `mp3→wav`, `md→docx`, `png→webp` work from a test CLI |
| **2. Ring UI** | Contextual wedges, animations, cancel, edge clamping, multi-monitor | Smooth ring for audio, image and doc inputs |
| **3. Options + PDF** | Options sheets, PDF reorder/delete/rotate, progress toast | Docx → PDF with reordered pages end-to-end |
| **4. Office + polish** | LibreOffice on-demand download, tray, settings, permissions onboarding, errors | Doc/Excel/PPT → PDF works with a clean first-run |
| **5. Windows parity + packaging** | Windows hook/overlay hardening, signed `.dmg` and `.msi`/NSIS, CI matrix (GitHub Actions on macOS + Windows) | Installers build in CI and pass a smoke test |
| **6. Later** | Archives, OCR, batch drag of many files, Linux | n/a |

## 7. Risks

1. Cross-app drag detection, especially on Windows (phase 0 mitigates).
2. Shift changing the drop effect to move (overlay must force copy).
3. macOS permission friction on first run, and notarization for distribution.
4. LibreOffice size and cold-start time (about 3-8 s for the first conversion).
5. Code signing and notarization cost (Apple Developer account, Windows certificate).
6. Licensing: FFmpeg builds must be LGPL (no GPL codecs bundled unless we accept GPL), and LibreOffice is MPL.

## 8. Open decisions

- Name of the app.
- Trigger key: Shift as asked (conflicts with the Explorer "move" gesture) versus a configurable option, such as Alt/Option.
- Whether to ship LibreOffice bundled or as an on-demand download.
- Rust: OK to adopt for the native layer, or prefer Electron + Node to stay in TS?
