import type { Point } from "./types";

const IS_MAC = navigator.userAgent.includes("Mac");

/**
 * Drag-drop positions arrive in different units per OS: wry on macOS reports
 * points (already CSS pixels), while on Windows it reports physical pixels.
 */
export function dragPositionToCss(pos: Point): Point {
  const scale = IS_MAC ? 1 : window.devicePixelRatio;
  return { x: pos.x / scale, y: pos.y / scale };
}
