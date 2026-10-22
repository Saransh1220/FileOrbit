import type { CSSProperties } from "react";
import { iconKindFor, PALETTE } from "./formatCategories";
import type { RingViewProps } from "./ringViewProps";
import type { Wedge } from "./types";

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

/** The id of the wedge (and hub drop) that opens the page editor; must match selection.rs. */
export const ORGANIZE = "organize";
/** The id of the wedge that opens the picture tool; must match selection.rs. */
export const RESIZE = "resize";

function captionFor(wedge: Wedge): string {
  if (wedge.id === ORGANIZE) return "Reorder, rotate, delete";
  if (wedge.id === RESIZE) return "Resize or compress";
  return wedge.tool ? "Into one PDF" : "Convert to";
}

/** What the center says: the file, the hovered target, or a short status. */
export function hubText(p: RingViewProps): HubText {
  if (p.wedges.length === 0) return { caption: null, main: "No conversions available", kind: "note" };
  if (p.confirmed !== null) {
    const opening = p.wedges[p.confirmed]?.id === ORGANIZE;
    return { caption: null, main: opening ? "Opening\u2026" : "Converting\u2026", kind: "note" };
  }
  if (p.hub) return { caption: "Drop to open", main: "Editor", kind: "label" };
  if (p.hovered !== null) {
    const wedge = p.wedges[p.hovered];
    return { caption: captionFor(wedge), main: wedge.label, kind: "label" };
  }
  return { caption: null, main: p.fileName, kind: "name" };
}

/** Inline CSS variables carrying a segment's color identity and stagger order. */
export function segmentVars(id: string, i: number): CSSProperties {
  return { "--c": PALETTE[iconKindFor(id)], "--i": i } as CSSProperties;
}
