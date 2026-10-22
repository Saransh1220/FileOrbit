import { PRESETS } from "./presets";
import "./Panel.css";

/** One-click starting points; each switches on and fills the parts it needs. */
export function PresetBar({ onPick }: { onPick: (id: string) => void }) {
  return (
    <div className="presets">
      <span>Start from</span>
      <div className="chips">
        {PRESETS.map((p) => (
          <button key={p.id} type="button" title={p.hint} onClick={() => onPick(p.id)}>
            {p.label}
          </button>
        ))}
      </div>
    </div>
  );
}
