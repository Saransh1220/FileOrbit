import type { CSSProperties } from "react";
import { iconKindFor, PALETTE } from "./formatCategories";
import type { RingViewProps } from "./ringViewProps";

export type SegmentState = "idle" | "hot" | "chosen" | "faded";

export function stateOf(i: number, hovered: number | null, confirmed: number | null): SegmentState {
  if (confirmed !== null) return i === confirmed ? "chosen" : "faded";
  return i === hovered ? "hot" : "idle";
}

export interface HubText {
  caption: string | null;
  main: string;
  kind: "name" | "label" | "note";
}

/** What the center says: the file, the hovered target, or a short status. */
export function hubText(p: RingViewProps): HubText {
  if (p.wedges.length === 0) return { caption: null, main: "No conversions available", kind: "note" };
  if (p.confirmed !== null) return { caption: null, main: "Converting\u2026", kind: "note" };
  if (p.hovered !== null) {
    const wedge = p.wedges[p.hovered];
    return { caption: wedge.tool ? "Into one PDF" : "Convert to", main: wedge.label, kind: "label" };
  }
  return { caption: null, main: p.fileName, kind: "name" };
}

/** Inline CSS variables carrying a segment's color identity and stagger order. */
export function segmentVars(id: string, i: number): CSSProperties {
  return { "--c": PALETTE[iconKindFor(id)], "--i": i } as CSSProperties;
}
