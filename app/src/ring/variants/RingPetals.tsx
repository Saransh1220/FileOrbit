import type { CSSProperties } from "react";
import { RING_SIZE } from "../constants";
import { FormatIcon } from "../FormatIcon";
import { midAngle, polar } from "../geometry";
import { hubText, segmentVars, stateOf } from "../ringState";
import type { RingViewProps } from "../ringViewProps";
import "./RingPetals.css";

const CENTER = RING_SIZE / 2;
const INNER = 46;
const OUTER = 122;
const LABEL_R = 86;

/** A leaf shape pointing up from the origin; wider petals for fewer targets. */
function petalPath(count: number): string {
  const half = Math.min(34, ((Math.PI * LABEL_R) / count) * 0.86);
  const ctrl = half / 0.75;
  return [
    `M 0 ${-INNER}`,
    `C ${ctrl} ${-INNER - 12} ${ctrl} ${-OUTER + 30} 0 ${-OUTER}`,
    `C ${-ctrl} ${-OUTER + 30} ${-ctrl} ${-INNER - 12} 0 ${-INNER}`,
    "Z",
  ].join(" ");
}

/**
 * Petals: soft leaf shapes radiating from the center, one tint per file kind.
 * The hovered petal swells outward and fills with its own color.
 */
export function RingPetals(props: RingViewProps) {
  const { wedges, hovered, confirmed, onConfirmed } = props;
  const count = wedges.length;
  const hub = hubText(props);
  const focused = confirmed ?? hovered;
  const hubStyle = focused === null ? undefined : segmentVars(wedges[focused].id, 0);
  const shape = petalPath(Math.max(count, 3));

  return (
    <div className={`petals${focused === null ? "" : " has-focus"}`} style={{ width: RING_SIZE, height: RING_SIZE }}>
      <svg width={RING_SIZE} height={RING_SIZE} viewBox={`0 0 ${RING_SIZE} ${RING_SIZE}`}>
        {wedges.map((w, i) => {
          const state = stateOf(i, hovered, confirmed);
          const degrees = (midAngle(i, count) * 180) / Math.PI + 90;
          return (
            <g key={w.id} transform={`translate(${CENTER} ${CENTER}) rotate(${degrees.toFixed(2)})`}>
              <g
                className={`petal ${state}`}
                style={segmentVars(w.id, i)}
                onAnimationEnd={(e) => {
                  if (state === "chosen" && e.animationName === "petals-chosen") onConfirmed();
                }}
              >
                <path d={shape} />
              </g>
            </g>
          );
        })}
      </svg>

      {wedges.map((w, i) => {
        const p = polar(LABEL_R, midAngle(i, count));
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
