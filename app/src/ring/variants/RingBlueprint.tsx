import type { CSSProperties } from "react";
import { DIAL_INNER, DIAL_OUTER, RING_SIZE } from "../constants";
import { FormatIcon } from "../FormatIcon";
import { contentPoint } from "../geometry";
import { hubText, stateOf } from "../ringState";
import type { RingViewProps } from "../ringViewProps";
import { sectorPath, tickRing } from "./sectorGeometry";
import "./RingBlueprint.css";

const CENTER = RING_SIZE / 2;
/** Angular gap between segments, in radians. */
const GAP = 0.05;
const TICKS = tickRing(DIAL_OUTER + 6, 72, 6, 3, 6);

/**
 * Blueprint: a technical HUD. Outlined segments, a tick scale, monospace
 * type and a single cyan accent on a dark plate, whatever the OS theme.
 */
export function RingBlueprint(props: RingViewProps) {
  const { wedges, hovered, confirmed, onConfirmed } = props;
  const count = wedges.length;
  const hub = hubText(props);
  const focused = confirmed ?? hovered;

  return (
    <div className={`blueprint${focused === null ? "" : " has-focus"}`} style={{ width: RING_SIZE, height: RING_SIZE }}>
      <svg width={RING_SIZE} height={RING_SIZE} viewBox={`0 0 ${RING_SIZE} ${RING_SIZE}`}>
        <circle className="plate" cx={CENTER} cy={CENTER} r={DIAL_OUTER + 14} />
        <path className="ticks" d={TICKS} />
        <circle className="guide" cx={CENTER} cy={CENTER} r={DIAL_INNER - 6} />
        {wedges.map((w, i) => {
          const state = stateOf(i, hovered, confirmed);
          return (
            <path
              key={w.id}
              className={`sector ${state}`}
              style={{ "--i": i } as CSSProperties}
              d={sectorPath(i, count, DIAL_INNER, DIAL_OUTER, GAP)}
              onAnimationEnd={(e) => {
                if (state === "chosen" && e.animationName === "bp-chosen") onConfirmed();
              }}
            />
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
