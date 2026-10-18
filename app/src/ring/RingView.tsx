import type { CSSProperties } from "react";
import { HUB_RADIUS, RING_SIZE } from "./constants";
import { FormatIcon } from "./FormatIcon";
import { iconKindFor, PALETTE } from "./formatCategories";
import { contentPoint, segmentPath } from "./geometry";
import type { Wedge } from "./types";
import "./Ring.css";

export interface RingViewProps {
  wedges: Wedge[];
  fileName: string;
  hovered: number | null;
  confirmed: number | null;
  /** Called when the chosen segment's confirmation animation has played. */
  onConfirmed: () => void;
}

type SegmentState = "idle" | "hot" | "chosen" | "faded";

function stateOf(i: number, hovered: number | null, confirmed: number | null): SegmentState {
  if (confirmed !== null) return i === confirmed ? "chosen" : "faded";
  return i === hovered ? "hot" : "idle";
}

interface HubText {
  caption: string | null;
  main: string;
  kind: "name" | "label" | "note";
}

/** What the center disc says: the file, the hovered target, or a short status. */
function hubText(p: RingViewProps): HubText {
  if (p.wedges.length === 0) return { caption: null, main: "No conversions available", kind: "note" };
  if (p.confirmed !== null) return { caption: null, main: "Converting\u2026", kind: "note" };
  if (p.hovered !== null) return { caption: "Convert to", main: p.wedges[p.hovered].label, kind: "label" };
  return { caption: null, main: p.fileName, kind: "name" };
}

/** Inline CSS variables carrying a segment's color identity and stagger order. */
function segmentVars(id: string, i: number): CSSProperties {
  return { "--c": PALETTE[iconKindFor(id)], "--i": i } as CSSProperties;
}

/** Presentational dial: all state comes from props so it can be previewed without Tauri. */
export function RingView(props: RingViewProps) {
  const { wedges, hovered, confirmed, onConfirmed } = props;
  const count = wedges.length;
  const hub = hubText(props);
  const focused = confirmed ?? hovered;
  const hubStyle = focused === null ? undefined : segmentVars(wedges[focused].id, 0);

  return (
    <div className="ring" style={{ width: RING_SIZE, height: RING_SIZE }}>
      <svg width={RING_SIZE} height={RING_SIZE} viewBox={`0 0 ${RING_SIZE} ${RING_SIZE}`}>
        {wedges.map((w, i) => {
          const state = stateOf(i, hovered, confirmed);
          const d = segmentPath(i, count);
          return (
            <g
              key={w.id}
              className={`seg ${state}`}
              style={segmentVars(w.id, i)}
              onAnimationEnd={(e) => {
                if (state === "chosen" && e.animationName === "chosen-pop") onConfirmed();
              }}
            >
              {/* Two stacked paths: the wider one shows as a hairline edge around the face. */}
              <path className="edge" d={d} fillRule="evenodd" />
              <path className="face" d={d} fillRule="evenodd" />
            </g>
          );
        })}
      </svg>

      {wedges.map((w, i) => {
        const p = contentPoint(i, count);
        const style = { ...segmentVars(w.id, i), left: p.x, top: p.y } as CSSProperties;
        return (
          <div key={w.id} className="slot" style={style}>
            <div className={`content ${stateOf(i, hovered, confirmed)}`}>
              <FormatIcon id={w.id} />
              <span className="ext">{w.label}</span>
            </div>
          </div>
        );
      })}

      <div
        className={`hub ${hub.kind}${focused === null ? "" : " lit"}`}
        style={{ ...hubStyle, width: HUB_RADIUS * 2, height: HUB_RADIUS * 2 }}
      >
        {hub.caption && <span className="caption">{hub.caption}</span>}
        <span className="main">{hub.main}</span>
      </div>
    </div>
  );
}
