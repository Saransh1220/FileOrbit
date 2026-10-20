"""Runs install.ps1 against a fake release served from localhost. Windows only (the script is).

The script adds its folder to the user's PATH (a persistent change in the
registry), so the test saves that value first and always puts it back.
"""

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

SCRIPT = Path(__file__).resolve().parent / "install.ps1"
CLI_ASSET = "radial-windows-x86_64.exe"
APP_ASSET = "Radial-Convert-windows-x86_64-setup.exe"


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def powershell(command: str) -> str:
    result = subprocess.run(
        ["powershell", "-NoProfile", "-NonInteractive", "-Command", command],
        capture_output=True,
        text=True,
        timeout=60,
    )
    return result.stdout.strip()


class QuietHandler(http.server.SimpleHTTPRequestHandler):
    def log_message(self, *args):
        pass


@unittest.skipUnless(platform.system() == "Windows", "install.ps1 is for Windows")
class InstallScript(unittest.TestCase):
    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.tmp = Path(self._tmp.name)
        self.release = self.tmp / "release"
        self.release.mkdir()
        self.bin_dir = self.tmp / "bin dir"  # a space on purpose: the real folder has one too
        self.make_release()

        # Remember the user PATH exactly, and restore it whatever happens.
        self.original_path = powershell("[Environment]::GetEnvironmentVariable('Path','User')")
        self.addCleanup(self.restore_path)

        handler = functools.partial(QuietHandler, directory=str(self.release))
        self.server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), handler)
        threading.Thread(target=self.server.serve_forever, daemon=True).start()
        self.addCleanup(self.server.server_close)
        self.addCleanup(self.server.shutdown)
        self.addCleanup(self._tmp.cleanup)
        self.url = f"http://127.0.0.1:{self.server.server_port}"

    def restore_path(self):
        value = self.original_path.replace("'", "''")
        powershell(f"[Environment]::SetEnvironmentVariable('Path','{value}','User')")

    def make_release(self):
        (self.release / CLI_ASSET).write_bytes(b"MZ fake radial command")
        (self.release / APP_ASSET).write_bytes(b"MZ fake installer")
        self.write_sums()

    def write_sums(self, names=(CLI_ASSET, APP_ASSET)):
        lines = [f"{sha(self.release / n).upper()}  {n}" for n in names]
        (self.release / "SHA256SUMS").write_text("\n".join(lines) + "\n")

    def run_script(self, *args: str, url: str | None = None) -> subprocess.CompletedProcess:
        env = dict(os.environ, RADIAL_BASE_URL=url or self.url, RADIAL_INSTALL_DIR=str(self.bin_dir))
        return subprocess.run(
            ["powershell", "-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-File", str(SCRIPT), *args],
            env=env,
            stdin=subprocess.DEVNULL,
            capture_output=True,
            text=True,
            timeout=120,
        )

    def test_installs_the_cli_and_puts_it_on_the_path(self):
        r = self.run_script("-NoApp", "-NoPacks")
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertEqual((self.bin_dir / "radial.exe").read_bytes(), b"MZ fake radial command")
        user_path = powershell("[Environment]::GetEnvironmentVariable('Path','User')")
        self.assertIn(str(self.bin_dir), user_path.split(";"))

    def test_a_tampered_cli_installs_nothing(self):
        (self.release / CLI_ASSET).write_bytes(b"MZ evil")  # SHA256SUMS still has the old hash
        r = self.run_script("-NoApp", "-NoPacks")
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("checksum mismatch", r.stdout + r.stderr)
        self.assertFalse((self.bin_dir / "radial.exe").exists())

    def test_a_tampered_app_installs_nothing_not_even_the_cli(self):
        (self.release / APP_ASSET).write_bytes(b"MZ evil")
        r = self.run_script("-NoPacks")
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("checksum mismatch", r.stdout + r.stderr)
        self.assertFalse((self.bin_dir / "radial.exe").exists(), "nothing is installed until everything verified")

    def test_a_file_missing_from_the_checksums_is_refused(self):
        self.write_sums(names=(APP_ASSET,))
        r = self.run_script("-NoApp", "-NoPacks")
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("not listed in SHA256SUMS", r.stdout + r.stderr)
        self.assertFalse((self.bin_dir / "radial.exe").exists())

    def test_the_placeholder_address_is_refused(self):
        r = self.run_script("-NoApp", "-NoPacks", url="https://github.com/OWNER/REPO/releases/latest/download")
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("not been pointed at a release", r.stdout + r.stderr)

    def test_an_unreachable_server_is_a_clear_error(self):
        r = self.run_script("-NoApp", "-NoPacks", url="http://127.0.0.1:9")
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("could not download", r.stdout + r.stderr)
        self.assertFalse((self.bin_dir / "radial.exe").exists())


if __name__ == "__main__":
    unittest.main()
