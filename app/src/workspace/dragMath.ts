import type { Rect } from "./types";

/** Distance from a point to a rectangle: 0 inside it. */
function distance(rect: Rect, x: number, y: number): number {
  const dx = Math.max(rect.left - x, 0, x - (rect.left + rect.width));
  const dy = Math.max(rect.top - y, 0, y - (rect.top + rect.height));
  return Math.hypot(dx, dy);
}

/**
 * Where a dragged page would land: the index of the card it goes before, or `rects.length`
 * for the end. Picks the nearest card, then the side of it the pointer is on.
 */
export function dropIndex(rects: readonly Rect[], x: number, y: number): number {
  if (rects.length === 0) return 0;
  let nearest = 0;
  let best = Infinity;
  rects.forEach((rect, i) => {
    const d = distance(rect, x, y);
    if (d < best) {
      best = d;
      nearest = i;
    }
  });
  const rect = rects[nearest];
  return x > rect.left + rect.width / 2 ? nearest + 1 : nearest;
}

/** How far a press must travel before it counts as a drag rather than a click. */
export const DRAG_THRESHOLD_PX = 5;

export const movedFar = (startX: number, startY: number, x: number, y: number) => Math.hypot(x - startX, y - startY) >= DRAG_THRESHOLD_PX;
