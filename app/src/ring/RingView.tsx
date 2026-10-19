import type { CSSProperties } from "react";
import { HUB_RADIUS, RING_SIZE } from "./constants";
import { FormatIcon } from "./FormatIcon";
import { contentPoint, segmentPath } from "./geometry";
import { hubText, segmentVars, stateOf } from "./ringState";
import type { RingViewProps } from "./ringViewProps";
import "./Ring.css";

export type { RingViewProps } from "./ringViewProps";

/** The current dial design: flat segments in a donut around a label disc. */
export function RingView(props: RingViewProps) {
  const { wedges, hovered, confirmed, onConfirmed } = props;
  const count = wedges.length;
  const hub = hubText(props);
  const focused = confirmed ?? hovered;
  const hubStyle = focused === null ? undefined : segmentVars(wedges[focused].id, 0);

  return (
    <div className="ring" style={{ width: RING_SIZE, height: RING_SIZE }}>
      <svg width={RING_SIZE} height={RING_SIZE} viewBox={`0 0 ${RING_SIZE} ${RING_SIZE}`}>
        {wedges.map((w, i) => {
          const state = stateOf(i, hovered, confirmed);
          const d = segmentPath(i, count);
          return (
            <g
              key={w.id}
              className={`seg ${state}`}
              style={segmentVars(w.id, i)}
              onAnimationEnd={(e) => {
                if (state === "chosen" && e.animationName === "chosen-pop") onConfirmed();
              }}
            >
              {/* Two stacked paths: the wider one shows as a hairline edge around the face. */}
              <path className="edge" d={d} fillRule="evenodd" />
              <path className="face" d={d} fillRule="evenodd" />
            </g>
          );
        })}
      </svg>

      {wedges.map((w, i) => {
        const p = contentPoint(i, count);
        const style = { ...segmentVars(w.id, i), left: p.x, top: p.y } as CSSProperties;
        return (
          <div key={w.id} className="slot" style={style}>
            <div className={`content ${stateOf(i, hovered, confirmed)}`}>
              <FormatIcon id={w.id} />
              <span className="ext">{w.label}</span>
            </div>
          </div>
        );
      })}

      <div
        className={`hub ${hub.kind}${focused === null ? "" : " lit"}`}
        style={{ ...hubStyle, width: HUB_RADIUS * 2, height: HUB_RADIUS * 2 }}
      >
        {hub.caption && <span className="caption">{hub.caption}</span>}
        <span className="main">{hub.main}</span>
      </div>
    </div>
  );
}
