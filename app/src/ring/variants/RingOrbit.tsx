import type { CSSProperties } from "react";
import { RING_SIZE } from "../constants";
import { FormatIcon } from "../FormatIcon";
import { midAngle, polar } from "../geometry";
import { hubText, segmentVars, stateOf } from "../ringState";
import type { RingViewProps } from "../ringViewProps";
import "./RingOrbit.css";

/** Radius of the track the nodes sit on. */
const TRACK_R = 86;
const CENTER = RING_SIZE / 2;

/**
 * Orbit: no wedge fills at all. A hairline track carries round nodes; the
 * hovered node swells into a solid disc and a thin spoke ties it to the hub.
 */
export function RingOrbit(props: RingViewProps) {
  const { wedges, hovered, confirmed, onConfirmed } = props;
  const count = wedges.length;
  const hub = hubText(props);
  const focused = confirmed ?? hovered;
  const hubStyle = focused === null ? undefined : segmentVars(wedges[focused].id, 0);

  return (
    <div className={`orbit${focused === null ? "" : " has-focus"}`} style={{ width: RING_SIZE, height: RING_SIZE }}>
      <svg width={RING_SIZE} height={RING_SIZE} viewBox={`0 0 ${RING_SIZE} ${RING_SIZE}`}>
        <circle className="track" cx={CENTER} cy={CENTER} r={TRACK_R} />
        {focused !== null && count > 0 && (
          <line
            className="spoke"
            style={segmentVars(wedges[focused].id, 0)}
            x1={CENTER}
            y1={CENTER}
            x2={polar(TRACK_R, midAngle(focused, count)).x}
            y2={polar(TRACK_R, midAngle(focused, count)).y}
          />
        )}
      </svg>

      {wedges.map((w, i) => {
        const p = polar(TRACK_R, midAngle(i, count));
        const state = stateOf(i, hovered, confirmed);
        const style = { ...segmentVars(w.id, i), left: p.x, top: p.y } as CSSProperties;
        return (
          <div key={w.id} className="slot" style={style}>
            <div
              className={`node ${state}`}
              onAnimationEnd={(e) => {
                if (state === "chosen" && e.animationName === "orbit-chosen") onConfirmed();
              }}
            >
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
