import "./controls.css";

/** A labelled number box with an optional unit after it. Empty or invalid text reads as 0, which the settings reject with a message. */
export function NumberField({ label, value, unit, min = 0, max, onChange }: { label: string; value: number; unit?: string; min?: number; max?: number; onChange: (v: number) => void }) {
  return (
    <label className="num">
      <span>{label}</span>
      <span className="num-box">
        <input type="number" inputMode="decimal" min={min} max={max} value={Number.isFinite(value) ? value : ""} onChange={(e) => onChange(e.target.value === "" ? 0 : Number(e.target.value))} />
        {unit && <em>{unit}</em>}
      </span>
    </label>
  );
}
