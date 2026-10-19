import type { CSSProperties } from "react";
import { RING_SIZE } from "../constants";
import { FormatIcon } from "../FormatIcon";
import { midAngle, polar } from "../geometry";
import { hubText, segmentVars, stateOf } from "../ringState";
import type { RingViewProps } from "../ringViewProps";
import { hexPoints } from "./sectorGeometry";
import "./RingHex.css";

const CELL_R = 86;
const CELL_SIZE = 31;
const HUB_SIZE = 42;

/**
 * Hex: a honeycomb. Each target is a flat-topped hexagon around a hexagonal
 * hub; the hovered cell fills solid and lifts.
 */
export function RingHex(props: RingViewProps) {
  const { wedges, hovered, confirmed, onConfirmed } = props;
  const count = wedges.length;
  const hub = hubText(props);
  const focused = confirmed ?? hovered;
  const hubStyle = focused === null ? undefined : segmentVars(wedges[focused].id, 0);
  const middle = RING_SIZE / 2;

  return (
    <div className={`hex${focused === null ? "" : " has-focus"}`} style={{ width: RING_SIZE, height: RING_SIZE }}>
      <svg width={RING_SIZE} height={RING_SIZE} viewBox={`0 0 ${RING_SIZE} ${RING_SIZE}`}>
        <polygon className="hub-shape" style={hubStyle} points={hexPoints(middle, middle, HUB_SIZE)} />
        {wedges.map((w, i) => {
          const p = polar(CELL_R, midAngle(i, count));
          const state = stateOf(i, hovered, confirmed);
          return (
            <g
              key={w.id}
              className={`cell ${state}`}
              style={{ ...segmentVars(w.id, i), transformOrigin: `${p.x}px ${p.y}px` }}
              onAnimationEnd={(e) => {
                if (state === "chosen" && e.animationName === "hex-chosen") onConfirmed();
              }}
            >
              <polygon points={hexPoints(p.x, p.y, CELL_SIZE)} />
            </g>
          );
        })}
      </svg>

      {wedges.map((w, i) => {
        const p = polar(CELL_R, midAngle(i, count));
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
