# Releasing Radial Convert

What gets built, what you must set up once, and the steps for each release.

## What a release contains

| File | What it is | Built by |
|---|---|---|
| `Radial-Convert-macos-universal.zip` | the app (Apple Silicon and Intel in one), signed and notarized | `release.yml`, job `app-macos` |
| `Radial-Convert-macos-universal.dmg` | same app, for people who drag it to Applications | `app-macos` |
| `Radial-Convert-windows-x86_64-setup.exe` | per-user Windows installer | `app-windows` |
| `radial-macos-aarch64`, `radial-macos-x86_64`, `radial-windows-x86_64.exe` | the `radial` command (picks and installs converters) | `cli` |
| `install.sh`, `install.ps1` | one-line installers, already pointed at this repo | `release` job |
| `SHA256SUMS` | checksums of everything above; the install scripts verify against it | `release` job |

The converter programs (ffmpeg, pandoc, LibreOffice) are **not** in the release. `radial install` downloads them from a separate **engines mirror** release (below), checked against the checksums in `crates/radial-packs/engines.manifest.json`.

Users install with:

```
macOS    curl -fsSL https://github.com/OWNER/REPO/releases/latest/download/install.sh | sh
Windows  irm https://github.com/OWNER/REPO/releases/latest/download/install.ps1 | iex
```

## One-time setup (needs you)

These need your accounts and secrets, so they are not automated.

1. **Repository and license: done.** The code is at github.com/Saransh1220/FileOrbit under the MIT license (`LICENSE`); the converters keep their own licenses, listed in `THIRD_PARTY_NOTICES.md`.
2. **App icon: done.** `scripts/make_icon.py` writes `app/src-tauri/icons/source.svg` (the dial on a graphite tile) and `npm run tauri icon src-tauri/icons/source.svg` (inside `app/`) produces every size. Replace it with a designed icon whenever you have one.
3. **Apple signing and notarization** (macOS users otherwise see "cannot be opened because the developer cannot be verified" if they download the dmg in a browser). Needs the Apple Developer Program (paid, per year).
   1. In your Apple Developer account, create a **Developer ID Application** certificate and export it from Keychain Access as a `.p12` file with a password.
   2. Create an **app-specific password** at appleid.apple.com (used only for notarization). Note your **Team ID**.
   3. Add these repository secrets (Settings, Secrets and variables, Actions):

      | Secret | Value |
      |---|---|
      | `APPLE_CERTIFICATE` | the `.p12` file, base64-encoded: `base64 -i cert.p12 \| pbcopy` |
      | `APPLE_CERTIFICATE_PASSWORD` | the `.p12` password |
      | `APPLE_SIGNING_IDENTITY` | e.g. `Developer ID Application: Your Name (TEAMID)` |
      | `APPLE_ID` | your Apple ID email |
      | `APPLE_PASSWORD` | the app-specific password |
      | `APPLE_TEAM_ID` | your Team ID |

   Without these the macOS build still succeeds but is **unsigned**, and the workflow prints a warning.
4. **Windows signing (optional, later).** Unsigned installers work but show a Microsoft SmartScreen warning until the file builds reputation. Options: a code-signing certificate, or Azure Trusted Signing. Not set up yet.

## Mirror the converter downloads (before the first release, and when versions change)

Upstream hosts delete and move files, so copy each pinned download to a release we control.

```
# 1. Download and verify everything; writes crates/radial-packs/engines.manifest.mirrored.json. Uploads nothing.
python3 scripts/mirror_artifacts.py --repo OWNER/REPO --tag engines-2026-10

# 2. Upload to a GitHub release (needs `gh` logged in with write access).
python3 scripts/mirror_artifacts.py --repo OWNER/REPO --tag engines-2026-10 --publish --in-place
```

Then commit the rewritten manifest (only the URLs change; the checksums stay the same, which is the proof the copies are identical). Check it with a real download on each platform:

```
cd crates/radial-packs && cargo test --test real_downloads -- --ignored --nocapture
```

The mirror release is marked "not latest" so it never shadows the app release. It includes the licence notices and source links; if you host copies of GPL programs, keep those links reachable.

## Cutting a release

1. Set the same new version in `app/src-tauri/tauri.conf.json`, `app/src-tauri/Cargo.toml`, `app/package.json` and `crates/radial-packs/Cargo.toml`. Commit.
2. Tag and push: `git tag v0.1.0 && git push origin v0.1.0`. The workflow fails fast if the tag does not match `tauri.conf.json`.
3. Wait for the **release** workflow. It creates a **draft** release.
4. Review the draft: all files present, macOS job log shows the signing authority and notarization, `SHA256SUMS` lists everything.
5. **Smoke test on a clean machine** before publishing (see below).
6. Click **Publish release**. The one-line installers now work, because `/releases/latest` points at it.

### Betas

Tag with a suffix: `git tag v0.1.0-beta.1 && git push origin v0.1.0-beta.1`. The part before the `-` must equal the app version. A beta builds everything like a normal release, but the **draft** is marked as a **pre-release**, titled "(beta)", with `docs/BETA_NOTES.md` as its text, so `/releases/latest` and the one-line installers stay untouched. Testers download the files from the Releases page. A tag cannot be reused for a second try: delete the draft and the tag, or use the next number (`-beta.2`). Publish from the draft page after checking the files and `SHA256SUMS`.

Without Apple secrets the macOS app is **unsigned** (see "One-time setup"); the beta notes tell testers how to open it.

To try the builds without releasing, run the workflow by hand from the Actions tab: it builds everything and stops before creating a release.

## Smoke test on a clean machine

A developer machine hides missing-pack problems because Homebrew's tools are on it. To test like a new user:

- Use a Mac or Windows PC that has never had the app, or run with `RADIAL_NO_SYSTEM_TOOLS=1` and an empty `RADIAL_HOME` (see `MODULAR_INSTALL_PLAN.md`).
- Run the one-line installer, pick one pack, drag a file onto a format. Then drag a file that needs a different pack and confirm the "Install ...?" toast appears and works.

## Building locally

`npm run build:local` (inside `app/`) makes a debug `.app` signed with the local self-signed certificate "Radial Convert Dev" (config in `app/src-tauri/tauri.local-signing.conf.json`). That certificate is for your machine only: it keeps macOS's Input Monitoring permission valid across rebuilds. It is not used for releases.

## What has and has not been verified

| Piece | Status |
|---|---|
| CI (`ci.yml`) | Runs on every push to GitHub. It failed on every push until the converter source folder (`app/src-tauri/src/engine/engines/`) was committed: a `.gitignore` rule had been hiding it. A fresh clone is now built and tested to prove it. |
| `install.sh` | Tested against a local fake release: success, tampered CLI, tampered app (nothing installed), missing checksum entry, `--no-app`, `--packs`, replacing an existing app, unreachable server. Not yet run against a real GitHub release or in a real terminal pipe (`curl \| sh` with the interactive picker). |
| `install.ps1` | Tested on Windows 11 against a fake release (`scripts/test_install_ps1.py`): normal install, tampered CLI, tampered app, file missing from the checksums, the placeholder address, an unreachable server. Not yet run against a real GitHub release. |
| `mirror_artifacts.py` | Tested with local files and a fake `gh` (checksum mismatch stops everything, release created as not-latest, upload replaces same-named files). Not yet run against the real upstream files or a real repo. |
| `release.yml` | Valid YAML and consistent job wiring, **never run on GitHub** (it runs on a version tag or by hand). Expect to fix small things on the first real run (runner image names, bundle paths). |
| Windows installer (NSIS) | Built and installed on a Windows 11 PC. Building it needs a download of the NSIS tool from GitHub; on a slow connection Tauri's built-in timeout can fail it, and running the build again usually works. |
| msi (LibreOffice) unpack | Works on Windows, including an install folder with a space in its name. |
| Notarization | Not exercised: needs your Apple credentials. |
| macOS build | Built by the original author on a Mac earlier; **not rebuilt since the Windows work, the manager window, the themes and the download cache.** |
