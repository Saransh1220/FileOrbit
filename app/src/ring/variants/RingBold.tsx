import type { CSSProperties } from "react";
import { RING_SIZE } from "../constants";
import { FormatIcon } from "../FormatIcon";
import { contentPoint, midAngle, segmentPath } from "../geometry";
import { hubText, segmentVars, stateOf } from "../ringState";
import type { RingViewProps } from "../ringViewProps";
import "./RingBold.css";

/** How far the hovered segment moves outward, in px. */
const PUSH_PX = 7;

/** Outward offset of a segment as CSS variables, so the move follows its own direction. */
function pushVars(i: number, count: number): CSSProperties {
  const a = midAngle(i, count);
  return { "--dx": `${(Math.cos(a) * PUSH_PX).toFixed(2)}px`, "--dy": `${(Math.sin(a) * PUSH_PX).toFixed(2)}px` } as CSSProperties;
}

/**
 * Bold: every segment is a solid block of its own color with white icons.
 * The hovered one pushes outward while the rest dim; the hub is a dark disc.
 */
export function RingBold(props: RingViewProps) {
  const { wedges, hovered, confirmed, onConfirmed } = props;
  const count = wedges.length;
  const hub = hubText(props);
  const focused = confirmed ?? hovered;
  const hubStyle = focused === null ? undefined : segmentVars(wedges[focused].id, 0);

  return (
    <div className={`bold${focused === null ? "" : " has-focus"}`} style={{ width: RING_SIZE, height: RING_SIZE }}>
      <svg width={RING_SIZE} height={RING_SIZE} viewBox={`0 0 ${RING_SIZE} ${RING_SIZE}`}>
        {wedges.map((w, i) => {
          const state = stateOf(i, hovered, confirmed);
          return (
            <g
              key={w.id}
              className={`seg ${state}`}
              style={{ ...segmentVars(w.id, i), ...pushVars(i, count) }}
              onAnimationEnd={(e) => {
                if (state === "chosen" && e.animationName === "bold-chosen") onConfirmed();
              }}
            >
              <path d={segmentPath(i, count)} fillRule="evenodd" />
            </g>
          );
        })}
      </svg>

      {wedges.map((w, i) => {
        const p = contentPoint(i, count);
        const state = stateOf(i, hovered, confirmed);
        const style = { ...segmentVars(w.id, i), ...pushVars(i, count), left: p.x, top: p.y } as CSSProperties;
        return (
          <div key={w.id} className="slot" style={style}>
            <div className={`content ${state}`}>
              <FormatIcon id={w.id} />
              <span className="ext">{w.label}</span>
            </div>
          </div>
        );
      })}

      <div className={`hub ${hub.kind}${focused === null ? "" : " lit"}`} style={hubStyle}>
        {hub.caption && <span className="caption">{hub.caption}</span>}
        <span className="main">{hub.main}</span>
      </div>
    </div>
  );
}
