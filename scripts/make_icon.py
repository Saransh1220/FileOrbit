#!/usr/bin/env python3
"""Writes the app icon source (app/src-tauri/icons/source.svg).

The icon is the dial itself: six segments in a ring on a graphite tile, one of
them lit in the accent color. Run it, then generate every size with:

    cd app && npm run tauri icon src-tauri/icons/source.svg
"""

import math
from pathlib import Path

SIZE = 1024
CENTER = SIZE / 2
SEGMENTS = 6
INNER, OUTER = 168, 338
GAP = 0.215  # radians between neighbours (the rounded stroke eats about half of it)
TILE = "#1f2430"
SEGMENT = "#e9ecf3"
LIT = "#5b8def"


def point(radius: float, angle: float) -> str:
    return f"{CENTER + radius * math.cos(angle):.2f} {CENTER + radius * math.sin(angle):.2f}"


def sector(index: int) -> str:
    # Segment 0 is centered at 12 o'clock, like the ring in the app.
    mid = -math.pi / 2 + index * 2 * math.pi / SEGMENTS
    half = math.pi / SEGMENTS - GAP / 2
    a0, a1 = mid - half, mid + half
    return (
        f"M {point(INNER, a0)} L {point(OUTER, a0)} "
        f"A {OUTER} {OUTER} 0 0 1 {point(OUTER, a1)} "
        f"L {point(INNER, a1)} A {INNER} {INNER} 0 0 0 {point(INNER, a0)} Z"
    )


def build() -> str:
    paths = []
    for i in range(SEGMENTS):
        fill = LIT if i == 1 else SEGMENT
        paths.append(f'  <path d="{sector(i)}" fill="{fill}" stroke="{fill}" stroke-width="26" stroke-linejoin="round"/>')
    return "\n".join(
        [
            f'<svg xmlns="http://www.w3.org/2000/svg" width="{SIZE}" height="{SIZE}" viewBox="0 0 {SIZE} {SIZE}">',
            f'  <rect width="{SIZE}" height="{SIZE}" rx="230" fill="{TILE}"/>',
            *paths,
            f'  <circle cx="{CENTER}" cy="{CENTER}" r="64" fill="{SEGMENT}"/>',
            "</svg>",
            "",
        ]
    )


if __name__ == "__main__":
    target = Path(__file__).resolve().parent.parent / "app" / "src-tauri" / "icons" / "source.svg"
    target.write_text(build(), encoding="utf-8")
    print(f"wrote {target}")
