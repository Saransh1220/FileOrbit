import type { ReactNode } from "react";
import "./Row.css";

/** One setting: what it is (and a hint) on the left, its value or control on the right. */
export function Row({ label, hint, children }: { label: string; hint?: string; children?: ReactNode }) {
  return (
    <div className="row">
      <div className="row-text">
        <span className="row-label">{label}</span>
        {hint && <span className="row-hint">{hint}</span>}
      </div>
      <div className="row-control">{children}</div>
    </div>
  );
}

/** A bordered group of rows, rows separated by hairlines. */
export function RowGroup({ children }: { children: ReactNode }) {
  return <div className="row-group">{children}</div>;
}
