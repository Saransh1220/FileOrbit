import {
  CORNER_PX,
  DIAL_INNER,
  DIAL_OUTER,
  INNER_RADIUS,
  OUTER_RADIUS,
  RING_SIZE,
  SEGMENT_GAP_PX,
} from "./constants";
import type { Point } from "./types";

const CENTER = RING_SIZE / 2;
const TAU = Math.PI * 2;

/** Index of the segment under a point in ring coordinates, or null if outside the dial. */
export function wedgeIndexAt(p: Point, count: number): number | null {
  // No segments (unsupported file type): dividing by zero would yield NaN,
  // which callers would mistake for a hovered segment.
  if (count === 0) return null;
  const dx = p.x - CENTER;
  const dy = p.y - CENTER;
  const r = Math.hypot(dx, dy);
  if (r < INNER_RADIUS || r > OUTER_RADIUS) return null;
  const step = TAU / count;
  // Segment 0 is centered at 12 o'clock, so shift by half a step.
  const fromTop = Math.atan2(dy, dx) + Math.PI / 2 + step / 2;
  const normalized = ((fromTop % TAU) + TAU) % TAU;
  return Math.floor(normalized / step) % count;
}

/** Angle (radians, 0 = 3 o'clock) at the middle of a segment; index 0 is 12 o'clock. */
export const midAngle = (index: number, count: number) => -Math.PI / 2 + (index * TAU) / count;

/** Point at `radius` along an angle, in ring coordinates. */
export function polar(radius: number, angle: number): Point {
  return { x: CENTER + radius * Math.cos(angle), y: CENTER + radius * Math.sin(angle) };
}

/**
 * Where a segment's straight edge meets a circle of `radius`, with the edge
 * moved `offset` px toward the segment's inside so neighbours keep a parallel,
 * constant-width gap. `side` is +1 for the segment's start edge, -1 for its end.
 */
function edgeAngle(boundary: number, side: 1 | -1, radius: number, offset: number): number {
  const along = Math.sqrt(radius * radius - offset * offset);
  const inward = boundary + side * (Math.PI / 2);
  const x = along * Math.cos(boundary) + offset * Math.cos(inward);
  const y = along * Math.sin(boundary) + offset * Math.sin(inward);
  return Math.atan2(y, x);
}

const fmt = (p: Point) => `${p.x.toFixed(2)} ${p.y.toFixed(2)}`;

/**
 * SVG path for one donut segment. The path is inset by CORNER_PX and meant to
 * be drawn with a stroke of 2 * CORNER_PX in the fill color and round joins,
 * which gives rounded corners without any curve math.
 */
export function segmentPath(index: number, count: number): string {
  const rIn = DIAL_INNER + CORNER_PX;
  const rOut = DIAL_OUTER - CORNER_PX;

  if (count === 1) {
    // A lone segment is a full ring: two half-arcs per edge, filled even-odd.
    const ring = (r: number) =>
      `M ${fmt(polar(r, 0))} A ${r} ${r} 0 1 1 ${fmt(polar(r, Math.PI))} A ${r} ${r} 0 1 1 ${fmt(polar(r, 0))} Z`;
    return `${ring(rOut)} ${ring(rIn)}`;
  }

  const mid = midAngle(index, count);
  const half = Math.PI / count;
  const startEdge = mid - half;
  const endEdge = mid + half;
  const offset = SEGMENT_GAP_PX / 2 + CORNER_PX;

  const a0In = edgeAngle(startEdge, 1, rIn, offset);
  const a0Out = edgeAngle(startEdge, 1, rOut, offset);
  const a1Out = edgeAngle(endEdge, -1, rOut, offset);
  const a1In = edgeAngle(endEdge, -1, rIn, offset);
  const sweep = (((a1Out - a0Out) % TAU) + TAU) % TAU;
  const large = sweep > Math.PI ? 1 : 0;

  return [
    `M ${fmt(polar(rIn, a0In))}`,
    `L ${fmt(polar(rOut, a0Out))}`,
    `A ${rOut} ${rOut} 0 ${large} 1 ${fmt(polar(rOut, a1Out))}`,
    `L ${fmt(polar(rIn, a1In))}`,
    `A ${rIn} ${rIn} 0 ${large} 0 ${fmt(polar(rIn, a0In))}`,
    "Z",
  ].join(" ");
}

/** Where a segment's icon and label sit: the middle of its band. */
export function contentPoint(index: number, count: number): Point {
  return polar((DIAL_INNER + DIAL_OUTER) / 2, midAngle(index, count));
}
