/** The sidebar sections, top to bottom. */
export type SectionId = "converters" | "appearance" | "storage" | "shortcut";

export interface Section {
  id: SectionId;
  label: string;
  /** One line under the page title. */
  blurb: string;
}

export const SECTIONS: Section[] = [
  {
    id: "converters",
    label: "Converters",
    blurb: "Each converter downloads once, when you choose it. Drop a file that needs one and the app offers it too.",
  },
  { id: "appearance", label: "Appearance", blurb: "Choose how the ring, the toast and this window look." },
  { id: "storage", label: "Storage", blurb: "What the converters take up on this computer." },
  { id: "shortcut", label: "Shift+drag", blurb: "How the app notices a file being dragged." },
];

export const DEFAULT_SECTION: SectionId = "converters";

/** The section above or below `current` for an arrow key; other keys and the ends change nothing. */
export function stepSection(current: SectionId, key: string): SectionId {
  const index = SECTIONS.findIndex((s) => s.id === current);
  const step = key === "ArrowDown" ? 1 : key === "ArrowUp" ? -1 : 0;
  const next = SECTIONS[index + step];
  return next ? next.id : current;
}
