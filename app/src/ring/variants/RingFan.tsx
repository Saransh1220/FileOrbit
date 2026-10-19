import type { CSSProperties } from "react";
import { RING_SIZE } from "../constants";
import { FormatIcon } from "../FormatIcon";
import { midAngle, polar } from "../geometry";
import { hubText, segmentVars, stateOf } from "../ringState";
import type { RingViewProps } from "../ringViewProps";
import "./RingFan.css";

const CARD_R = 84;

/**
 * Fan: each target is a card laid out like a hand of playing cards, tilted
 * outward. Its icon and label stay upright; the hovered card slides out.
 */
export function RingFan(props: RingViewProps) {
  const { wedges, hovered, confirmed, onConfirmed } = props;
  const count = wedges.length;
  const hub = hubText(props);
  const focused = confirmed ?? hovered;
  const hubStyle = focused === null ? undefined : segmentVars(wedges[focused].id, 0);

  return (
    <div className={`fan${focused === null ? "" : " has-focus"}`} style={{ width: RING_SIZE, height: RING_SIZE }}>
      {wedges.map((w, i) => {
        const angle = midAngle(i, count);
        const p = polar(CARD_R, angle);
        const tilt = (angle * 180) / Math.PI + 90;
        const state = stateOf(i, hovered, confirmed);
        const style = { ...segmentVars(w.id, i), left: p.x, top: p.y, "--tilt": `${tilt.toFixed(2)}deg` } as CSSProperties;
        return (
          <div key={w.id} className="slot" style={style}>
            <div
              className={`card ${state}`}
              onAnimationEnd={(e) => {
                if (state === "chosen" && e.animationName === "fan-chosen") onConfirmed();
              }}
            >
              <div className="face">
                <FormatIcon id={w.id} />
                <span className="ext">{w.label}</span>
              </div>
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
