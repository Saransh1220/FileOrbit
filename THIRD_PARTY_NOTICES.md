# Third-party notices

Radial Convert does not contain these programs. When you install a pack with `radial install`, the installer downloads the program from its publisher, checks its SHA-256 checksum, and runs it as a **separate program** to do the conversion. Each program keeps its own license, listed below.

| Program | Used for | License | Source code |
|---|---|---|---|
| ffmpeg (macOS builds by Martin Riedl) | audio, video, images | GNU GPL v2 or later (these builds include x264 and x265) | https://ffmpeg.org/download.html and https://ffmpeg.martin-riedl.de/ (build scripts) |
| ffmpeg (Windows builds by BtbN) | audio, video, images | GNU LGPL v3 (built without x264/x265) | https://ffmpeg.org/download.html and https://github.com/BtbN/FFmpeg-Builds |
| pandoc | Markdown, HTML, EPUB, TeX, plain text | GNU GPL v2 or later | https://github.com/jgm/pandoc |
| LibreOffice | Word, Excel, PowerPoint to PDF and each other | Mozilla Public License 2.0 (plus bundled components) | https://www.libreoffice.org/download/source-code/ |
| PDFium (prebuilt by bblanchon/pdfium-binaries) | reading, rendering and editing PDFs | BSD-style and Apache 2.0 for PDFium itself, with the licenses of its third-party parts shipped in the download; the packaging is MIT | https://pdfium.googlesource.com/pdfium/ and https://github.com/bblanchon/pdfium-binaries |

## Notes for whoever redistributes this project

- The exact versions and download addresses we pin are in `crates/radial-packs/engines.manifest.json`.
- If you host copies of these programs yourself (a mirror or a release bundle), you are redistributing them. Include the license texts of the programs you host (GPL for the macOS ffmpeg and pandoc, LGPL for the Windows ffmpeg), and keep the source links above reachable, or host the matching source yourself.
- Radial Convert's own code is MIT-licensed (`LICENSE`).
- PDFium is a library, not a separate program: the app loads it from the folder the "PDF tools" pack installs it in.
- ffmpeg is a trademark of Fabrice Bellard; pandoc, LibreOffice and the other names belong to their owners.
