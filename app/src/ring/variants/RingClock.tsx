import type { CSSProperties } from "react";
import { RING_SIZE } from "../constants";
import { midAngle, polar } from "../geometry";
import { hubText, stateOf } from "../ringState";
import type { RingViewProps } from "../ringViewProps";
import { tickRing } from "./sectorGeometry";
import "./RingClock.css";

const CENTER = RING_SIZE / 2;
const FACE_R = 112;
const LABEL_R = 84;
const TICKS = tickRing(FACE_R - 9, 60, 5, 4, 8);

/**
 * Clock: an editorial face. Serif labels sit on a thin ring with a tick
 * scale; the hovered target gets a marker on the ring and an underline.
 */
export function RingClock(props: RingViewProps) {
  const { wedges, hovered, confirmed, onConfirmed } = props;
  const count = wedges.length;
  const hub = hubText(props);
  const focused = confirmed ?? hovered;

  return (
    <div className={`clock${focused === null ? "" : " has-focus"}`} style={{ width: RING_SIZE, height: RING_SIZE }}>
      <svg width={RING_SIZE} height={RING_SIZE} viewBox={`0 0 ${RING_SIZE} ${RING_SIZE}`}>
        <circle className="face" cx={CENTER} cy={CENTER} r={FACE_R + 12} />
        <circle className="rim" cx={CENTER} cy={CENTER} r={FACE_R} />
        <path className="ticks" d={TICKS} />
        {focused !== null && count > 0 && (
          <circle
            className="marker"
            cx={polar(FACE_R, midAngle(focused, count)).x}
            cy={polar(FACE_R, midAngle(focused, count)).y}
            r={5}
          />
        )}
      </svg>

      {wedges.map((w, i) => {
        const p = polar(LABEL_R, midAngle(i, count));
        const state = stateOf(i, hovered, confirmed);
        const style = { "--i": i, left: p.x, top: p.y } as CSSProperties;
        return (
          <div key={w.id} className="slot" style={style}>
            <div
              className={`label ${state}`}
              onAnimationEnd={(e) => {
                if (state === "chosen" && e.animationName === "clock-chosen") onConfirmed();
              }}
            >
              {w.label}
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
