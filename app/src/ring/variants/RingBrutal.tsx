import type { CSSProperties } from "react";
import { DIAL_INNER, DIAL_OUTER, RING_SIZE } from "../constants";
import { FormatIcon } from "../FormatIcon";
import { contentPoint } from "../geometry";
import { hubText, segmentVars, stateOf } from "../ringState";
import type { RingViewProps } from "../ringViewProps";
import { sectorPath } from "./sectorGeometry";
import "./RingBrutal.css";

const GAP = 0.075;

/**
 * Brutal: chunky sharp blocks with black outlines and hard offset shadows.
 * Pastel fill per file kind; the hovered block presses toward its shadow.
 */
export function RingBrutal(props: RingViewProps) {
  const { wedges, hovered, confirmed, onConfirmed } = props;
  const count = wedges.length;
  const hub = hubText(props);
  const focused = confirmed ?? hovered;

  return (
    <div className={`brutal${focused === null ? "" : " has-focus"}`} style={{ width: RING_SIZE, height: RING_SIZE }}>
      <svg width={RING_SIZE} height={RING_SIZE} viewBox={`0 0 ${RING_SIZE} ${RING_SIZE}`}>
        {wedges.map((w, i) => {
          const state = stateOf(i, hovered, confirmed);
          const d = sectorPath(i, count, DIAL_INNER, DIAL_OUTER, GAP);
          return (
            <g
              key={w.id}
              className={`block ${state}`}
              style={segmentVars(w.id, i)}
              onAnimationEnd={(e) => {
                if (state === "chosen" && e.animationName === "brutal-chosen") onConfirmed();
              }}
            >
              <path className="shadow" d={d} />
              <path className="face" d={d} />
            </g>
          );
        })}
      </svg>

      {wedges.map((w, i) => {
        const p = contentPoint(i, count);
        const state = stateOf(i, hovered, confirmed);
        const style = { "--i": i, left: p.x, top: p.y } as CSSProperties;
        return (
          <div key={w.id} className="slot" style={style}>
            <div className={`content ${state}`}>
              <FormatIcon id={w.id} />
              <span className="ext">{w.label}</span>
            </div>
          </div>
        );
      })}

      <div className={`hub ${hub.kind}${focused === null ? "" : " lit"}`}>
        {hub.caption && <span className="caption">{hub.caption}</span>}
        <span className="main">{hub.main}</span>
      </div>
    </div>
  );
}
