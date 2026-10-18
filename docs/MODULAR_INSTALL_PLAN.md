# Modular install plan

Goal: a user installs Radial Convert and then chooses which converters to add. Nothing is required up front, nothing needs Homebrew/winget, and macOS and Windows work the same way.

## 1. Concepts

**Engine**: an external program that does the conversion (ffmpeg, pandoc, LibreOffice). Large, per-OS, versioned.

**Pack**: what the user picks. A pack is a feature the user understands ("Audio", "Documents") and lists the engines it needs. Several packs can share an engine, which is installed once.

| Pack | Converts | Engine(s) |
|---|---|---|
| `documents` | Word, Excel, PowerPoint, RTF to PDF and to each other | LibreOffice |
| `text` | Markdown, HTML, EPUB, TeX, plain text, docx to markdown | pandoc |
| `audio` | MP3, WAV, FLAC, AAC, M4A, OGG | ffmpeg |
| `video` | MP4, MOV, MKV, WEBM, GIF, extract audio | ffmpeg |
| `images` | PNG, JPG, WEBP, TIFF, BMP, GIF (HEIC on macOS uses the built-in `sips`) | ffmpeg |

Picking `audio` + `images` downloads ffmpeg once. Future packs (`pdf-tools`: merge/split/reorder, `archives`, `ocr`) are added to the catalog without changing the installer.

## 2. The `radial` CLI

```
radial install                 interactive multi-select of packs
radial install --packs audio,images --yes
radial install --all
radial list                    packs, what they need, installed or not
radial remove <pack>...        removes a pack; an engine no pack needs any more is deleted
radial doctor                  runs every installed engine and reports what works
radial where                   prints the install folder
```

Interactive flow: checklist (space toggles, enter confirms) → summary of what will be downloaded and how big → confirm → progress bars → "Installed: audio, images". Already-installed packs are shown and skipped.

## 3. Where things live

| | macOS | Windows |
|---|---|---|
| Install home | `~/Library/Application Support/Radial Convert/` | `%LOCALAPPDATA%\Radial Convert\` |
| Engines | `engines/<engine>/<version>/` | same |
| State | `installed.json` (packs + engine paths and versions) | same |

`RADIAL_HOME` overrides the folder (tests, portable installs, enterprise mirrors).

The app resolves an engine in this order: **managed install** (`installed.json`) → system PATH and well-known folders (developer fallback). If a pack's engine is missing, the toast says which pack to install.

## 4. Engine manifest

`engines.manifest.json` ships inside the CLI. Per engine and per platform key (`macos-aarch64`, `macos-x86_64`, `windows-x86_64`):

```
url, sha256, archive (zip | tar-gz | raw | dmg | msi), bin (path to the executable inside), size_bytes
```

Every download is checked against its SHA-256 before anything is extracted. Extraction goes to a `.partial` folder that is renamed only on success, so a killed install never leaves a half-working engine.

Policy when an engine has a published artifact: always use the downloaded, pinned copy (reproducible). Only when none is published for the platform does the CLI adopt a tool already on the machine; this is the developer path and is labelled "system" in `radial list`.

## 5. Engine sources and licensing

Status of the pinned artifacts (all downloaded and checksum-verified; see `tests/real_downloads.rs`):

| Engine | macOS arm64 | macOS x86_64 | Windows x86_64 |
|---|---|---|---|
| pandoc 3.12.1 | official zip, pinned | official zip, pinned | official zip, pinned |
| ffmpeg 9.0.2 | Martin Riedl build (GPL), pinned | Martin Riedl build (GPL), pinned | BtbN `win64-lgpl-shared`, pinned |
| LibreOffice 26.8.1 | TDF dmg, pinned **and installed for real** | TDF dmg, pinned (checksum from TDF, not downloaded) | TDF msi, pinned (downloaded and hashed; unpacking needs Windows) |

Decisions:

- **The project is open source and not for sale, so GPL builds are acceptable.** The macOS ffmpeg builds include x264/x265 (GPL); the Windows build is LGPL. `THIRD_PARTY_NOTICES.md` lists every program, its license and its source. A GPL-only conclusion would change if the project ever became closed-source or commercial: then macOS needs an LGPL ffmpeg built from source.
- **Checked on the downloaded files, not assumed:** the macOS arm64 build runs natively and the Intel build under Rosetta; both include the MP3, Opus, Vorbis, WebP and VP9 encoders plus Apple's hardware H.264 encoder. The Windows build's configuration has `--disable-libx264 --disable-libx265` and no `--enable-gpl`. (Windows executables were unpacked and located, not run.)
- **Pinned URLs must be ours before release.** BtbN deletes old `autobuild-*` tags and Martin Riedl's site may reorganise, so a pinned URL can stop working. Mirror each artifact to our own release hosting and change only the `url` fields; the checksums stay the same. `cargo test --test real_downloads -- --ignored` catches a moved or deleted file.
- **pandoc is GPL-2.0-or-later.** We run it as a separate program, which is allowed. If we host copies, include its license text and keep the source link reachable.
- **LibreOffice** (MPL): about 285-375 MB to download and about 805 MB on disk once unpacked, so it is the heavy pack. macOS: the dmg is mounted read-only, `LibreOffice.app` is copied out with `ditto` (keeps symlinks and code signatures) and the image is always unmounted. Windows: `msiexec /a` unpacks the msi without installing anything. TDF publishes SHA-256 files next to each download. **Verified on this Mac:** download, checksum, install, `soffice --version`, and a real RTF to PDF through the app with the system copy hidden. **Not verified:** the Windows unpack, because `msiexec` only exists on Windows. The msi's file list (via msitools) shows `LibreOffice/program/soffice.exe`; since `msiexec /a` can nest that under another folder name, the installer falls back to finding a *unique* file of that name. `cargo test --test real_downloads -- --ignored` on a Windows machine (or the CI job) is the check.
- **TDF paths rot.** `stable/26.8.1` moves to an archive path once a newer release ships, so mirror the three files before release, like the others.

## 6. Getting the CLI itself

One-line bootstrap per OS that downloads the `radial` binary (and later the app) and puts it on PATH:

- macOS/Linux: `curl -fsSL <host>/install.sh | sh`
- Windows: `irm <host>/install.ps1 | iex`

The scripts are a later phase; the CLI works today with `cargo run`.

## 6b. Testing like a clean machine

A developer machine usually has Homebrew's ffmpeg and friends, which hides every "pack missing" path. Two environment variables fix that for testing:

- `RADIAL_NO_SYSTEM_TOOLS=1` makes the app and CLI ignore tools already on the machine.
- `RADIAL_HOME=/some/empty/folder` uses a throwaway install folder.
- `RADIAL_MANIFEST=/path/to/manifest.json` replaces the built-in manifest, e.g. to install from local `file://` copies, a company mirror, or offline.

macOS: `open --env RADIAL_HOME=/tmp/x --env RADIAL_NO_SYSTEM_TOOLS=1 "Radial Convert.app"`. Then drag a file that needs a pack; the toast offers to install it. (Launching with plain `open` afterwards starts a normal instance.)

## 7. Phases

1. **(done in this slice)** crate with catalog, manifest, state, download/verify/extract, install/remove/doctor, `radial` CLI, app resolves managed engines first. Tested with local fixture artifacts.
2. **(done)** Pinned and downloaded for real: pandoc and ffmpeg on all three platforms. **Open:** hosting mirror.
3. **(done on macOS)** LibreOffice extraction (`dmg`, `msi`). **Open:** confirm the msi unpack on Windows.
4. Bootstrap scripts, release builds, signing/notarization, GitHub Actions for macOS and Windows.
5. **(done)** In-app prompt: a missing pack shows "Install <pack>?" in the toast, downloads with progress using the same crate, then finishes the waiting conversion.
6. Windows hook and overlay (separate from this plan).
