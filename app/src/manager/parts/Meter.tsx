import "./Meter.css";

/** A thin progress bar. `percent` null means "working, no size known": a sliding bar. */
export function Meter({ percent }: { percent: number | null }) {
  return (
    <div className="meter" role="progressbar" aria-valuemin={0} aria-valuemax={100} aria-valuenow={percent ?? undefined}>
      <i className={percent === null ? "indeterminate" : ""} style={percent === null ? undefined : { width: `${percent}%` }} />
    </div>
  );
}
