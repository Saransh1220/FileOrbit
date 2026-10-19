import type { CSSProperties } from "react";
import { RING_SIZE } from "../constants";
import { midAngle, polar } from "../geometry";
import { hubText, segmentVars, stateOf } from "../ringState";
import type { RingViewProps } from "../ringViewProps";
import "./RingChips.css";

const CENTER = RING_SIZE / 2;
const CHIP_R = 90;

/**
 * Chips: text only. Black pills float around a small black hub; the hovered
 * one turns lime and a thin line ties it to the center. No icons, no wedges.
 */
export function RingChips(props: RingViewProps) {
  const { wedges, hovered, confirmed, onConfirmed } = props;
  const count = wedges.length;
  const hub = hubText(props);
  const focused = confirmed ?? hovered;

  return (
    <div className={`chips${focused === null ? "" : " has-focus"}`} style={{ width: RING_SIZE, height: RING_SIZE }}>
      <svg width={RING_SIZE} height={RING_SIZE} viewBox={`0 0 ${RING_SIZE} ${RING_SIZE}`}>
        <circle className="orbit" cx={CENTER} cy={CENTER} r={CHIP_R} />
        {focused !== null && count > 0 && (
          <line
            className="link"
            x1={CENTER}
            y1={CENTER}
            x2={polar(CHIP_R, midAngle(focused, count)).x}
            y2={polar(CHIP_R, midAngle(focused, count)).y}
          />
        )}
      </svg>

      {wedges.map((w, i) => {
        const p = polar(CHIP_R, midAngle(i, count));
        const state = stateOf(i, hovered, confirmed);
        const style = { ...segmentVars(w.id, i), left: p.x, top: p.y } as CSSProperties;
        return (
          <div key={w.id} className="slot" style={style}>
            <div
              className={`chip ${state}`}
              onAnimationEnd={(e) => {
                if (state === "chosen" && e.animationName === "chips-chosen") onConfirmed();
              }}
            >
              <i className="dot" />
              <span>{w.label}</span>
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
