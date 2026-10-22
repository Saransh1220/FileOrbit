"""Takes the README screenshots from the app's browser demo pages.

Needs the UI dev server running (`cd app && npm run dev`) and Microsoft Edge or Chrome.
The demo pages are opened with ?clean, which hides their row of preview buttons.

    python scripts/make_screenshots.py            # all of them
    python scripts/make_screenshots.py workspace  # one of them

Pictures go to docs/assets/. The demo pages use made-up files, so nothing personal is shown.
"""

import subprocess
import sys
import tempfile
import time
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "docs" / "assets"
URL = "http://localhost:1420/"

BROWSERS = [
    r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
    r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
    r"C:\Program Files\Google\Chrome\Application\chrome.exe",
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
]

# name -> (page address, window size, dark mode)
SHOTS = {
    "ring": ("?clean&set=docx&bg=dark%20wallpaper&theme=dark#ring-demo", (1180, 1500), True),
    "workspace": ("?clean&mode=two%20files#workspace-demo", (1200, 780), True),
    "manager": ("?clean#manager-demo", (1100, 760), True),
    "imagetool": ("?clean&count=3&preset=email#imagetool-demo", (1200, 760), True),
}


def browser() -> str:
    for path in BROWSERS:
        if Path(path).exists():
            return path
    sys.exit("Edge or Chrome was not found; edit BROWSERS in this script.")


# Where each theme's ring sits in the tall "ring" capture: (left, top) of a 230 x 245 crop.
RING_SPOTS = [(10, 15), (585, 15), (10, 308), (585, 308), (10, 599), (585, 599), (10, 891)]


def tile_rings(image: Image.Image) -> Image.Image:
    """Cuts the rings out of the tall capture and lays them side by side, four to a row."""
    width, height, gap = 230, 245, 14
    columns = 4
    rows = -(-len(RING_SPOTS) // columns)
    background = image.getpixel((600, 1400))
    sheet = Image.new("RGB", (columns * width + (columns + 1) * gap, rows * height + (rows + 1) * gap), background)
    for number, (left, top) in enumerate(RING_SPOTS):
        tile = image.crop((left, top, left + width, top + height))
        sheet.paste(tile, (gap + (number % columns) * (width + gap), gap + (number // columns) * (height + gap)))
    return sheet


def shoot(name: str) -> None:
    page, (width, height), dark = SHOTS[name]
    target = OUT / f"{name}.png"
    # The browser may still be closing when the profile folder is removed; that is harmless.
    with tempfile.TemporaryDirectory(ignore_cleanup_errors=True) as profile:
        command = [
            browser(), "--headless=new", "--disable-gpu", "--hide-scrollbars",
            f"--window-size={width},{height}", "--virtual-time-budget=40000",
            f"--user-data-dir={profile}", f"--screenshot={target}", URL.rstrip("/") + "/" + page,
        ]
        if dark:
            command.insert(2, "--force-dark-mode")
        target.unlink(missing_ok=True)
        subprocess.run(command, check=True, capture_output=True, timeout=180)
        # The browser can return a moment before its last child process has written the file.
        for _ in range(100):
            if target.exists() and target.stat().st_size > 0:
                break
            time.sleep(0.2)
        else:
            sys.exit(f"{name}: the browser wrote no screenshot")
    image = Image.open(target)
    if name == "ring":
        image = tile_rings(image.convert("RGB"))
    image.save(target, optimize=True)
    print(f"{target.relative_to(ROOT)}  {image.width}x{image.height}")


if __name__ == "__main__":
    OUT.mkdir(parents=True, exist_ok=True)
    for chosen in sys.argv[1:] or SHOTS:
        shoot(chosen)
