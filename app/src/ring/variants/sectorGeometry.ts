import { polar, midAngle } from "../geometry";

const fmt = (n: number) => n.toFixed(2);
const pt = (r: number, a: number) => {
  const p = polar(r, a);
  return `${fmt(p.x)} ${fmt(p.y)}`;
};

/**
 * Sharp-cornered donut segment between two radii with a constant angular gap
 * to its neighbours. For designs that outline segments instead of filling
 * them (the rounded `segmentPath` is made for thick strokes).
 */
export function sectorPath(index: number, count: number, rIn: number, rOut: number, gapRad: number): string {
  const mid = midAngle(index, count);
  const half = Math.PI / count - gapRad / 2;
  const a0 = mid - half;
  const a1 = mid + half;
  const large = a1 - a0 > Math.PI ? 1 : 0;
  return [
    `M ${pt(rIn, a0)}`,
    `L ${pt(rOut, a0)}`,
    `A ${rOut} ${rOut} 0 ${large} 1 ${pt(rOut, a1)}`,
    `L ${pt(rIn, a1)}`,
    `A ${rIn} ${rIn} 0 ${large} 0 ${pt(rIn, a0)}`,
    "Z",
  ].join(" ");
}

/** One path of small radial ticks around a circle: `count` ticks, every `major`th one longer. */
export function tickRing(radius: number, count: number, major: number, minorLen: number, majorLen: number): string {
  const parts: string[] = [];
  for (let i = 0; i < count; i++) {
    const a = (i * Math.PI * 2) / count - Math.PI / 2;
    const len = i % major === 0 ? majorLen : minorLen;
    parts.push(`M ${pt(radius, a)} L ${pt(radius + len, a)}`);
  }
  return parts.join(" ");
}

/** Corner points of a flat-topped hexagon centered on (cx, cy), as an SVG `points` value. */
export function hexPoints(cx: number, cy: number, radius: number): string {
  return Array.from({ length: 6 }, (_, k) => {
    const a = (k * Math.PI) / 3;
    return `${fmt(cx + radius * Math.cos(a))},${fmt(cy + radius * Math.sin(a))}`;
  }).join(" ");
}
