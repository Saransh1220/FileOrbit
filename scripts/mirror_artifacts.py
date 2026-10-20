#!/usr/bin/env python3
"""Re-host the pinned engine downloads on our own GitHub release.

Upstream hosts move and delete files (BtbN prunes old builds, TDF archives old
versions), so before a release every artifact in the manifest is copied to a
release we control. The copies must be byte-identical: each file is checked
against the manifest's SHA-256 first, and the checksums never change, only the
URLs do.

    # 1. Download and verify everything; write a manifest that points at the mirror.
    python3 scripts/mirror_artifacts.py --repo OWNER/REPO --tag engines-2026-10

    # 2. Same, then upload to the GitHub release (needs `gh` logged in).
    python3 scripts/mirror_artifacts.py --repo OWNER/REPO --tag engines-2026-10 --publish

Without --publish nothing leaves this machine except the downloads.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import subprocess
import sys
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MANIFEST = ROOT / "crates/radial-packs/engines.manifest.json"
NOTICES = ROOT / "THIRD_PARTY_NOTICES.md"
CHUNK = 1024 * 1024


def asset_name(engine: str, version: str, platform: str, url: str) -> str:
    """Stable file name on the mirror, e.g. ffmpeg-9.0.2-macos-aarch64.zip."""
    base = url.split("?")[0].rsplit("/", 1)[-1]
    ext = ".tar.gz" if base.endswith(".tar.gz") else Path(base).suffix
    return f"{engine}-{version}-{platform}{ext}"


def sha256_of(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        while chunk := handle.read(CHUNK):
            digest.update(chunk)
    return digest.hexdigest()


def download(url: str, dest: Path) -> None:
    dest.parent.mkdir(parents=True, exist_ok=True)
    part = dest.with_suffix(dest.suffix + ".part")
    with urllib.request.urlopen(url) as response, part.open("wb") as out:
        shutil.copyfileobj(response, out, CHUNK)
    part.replace(dest)


def fetch_verified(url: str, expected: str, dest: Path) -> None:
    """Make `dest` hold the file at `url`, reusing a cached copy only if its checksum matches."""
    if dest.exists() and sha256_of(dest).lower() == expected.lower():
        print(f"  cached   {dest.name}")
        return
    print(f"  fetching {url}")
    download(url, dest)
    actual = sha256_of(dest)
    if actual.lower() != expected.lower():
        dest.unlink(missing_ok=True)
        raise SystemExit(
            f"checksum mismatch for {url}\n  expected {expected}\n  got      {actual}\n"
            "The file changed upstream or the manifest is wrong; nothing was mirrored."
        )


def rewrite_manifest(manifest: dict, repo: str, tag: str) -> dict:
    """Copy of the manifest with every URL pointing at the mirror (checksums untouched)."""
    out = json.loads(json.dumps(manifest))
    for engine, release in out["engines"].items():
        for platform, artifact in release["platforms"].items():
            name = asset_name(engine, release["version"], platform, artifact["url"])
            artifact["url"] = f"https://github.com/{repo}/releases/download/{tag}/{name}"
    return out


def release_notes(tag: str) -> str:
    notices = NOTICES.read_text() if NOTICES.exists() else ""
    return (
        f"Mirrored converter programs for Radial Convert ({tag}).\n\n"
        "These are unmodified copies of the upstream downloads, verified by SHA-256 "
        "before upload. They exist so installs keep working if an upstream host "
        "moves or removes a file. Each program keeps its own license; the source "
        "code for each is available from the links below.\n\n" + notices
    )


def gh(args: list[str]) -> subprocess.CompletedProcess:
    # The full path `which` found, not just "gh": Windows only finds .exe files by bare name.
    return subprocess.run([shutil.which("gh") or "gh", *args], capture_output=True, text=True)


def publish(repo: str, tag: str, files: list[Path], notes_file: Path) -> None:
    if shutil.which("gh") is None:
        raise SystemExit("--publish needs the GitHub CLI (`gh`), logged in with access to the repo")
    if gh(["release", "view", tag, "--repo", repo]).returncode == 0:
        print(f"release {tag} exists; uploading (replacing same-named files)")
        result = gh(["release", "upload", tag, *map(str, files), "--repo", repo, "--clobber"])
    else:
        print(f"creating release {tag}")
        # --latest=false: the app's bootstrap scripts use /releases/latest, which must stay the app release.
        result = gh([
            "release", "create", tag, *map(str, files), "--repo", repo,
            "--title", f"Engines {tag}", "--notes-file", str(notes_file), "--latest=false",
        ])
    if result.returncode != 0:
        raise SystemExit(f"gh failed:\n{result.stderr.strip()}")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--repo", required=True, help="GitHub repo that will host the mirror, OWNER/REPO")
    parser.add_argument("--tag", required=True, help="release tag for the mirror, e.g. engines-2026-10")
    parser.add_argument("--manifest", type=Path, default=MANIFEST)
    parser.add_argument("--cache", type=Path, default=ROOT / ".mirror-cache")
    parser.add_argument("--out", type=Path, default=None,
                        help="where to write the rewritten manifest (default: next to the input, .mirrored.json)")
    parser.add_argument("--in-place", action="store_true", help="overwrite the input manifest instead")
    parser.add_argument("--publish", action="store_true", help="upload to the GitHub release (otherwise download + verify only)")
    args = parser.parse_args(argv)

    manifest = json.loads(args.manifest.read_text())
    files: list[Path] = []
    for engine, release in manifest["engines"].items():
        print(f"{engine} {release['version']}")
        for platform, artifact in release["platforms"].items():
            dest = args.cache / asset_name(engine, release["version"], platform, artifact["url"])
            fetch_verified(artifact["url"], artifact["sha256"], dest)
            files.append(dest)

    rewritten = rewrite_manifest(manifest, args.repo, args.tag)
    out = args.manifest if args.in_place else (args.out or args.manifest.with_suffix(".mirrored.json"))
    out.write_text(json.dumps(rewritten, indent=2) + "\n")
    print(f"\nverified {len(files)} files; wrote {out}")

    if args.publish:
        notes = args.cache / "RELEASE_NOTES.md"
        notes.write_text(release_notes(args.tag))
        publish(args.repo, args.tag, files, notes)
        print(f"published to https://github.com/{args.repo}/releases/tag/{args.tag}")
    else:
        print("dry run: nothing was uploaded (add --publish to upload)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
