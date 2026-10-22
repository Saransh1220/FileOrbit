import "./controls.css";

/** An on/off switch; the label names what it turns on for screen readers. */
export function Switch({ label, on, onChange }: { label: string; on: boolean; onChange: (on: boolean) => void }) {
  return (
    <button type="button" role="switch" aria-checked={on} aria-label={label} className="switch" onClick={() => onChange(!on)}>
      <i />
    </button>
  );
}
