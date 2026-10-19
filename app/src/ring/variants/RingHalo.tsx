import type { CSSProperties } from "react";
import { RING_SIZE } from "../constants";
import { FormatIcon } from "../FormatIcon";
import { midAngle, polar } from "../geometry";
import { hubText, segmentVars, stateOf } from "../ringState";
import type { RingViewProps } from "../ringViewProps";
import { arcPath } from "./sectorGeometry";
import "./RingHalo.css";

const CENTER = RING_SIZE / 2;
const ARC_R = 118;
const CONTENT_R = 86;
const GAP = 0.09;

/**
 * Halo: no wedges. Each target is a thin colored arc on the outer edge, with
 * its icon and label floating inside. The hovered arc thickens and lights up.
 */
export function RingHalo(props: RingViewProps) {
  const { wedges, hovered, confirmed, onConfirmed } = props;
  const count = wedges.length;
  const hub = hubText(props);
  const focused = confirmed ?? hovered;
  const hubStyle = focused === null ? undefined : segmentVars(wedges[focused].id, 0);

  return (
    <div className={`halo${focused === null ? "" : " has-focus"}`} style={{ width: RING_SIZE, height: RING_SIZE }}>
      <svg width={RING_SIZE} height={RING_SIZE} viewBox={`0 0 ${RING_SIZE} ${RING_SIZE}`}>
        <circle className="disc" cx={CENTER} cy={CENTER} r={ARC_R + 10} />
        {wedges.map((w, i) => {
          const state = stateOf(i, hovered, confirmed);
          return (
            <path
              key={w.id}
              className={`arc ${state}`}
              style={segmentVars(w.id, i)}
              d={arcPath(i, count, ARC_R, GAP)}
              onAnimationEnd={(e) => {
                if (state === "chosen" && e.animationName === "halo-chosen") onConfirmed();
              }}
            />
          );
        })}
      </svg>

      {wedges.map((w, i) => {
        const p = polar(CONTENT_R, midAngle(i, count));
        const state = stateOf(i, hovered, confirmed);
        const style = { ...segmentVars(w.id, i), left: p.x, top: p.y } as CSSProperties;
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
