import type { CSSProperties } from "react";
import { DIAL_OUTER, RING_SIZE } from "../constants";
import { midAngle, polar } from "../geometry";
import { hubText, stateOf } from "../ringState";
import type { RingViewProps } from "../ringViewProps";
import { tickRing } from "./sectorGeometry";
import "./RingCompass.css";

const CENTER = RING_SIZE / 2;
const FACE_R = 46;
const LABEL_R = 90;
const TICKS = tickRing(DIAL_OUTER - 2, 90, 5, 4, 8);

/**
 * Compass: a matte instrument. Labels sit on a tick scale and a small orange
 * pointer swings to the hovered target. A dark center face shows the readout.
 */
export function RingCompass(props: RingViewProps) {
  const { wedges, hovered, confirmed, onConfirmed } = props;
  const count = wedges.length;
  const hub = hubText(props);
  const focused = confirmed ?? hovered;
  // Pointer angle in degrees clockwise from 12 o'clock; rests pointing up.
  const degrees = focused === null || count === 0 ? 0 : (focused * 360) / count;

  return (
    <div className={`compass${focused === null ? "" : " has-focus"}`} style={{ width: RING_SIZE, height: RING_SIZE }}>
      <svg width={RING_SIZE} height={RING_SIZE} viewBox={`0 0 ${RING_SIZE} ${RING_SIZE}`}>
        <circle className="plate" cx={CENTER} cy={CENTER} r={DIAL_OUTER + 8} />
        <path className="ticks" d={TICKS} />
        <circle className="face" cx={CENTER} cy={CENTER} r={FACE_R} />
        <g className="pointer" style={{ transform: `rotate(${degrees}deg)`, opacity: focused === null ? 0.35 : 1 }}>
          <path d={`M ${CENTER} ${CENTER - FACE_R - 17} L ${CENTER - 7} ${CENTER - FACE_R - 3} L ${CENTER + 7} ${CENTER - FACE_R - 3} Z`} />
        </g>
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
                if (state === "chosen" && e.animationName === "compass-chosen") onConfirmed();
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
