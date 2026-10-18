"""Runs install.sh against a fake release served from localhost. macOS only (the script is)."""

import functools
import hashlib
import http.server
import os
import platform
import subprocess
import tempfile
import threading
import unittest
from pathlib import Path

SCRIPT = Path(__file__).resolve().parent / "install.sh"
ARCH = {"arm64": "aarch64", "x86_64": "x86_64"}.get(platform.machine(), platform.machine())
CLI_ASSET = f"radial-macos-{ARCH}"
APP_ASSET = "Radial-Convert-macos-universal.zip"


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


class QuietHandler(http.server.SimpleHTTPRequestHandler):
    def log_message(self, *args):
        pass


@unittest.skipUnless(platform.system() == "Darwin", "install.sh is for macOS")
class InstallScript(unittest.TestCase):
    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.tmp = Path(self._tmp.name)
        self.release = self.tmp / "release"
        self.release.mkdir()
        self.bin_dir = self.tmp / "bin"
        self.app_dir = self.tmp / "apps"
        self.make_release()
        handler = functools.partial(QuietHandler, directory=str(self.release))
        self.server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), handler)
        threading.Thread(target=self.server.serve_forever, daemon=True).start()
        self.addCleanup(self.server.shutdown)
        self.addCleanup(self._tmp.cleanup)
        self.url = f"http://127.0.0.1:{self.server.server_port}"

    def make_release(self):
        cli = self.release / CLI_ASSET
        cli.write_text('#!/bin/sh\necho "fake radial $@"\n')
        app = self.tmp / "src/Radial Convert.app/Contents/MacOS"
        app.mkdir(parents=True)
        (app / "app").write_text("#!/bin/sh\n")
        subprocess.run(["ditto", "-c", "-k", "--keepParent", str(self.tmp / "src/Radial Convert.app"), str(self.release / APP_ASSET)], check=True)
        self.write_sums()

    def write_sums(self):
        lines = [f"{sha(self.release / n)}  {n}" for n in (CLI_ASSET, APP_ASSET)]
        (self.release / "SHA256SUMS").write_text("\n".join(lines) + "\n")

    def run_script(self, *args: str, url: str | None = None) -> subprocess.CompletedProcess:
        env = dict(os.environ, RADIAL_BASE_URL=url or self.url, RADIAL_INSTALL_DIR=str(self.bin_dir), RADIAL_APP_DIR=str(self.app_dir))
        return subprocess.run(["sh", str(SCRIPT), *args], env=env, stdin=subprocess.DEVNULL, capture_output=True, text=True, timeout=60)

    def test_installs_the_cli_and_the_app(self):
        r = self.run_script("--no-packs")
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        radial = self.bin_dir / "radial"
        self.assertTrue(os.access(radial, os.X_OK))
        self.assertIn("fake radial", subprocess.run([str(radial), "hi"], capture_output=True, text=True).stdout)
        self.assertTrue((self.app_dir / "Radial Convert.app/Contents/MacOS/app").is_file())
        self.assertIn("export PATH", r.stdout, "should tell the user how to put radial on PATH")

    def test_a_tampered_cli_installs_nothing(self):
        (self.release / CLI_ASSET).write_text("#!/bin/sh\necho evil\n")  # SHA256SUMS still has the old hash
        r = self.run_script("--no-packs")
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("checksum mismatch", r.stderr)
        self.assertFalse((self.bin_dir / "radial").exists())
        self.assertFalse(self.app_dir.exists())

    def test_a_tampered_app_installs_nothing_not_even_the_cli(self):
        (self.release / APP_ASSET).write_bytes(b"not the real zip")
        r = self.run_script("--no-packs")
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("checksum mismatch", r.stderr)
        self.assertFalse((self.bin_dir / "radial").exists(), "the CLI must wait for every download to verify")
        self.assertFalse(self.app_dir.exists())

    def test_a_file_missing_from_the_checksum_list_is_refused(self):
        (self.release / "SHA256SUMS").write_text(f"{sha(self.release / CLI_ASSET)}  {CLI_ASSET}\n")
        r = self.run_script("--no-packs")
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("not listed in SHA256SUMS", r.stderr)
        self.assertFalse((self.bin_dir / "radial").exists())

    def test_no_app_installs_only_the_cli(self):
        (self.release / APP_ASSET).unlink()  # would fail if it were requested
        r = self.run_script("--no-app", "--no-packs")
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertTrue((self.bin_dir / "radial").exists())
        self.assertFalse(self.app_dir.exists())

    def test_packs_are_handed_to_the_cli_without_asking(self):
        r = self.run_script("--no-app", "--packs", "audio,images")
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertIn("fake radial install --packs audio,images --yes", r.stdout)

    def test_an_existing_app_is_replaced_not_merged(self):
        old = self.app_dir / "Radial Convert.app/Contents/old-file"
        old.parent.mkdir(parents=True)
        old.write_text("left over from a previous version")
        r = self.run_script("--no-packs")
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertFalse(old.exists())
        self.assertTrue((self.app_dir / "Radial Convert.app/Contents/MacOS/app").is_file())

    def test_an_unreachable_release_is_a_clear_error(self):
        r = self.run_script("--no-packs", url="http://127.0.0.1:9")
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("could not download", r.stderr)

    def test_an_unknown_option_is_rejected(self):
        r = self.run_script("--bogus")
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("unknown option", r.stderr)

    def test_the_picker_hint_appears_when_there_is_no_terminal(self):
        # stdin is not a terminal in this test, so the script must not hang on the picker.
        r = self.run_script("--no-app")
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)


if __name__ == "__main__":
    unittest.main()
