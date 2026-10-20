# PDF tools plan

Status: **plan only, nothing built yet.** Written 2026-10-21 after the Windows port, themes and manager window.

Goal: PDFs become first-class. You can convert them to other formats, and you can open them in a page editor to reorder, delete, rotate and merge, before anything is saved. Merging several PDFs and combining images into a PDF come from the same editor.

## 1. What other apps do (and what we take from each)

| App | What it does well | What we take |
|---|---|---|
| macOS Preview | Thumbnail sidebar, drag to reorder, drag another PDF in to merge, delete, rotate. No export dialog. | A plain, forgiving page grid; dropping a file in just works. |
| Adobe Acrobat "Organize pages" | Hover tools on each page (rotate, delete, insert, extract), insert pages from another file at a position. | Hover actions on a page, "insert here" targets between pages. |
| iLovePDF / Smallpdf | Merge starts as a list of file cards you reorder, then optionally a page view. One clear primary button. | Treat merge as "add files, order them, save". One primary button. |
| PDF Arranger (open source) | Grid of pages, drag and drop, rotate, crop, import. Fast. | Same model; lazy thumbnails for big files. |
| PDFsam | Separate modules per task, page ranges as text ("1-3,5,8-"). | Optional range box for power users (split, extract). |

The common pattern is two levels: files first, pages second. We combine them into **one window**: a file rail on the left and a single page grid, where each file's pages carry a thin colour tag. Merging is then just "add another file"; reordering files reorders their blocks; page-level changes are the same grid. There is no separate "merge wizard".

## 2. How it appears in the product

### The ring (drop on a format)

- A PDF shows format wedges (PNG, JPG, TXT, DOCX) plus one **tool wedge: "Organize"**. Tool wedges look different from format wedges (outlined, pencil or pages icon) so nobody expects a file out of them.
- **Dropping on the centre hub** opens the same editor ("open in workspace"). The hub does nothing today, so this costs no layout.
- Hover sub-menus on wedges are **not** planned: they are fragile while a drag is in flight and hard to hit. Options live in the editor instead (see "Options before converting").

### Several files selected

| Selection | Ring shows |
|---|---|
| One PDF | PNG, JPG, TXT, DOCX, **Organize** |
| Several PDFs | **Merge**, **Organize all** (opens the editor with every file loaded in selection order), plus the format wedges applied to each file |
| Several images | **Combine into PDF** (new), plus the usual image conversions applied to each |
| Mixed, or files with no common action (for example a PDF and a video) | **No ring.** Windows and macOS drag stays untouched. |
| Unsupported files | No ring (as today) |

Two merge speeds: **Merge** is the fast path (current order, no window, result next to the first file); **Organize all** is the careful path. Both end with the same naming rule as today (never overwrite: `name (merged).pdf`, then `(1)`).

### Options before converting

Some conversions have real choices (image resolution, JPG quality, page range, which pages). Plan:
- Defaults are good enough that dropping on a wedge still converts at once.
- The editor window has an **Export** panel (format, pages, resolution, quality) for people who want control.
- A setting "Ask before converting" shows that same panel as a small sheet on every drop.

## 3. The page editor window

See the wireframe in the chat for this plan (file rail, page grid, toolbar, bottom bar).

**Layout**
- Toolbar: Add files, Undo, Redo, Rotate left and right, Duplicate, Delete, Extract selection, thumbnail size slider.
- File rail: each source file with page count, a colour tag, drag to reorder, remove (x). Drop more PDFs or images onto it to add them.
- Page grid: virtualized (only visible thumbnails are drawn), drag to reorder with a clear insertion line, click / Shift-click / Ctrl-click and rubber-band selection, per-page hover actions, page number under each thumbnail.
- Bottom bar: "9 pages from 2 files, 2 selected", the output name, and one primary **Save PDF** button. Saving never touches the originals.
- Large preview: Space (or double-click) opens one page big, arrows move between pages.

**Behaviour**
- Non-destructive: the editor edits a list of page references (file, page, rotation), not the PDFs. Thumbnails come from the originals; the new PDF is written only on Save. Undo and redo are cheap because each step is a new list.
- Keyboard first: Delete, Ctrl+Z / Ctrl+Y, R to rotate, Ctrl+A, arrows, Space.
- Everything respects the chosen theme and follows the flat, quiet look of the manager window.

## 4. Features worth adding that you did not list

Must have for a good first version:
1. **Images to PDF** (combine) and **PDF to images**.
2. **Rotate**, **delete**, **duplicate** pages.
3. **Extract selected pages** into a new PDF, and **split** (every N pages, or by ranges like `1-3,5,8-`).
4. **Insert a blank page**, and insert pages from another file at an exact position.
5. **Undo and redo** and a **cancel** button on long jobs, with progress ("page 12 of 80").
6. **Password-protected PDFs**: ask for the password instead of failing.
7. **Batch conversion** with one progress indicator ("3 of 8 files").
8. A real **error message** for damaged files that says which file and why.

Good next steps (after v1):
9. **Compress PDF** (smaller file size).
10. **Protect and unlock** with a password.
11. **Remove blank pages** (detect them automatically).
12. **Page numbers, watermark, crop**.
13. **OCR** for scanned PDFs (makes them searchable and convertible to text); a large optional pack.
14. **Edit title, author** metadata.
15. Normalize page size when merging (fit everything to A4 or Letter).

Honest limits to show in the UI:
- **PDF to Word/ODT is best effort.** It reads text and rough layout; complex layouts, tables and columns can come out different. We label it "Editable text, layout may differ" so nobody is surprised. It needs the Documents pack (LibreOffice).
- Merging keeps page content and annotations but **bookmarks and document-level scripts are not carried over**.
- Scanned PDFs have no text until OCR exists (item 13).

## 5. Technical approach

### One PDF engine: PDFium

PDFium (the PDF library from Chrome) via the `pdfium-render` Rust crate. It renders pages, reads text, imports pages from other PDFs, rotates, deletes and writes PDFs: everything above, in one engine, from Rust.

| Option | For | Against |
|---|---|---|
| **PDFium (recommended)** | One library for rendering, conversion and editing; works in our existing Rust job pipeline; robust on odd PDFs; prebuilt binaries for Windows and macOS (BSD-style license). | A native library to ship (about 5 to 10 MB per platform); macOS needs it signed inside the app. |
| pdf.js + pdf-lib in the page | No native library. | Two stacks; weaker on encrypted or damaged files; conversions would need the web view, which our headless job pipeline does not have. |
| qpdf / Ghostscript / poppler | Mature command-line tools. | Extra programs per platform; no rendering for the editor. Reserved for later (compress, protect). |

It fits the modular design we already have: a **"PDF tools" pack** whose engine is `pdfium`, downloaded once like the other converters, with a checksum in `engines.manifest.json` and listed in the manager window.

### Formats

| From | To | How |
|---|---|---|
| PDF | PNG, JPG | PDFium renders each page (one file, or a folder `name pages/` for several) |
| PDF | TXT | PDFium text extraction |
| PDF | DOCX, ODT, HTML | LibreOffice's PDF import, then its normal export (best effort) |
| Images | PDF | PDFium page creation from the images |
| PDFs | PDF (merge, reorder, split, extract) | PDFium page import and save |

### Where the code goes (one concern per file, UI as `.tsx` plus `.css`)

- `crates/radial-packs`: catalog entry for the `pdfium` engine and the `pdf` pack, manifest rows (all three platforms, checksums).
- `app/src-tauri/src/engine/engines/pdfium.rs`: render, text, import and save; a small Rust module `pdf_plan.rs` that holds the page-list model and is fully unit-tested without any window.
- Commands: open files (returns page counts, never the whole file), render a thumbnail (binary reply, about 15 KB each), save a plan to a new PDF, cancel.
- `app/src/workspace/`: `Workspace.tsx` (container), `PageGrid.tsx`, `FileRail.tsx`, `Toolbar.tsx`, `BottomBar.tsx`, `ExportPanel.tsx`, `planModel.ts` (pure, undo and redo), each with its own CSS; a browser demo route (`#workspace-demo`) with mock thumbnails so the design is judged **in the browser before any rebuild**, as the project rules require.
- Lazy thumbnails use `IntersectionObserver` and animation frames, never `setTimeout`.
- The hook and ring: selection becomes a list (it only reads one file's type today); a small pure function decides which wedges a given selection gets (the table in section 2), with tests.

### Windows and macOS details

- The PDFium library lives in the engine folder like the other engines. On macOS it must be code-signed together with the app for notarization; on Windows the DLL sits beside the program that loads it.
- Hold the whole plan in memory only as references; render on demand and keep a small thumbnail cache on disk, so a 500-page PDF does not need 500 images in memory.

## 6. Delivery in phases

| Phase | What ships | Checkpoint |
|---|---|---|
| 0. Spike (1 to 2 days) | Download PDFium as a pack, load it from Rust on Windows and macOS, render one page to a PNG. | Size, load time and signing on macOS are acceptable; otherwise reconsider the pdf.js route. |
| 1. Conversions | PDF to PNG, JPG, TXT; images to PDF; batch progress; PDF to DOCX through LibreOffice. | Convert real files on both OSes. |
| 2. Smart ring | Multi-file selection, the table of wedges, "no ring for unsupported mixes", fast **Merge**. | Select PDFs and images in every combination. |
| 3. Editor v1 | Single-file page grid, thumbnails, drag reorder, delete, rotate, undo and redo, Save. | Reorder a 100-page PDF; originals untouched. |
| 4. Multi-file editor | File rail, add by drop, colour tags, extract selection, split, blank page, insert at position, range box. | Merge three PDFs and fix the order by page. |
| 5. Polish | Large preview, password prompt, "Ask before converting", themes, keyboard shortcuts, accessibility. | Used without a mouse; all seven themes. |
| 6. Later | Compress, protect, remove blank pages, OCR pack, metadata. | Each is its own small pack or feature. |

## 7. Testing

- Pure logic (`pdf_plan.rs`, `planModel.ts`, wedge rules): unit tests, no PDFs needed.
- Real PDFs: generate small fixtures with PDFium in the tests, then check page count, order and rotation of the output, and the text on each page, after merge, reorder, delete and rotate.
- Edge cases: encrypted, zero pages, very large, damaged, mixed page sizes, landscape pages.
- UI: preview every state (empty, loading, 500 pages, error, password) in the browser first.

## 8. Decisions needed

1. **PDFium as the single engine** (recommended), or pdf.js + pdf-lib?
2. **Ring**: an "Organize" wedge plus drop on the hub, as above?
3. **Scope of the first release**: phases 0 to 3 (conversions plus a single-file editor), or include the multi-file editor (phase 4) before releasing?
4. **Output**: always next to the first source file with a new name (as now), or a "Save as" dialog for the editor?
5. **PDF to Word** with its honest "layout may differ" label, or leave it out until quality is better?
