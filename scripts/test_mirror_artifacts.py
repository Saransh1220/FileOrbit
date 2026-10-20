"""Tests for mirror_artifacts.py using local files and a fake `gh`; no network, no GitHub."""

import hashlib
import json
import os
import stat
import tempfile
import unittest
from pathlib import Path

import mirror_artifacts as m


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


class Fixture:
    """A manifest whose URLs are local files."""

    def __init__(self, tmp: Path, corrupt: bool = False):
        self.tmp = tmp
        self.blobs = {"tool.zip": b"zip-bytes", "big.tar.gz": b"targz-bytes", "app.dmg": b"dmg-bytes"}
        for name, data in self.blobs.items():
            (tmp / name).write_bytes(data)
        wrong = "0" * 64
        self.manifest = {
            "version": 1,
            "engines": {
                "ffmpeg": {
                    "version": "9.0.2",
                    "platforms": {
                        "macos-aarch64": {"url": (tmp / "tool.zip").as_uri(), "sha256": wrong if corrupt else sha(self.blobs["tool.zip"]),
                                          "archive": "zip", "bin": "ffmpeg", "size_bytes": 9},
                        "windows-x86_64": {"url": (tmp / "big.tar.gz").as_uri() + "?download=1", "sha256": sha(self.blobs["big.tar.gz"]),
                                           "archive": "tar-gz", "bin": "ffmpeg.exe", "size_bytes": 11},
                    },
                },
                "libreoffice": {
                    "version": "26.8.1",
                    "platforms": {
                        "macos-aarch64": {"url": (tmp / "app.dmg").as_uri(), "sha256": sha(self.blobs["app.dmg"]),
                                          "archive": "dmg", "bin": "LibreOffice.app/Contents/MacOS/soffice", "size_bytes": 9},
                    },
                },
            },
        }
        self.path = tmp / "manifest.json"
        self.path.write_text(json.dumps(self.manifest))

    def run(self, *extra: str) -> int:
        return m.main(["--repo", "me/radial", "--tag", "engines-test", "--manifest", str(self.path),
                       "--cache", str(self.tmp / "cache"), *extra])


class AssetNames(unittest.TestCase):
    def test_names_are_stable_and_keep_compound_extensions(self):
        self.assertEqual(m.asset_name("ffmpeg", "9.0.2", "macos-aarch64", "https://x/y/ffmpeg.zip"), "ffmpeg-9.0.2-macos-aarch64.zip")
        self.assertEqual(m.asset_name("ffmpeg", "9", "linux-x86_64", "https://x/a.tar.gz?token=1"), "ffmpeg-9-linux-x86_64.tar.gz")
        self.assertEqual(m.asset_name("libreoffice", "26.8.1", "windows-x86_64", "https://x/LO_Win.msi"), "libreoffice-26.8.1-windows-x86_64.msi")


class Mirroring(unittest.TestCase):
    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.tmp = Path(self._tmp.name)

    def tearDown(self):
        self._tmp.cleanup()

    def test_rewrites_urls_but_never_checksums(self):
        f = Fixture(self.tmp)
        self.assertEqual(f.run(), 0)
        out = json.loads(f.path.with_suffix(".mirrored.json").read_text())
        mac = out["engines"]["ffmpeg"]["platforms"]["macos-aarch64"]
        self.assertEqual(mac["url"], "https://github.com/me/radial/releases/download/engines-test/ffmpeg-9.0.2-macos-aarch64.zip")
        self.assertEqual(mac["sha256"], f.manifest["engines"]["ffmpeg"]["platforms"]["macos-aarch64"]["sha256"])
        self.assertEqual(out["engines"]["libreoffice"]["platforms"]["macos-aarch64"]["bin"], "LibreOffice.app/Contents/MacOS/soffice")

    def test_a_checksum_mismatch_stops_everything(self):
        f = Fixture(self.tmp, corrupt=True)
        with self.assertRaises(SystemExit) as caught:
            f.run()
        self.assertIn("checksum mismatch", str(caught.exception))
        self.assertFalse(f.path.with_suffix(".mirrored.json").exists(), "no manifest may be written")
        self.assertFalse((self.tmp / "cache/ffmpeg-9.0.2-macos-aarch64.zip").exists(), "the bad file is not kept")

    def test_a_cached_file_is_reused_only_if_its_hash_matches(self):
        f = Fixture(self.tmp)
        f.run()
        cached = self.tmp / "cache/ffmpeg-9.0.2-macos-aarch64.zip"
        cached.write_bytes(b"tampered")
        f.run()  # re-fetches because the cache no longer matches
        self.assertEqual(cached.read_bytes(), f.blobs["tool.zip"])

    def test_in_place_overwrites_the_input_manifest(self):
        f = Fixture(self.tmp)
        f.run("--in-place")
        self.assertIn("github.com/me/radial", f.path.read_text())

    def test_dry_run_never_calls_gh(self):
        f = Fixture(self.tmp)
        log = self.tmp / "gh.log"
        self.install_fake_gh(log)
        f.run()
        self.assertFalse(log.exists(), "gh must not run without --publish")

    def install_fake_gh(self, log: Path, release_exists: bool = False) -> None:
        bindir = self.tmp / "bin"
        bindir.mkdir(exist_ok=True)
        view_rc = 0 if release_exists else 1
        if os.name == "nt":
            gh = bindir / "gh.cmd"
            gh.write_text(f'@echo off\r\necho %* >> "{log}"\r\nif "%1 %2"=="release view" exit /b {view_rc}\r\nexit /b 0\r\n')
        else:
            gh = bindir / "gh"
            gh.write_text(f'#!/bin/sh\necho "$@" >> "{log}"\n[ "$1 $2" = "release view" ] && exit {view_rc}\nexit 0\n')
            gh.chmod(gh.stat().st_mode | stat.S_IEXEC)
        self._old_path = os.environ["PATH"]
        os.environ["PATH"] = f"{bindir}{os.pathsep}{self._old_path}"
        self.addCleanup(lambda: os.environ.__setitem__("PATH", self._old_path))

    def test_publish_creates_a_non_latest_release_with_all_files(self):
        f = Fixture(self.tmp)
        log = self.tmp / "gh.log"
        self.install_fake_gh(log)
        f.run("--publish")
        create = [line for line in log.read_text().splitlines() if line.startswith("release create")][0]
        self.assertIn("engines-test", create)
        self.assertIn("--repo me/radial", create)
        self.assertIn("--latest=false", create)
        for name in ("ffmpeg-9.0.2-macos-aarch64.zip", "ffmpeg-9.0.2-windows-x86_64.tar.gz", "libreoffice-26.8.1-macos-aarch64.dmg"):
            self.assertIn(name, create)

    def test_publish_to_an_existing_release_uploads_and_replaces(self):
        f = Fixture(self.tmp)
        log = self.tmp / "gh.log"
        self.install_fake_gh(log, release_exists=True)
        f.run("--publish")
        text = log.read_text()
        self.assertIn("release upload engines-test", text)
        self.assertIn("--clobber", text)
        self.assertNotIn("release create", text)

    def test_the_notes_carry_the_license_notices(self):
        notes = m.release_notes("engines-test")
        self.assertIn("unmodified copies", notes)
        self.assertIn("GNU", notes)  # from THIRD_PARTY_NOTICES.md


if __name__ == "__main__":
    unittest.main()
