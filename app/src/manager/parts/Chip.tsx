import type { ReactNode } from "react";
import type { Tone } from "../hookStatus";
import "./Chip.css";

/** A small pill that names a state: "Installed", "On this computer", "Listening". */
export function Chip({ tone = "neutral", children }: { tone?: Tone; children: ReactNode }) {
  return <span className={`chip ${tone}`}>{children}</span>;
}
