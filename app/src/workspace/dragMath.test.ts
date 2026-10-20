import { describe, expect, it } from "vitest";
import { dropIndex, movedFar } from "./dragMath";
import type { Rect } from "./types";

// Two rows of three cards, 100 wide, 140 tall, with 10 px gaps.
const rects: Rect[] = Array.from({ length: 6 }, (_, i) => ({ left: (i % 3) * 110, top: Math.floor(i / 3) * 150, width: 100, height: 140 }));

describe("where a dragged page lands", () => {
  it("goes before a card when the pointer is on its left half", () => {
    expect(dropIndex(rects, 120, 50)).toBe(1);
  });

  it("goes after a card when the pointer is on its right half", () => {
    expect(dropIndex(rects, 200, 50)).toBe(2);
  });

  it("goes to the end past the last card", () => {
    expect(dropIndex(rects, 400, 200)).toBe(6);
  });

  it("goes to the start before the first card", () => {
    expect(dropIndex(rects, -30, 10)).toBe(0);
  });

  it("uses the row the pointer is in, also when it is in a gap", () => {
    // Left half of the first card of row two.
    expect(dropIndex(rects, 40, 220)).toBe(3);
    // In the gap between rows, nearer to the top row's second card.
    expect(dropIndex(rects, 115, 144)).toBe(1);
  });

  it("an empty grid takes it at the start", () => {
    expect(dropIndex([], 5, 5)).toBe(0);
  });

  it("tells a click from a drag", () => {
    expect(movedFar(10, 10, 12, 11)).toBe(false);
    expect(movedFar(10, 10, 20, 10)).toBe(true);
  });
});
