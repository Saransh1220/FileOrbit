import { Segmented } from "./controls/Segmented";
import { NumberField } from "./controls/NumberField";
import { Slider } from "./controls/Slider";
import type { Settings, SizeUnit } from "./settings";
import type { OutFormat } from "./types";
import "./Panel.css";

type Compress = Settings["compress"];

const UNITS: { value: SizeUnit; label: string }[] = [
  { value: "KB", label: "KB" },
  { value: "MB", label: "MB" },
];

/** The controls of the Compress part. `format` decides which warning applies. */
export function CompressPanel({ value, format, onChange }: { value: Compress; format: OutFormat; onChange: (patch: Partial<Compress>) => void }) {
  return (
    <>
      <Segmented
        label="How to compress"
        value={value.mode}
        choices={[
          { value: "size", label: "File size" },
          { value: "quality", label: "Quality" },
        ]}
        onChange={(mode) => onChange({ mode })}
      />
      {value.mode === "size" ? (
        <>
          <div className="pair size-pair">
            <NumberField label="Stay under" min={1} value={value.sizeValue} onChange={(sizeValue) => onChange({ sizeValue })} />
            <Segmented label="Unit" value={value.sizeUnit} choices={UNITS} onChange={(sizeUnit) => onChange({ sizeUnit })} />
          </div>
          <p className="note">Keeps the best quality that fits. If quality alone is not enough, the picture is made smaller too.</p>
        </>
      ) : (
        <Slider label="Quality" min={1} max={100} value={value.quality} onChange={(quality) => onChange({ quality })} />
      )}
      {format === "png" && value.mode === "quality" && <p className="note warn">PNG has no quality setting. Choose JPG, or a file size.</p>}
    </>
  );
}
