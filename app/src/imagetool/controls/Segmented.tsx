import "./controls.css";

export interface Choice<T extends string> {
  value: T;
  label: string;
}

/** A row of mutually exclusive options, like tabs for one setting. */
export function Segmented<T extends string>({ label, value, choices, onChange }: { label: string; value: T; choices: Choice<T>[]; onChange: (v: T) => void }) {
  return (
    <div className="seg" role="radiogroup" aria-label={label}>
      {choices.map((c) => (
        <button key={c.value} type="button" role="radio" aria-checked={c.value === value} className={c.value === value ? "on" : ""} onClick={() => onChange(c.value)}>
          {c.label}
        </button>
      ))}
    </div>
  );
}
