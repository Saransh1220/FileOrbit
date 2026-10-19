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
