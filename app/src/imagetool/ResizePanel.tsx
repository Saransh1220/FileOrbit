import { Segmented } from "./controls/Segmented";
import { NumberField } from "./controls/NumberField";
import { RATIOS, type ResizeMode, type Settings } from "./settings";
import type { FitMode } from "./types";
import "./Panel.css";

type Resize = Settings["resize"];

const MODES: { value: ResizeMode; label: string }[] = [
  { value: "percent", label: "Percent" },
  { value: "fit", label: "Fit" },
  { value: "longEdge", label: "Long side" },
  { value: "exact", label: "Exact" },
  { value: "ratio", label: "Ratio" },
];

const MODE_HINT: Record<ResizeMode, string> = {
  percent: "Scales the picture by a percentage.",
  fit: "Shrinks to fit inside this box, keeping proportions. Never enlarges.",
  longEdge: "Shrinks so the longer side is this long. Never enlarges.",
  exact: "Makes the picture exactly this size.",
  ratio: "Changes only the shape, keeping the original pixels.",
};

const FITS: { value: FitMode; label: string }[] = [
  { value: "crop", label: "Crop" },
  { value: "pad", label: "Pad" },
  { value: "stretch", label: "Stretch" },
];

/** The controls of the Resize part. */
export function ResizePanel({ value, onChange }: { value: Resize; onChange: (patch: Partial<Resize>) => void }) {
  const sizeBox = (
    <div className="pair">
      <NumberField label="Width" unit="px" min={1} value={value.width} onChange={(width) => onChange({ width })} />
      <NumberField label="Height" unit="px" min={1} value={value.height} onChange={(height) => onChange({ height })} />
    </div>
  );
  return (
    <>
      <Segmented label="How to resize" value={value.mode} choices={MODES} onChange={(mode) => onChange({ mode })} />
      <p className="note">{MODE_HINT[value.mode]}</p>

      {value.mode === "percent" && <NumberField label="Scale" unit="%" min={1} max={1000} value={value.percent} onChange={(percent) => onChange({ percent })} />}
      {value.mode === "fit" && sizeBox}
      {value.mode === "longEdge" && <NumberField label="Longer side" unit="px" min={1} value={value.longEdge} onChange={(longEdge) => onChange({ longEdge })} />}
      {value.mode === "exact" && (
        <>
          {sizeBox}
          <Segmented label="When the shape differs" value={value.fit} choices={FITS} onChange={(fit) => onChange({ fit })} />
        </>
      )}
      {value.mode === "ratio" && (
        <>
          <div className="chips" role="radiogroup" aria-label="Aspect ratio">
            {[...RATIOS.map((r) => r.id), "custom"].map((id) => (
              <button key={id} type="button" role="radio" aria-checked={value.ratio === id} className={value.ratio === id ? "on" : ""} onClick={() => onChange({ ratio: id })}>
                {id === "custom" ? "Custom" : id}
              </button>
            ))}
          </div>
          {value.ratio === "custom" && (
            <div className="pair">
              <NumberField label="Width" min={1} value={value.ratioWidth} onChange={(ratioWidth) => onChange({ ratioWidth })} />
              <NumberField label="Height" min={1} value={value.ratioHeight} onChange={(ratioHeight) => onChange({ ratioHeight })} />
            </div>
          )}
          <Segmented label="When the shape differs" value={value.fit === "pad" ? "pad" : "crop"} choices={FITS.slice(0, 2)} onChange={(fit) => onChange({ fit })} />
        </>
      )}
    </>
  );
}
