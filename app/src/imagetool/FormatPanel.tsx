import { Segmented } from "./controls/Segmented";
import type { OutFormat } from "./types";
import "./Panel.css";

const FORMATS: { value: OutFormat; label: string }[] = [
  { value: "keep", label: "Keep" },
  { value: "jpg", label: "JPG" },
  { value: "png", label: "PNG" },
];

const HINT: Record<OutFormat, string> = {
  keep: "Same format as the original. GIF, WEBP, TIFF and BMP are saved as JPG, or PNG if they have transparency.",
  jpg: "Smallest for photos. Transparent areas become white.",
  png: "Lossless, keeps transparency. Larger for photos.",
};

/** The controls of the Format part. */
export function FormatPanel({ value, onChange }: { value: OutFormat; onChange: (v: OutFormat) => void }) {
  return (
    <>
      <Segmented label="Output format" value={value} choices={FORMATS} onChange={onChange} />
      <p className="note">{HINT[value]}</p>
    </>
  );
}
