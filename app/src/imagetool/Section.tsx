import type { ReactNode } from "react";
import { Switch } from "./controls/Switch";
import "./Section.css";

/** One part of the job (Resize, Compress, Format) with its own switch; its controls show only while it is on. */
export function Section({ title, hint, on, onToggle, children }: { title: string; hint: string; on: boolean; onToggle?: (on: boolean) => void; children: ReactNode }) {
  return (
    <section className={`it-section ${on ? "on" : ""}`}>
      <header>
        <div>
          <h2>{title}</h2>
          <p>{hint}</p>
        </div>
        {onToggle && <Switch label={`Use ${title.toLowerCase()}`} on={on} onChange={onToggle} />}
      </header>
      {on && <div className="it-body">{children}</div>}
    </section>
  );
}
