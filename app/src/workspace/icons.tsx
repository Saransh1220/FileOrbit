/** Small line icons for the editor's buttons. One stroke weight, no fills, so every theme can recolor them. */
const PATHS = {
  undo: "M9 14 4 9l5-5 M4 9h10a6 6 0 0 1 0 12h-3",
  redo: "m15 14 5-5-5-5 M20 9H10a6 6 0 0 0 0 12h3",
  "rotate-left": "M3 12a9 9 0 1 0 3-6.7 M3 4v5h5",
  "rotate-right": "M21 12a9 9 0 1 1-3-6.7 M21 4v5h-5",
  duplicate: "M9 9h11v11H9z M5 15V5a1 1 0 0 1 1-1h10",
  trash: "M4 7h16 M10 11v6 M14 11v6 M6 7l1 13h10l1-13 M9 7V4h6v3",
  save: "M12 4v11 M7 10l5 5 5-5 M5 20h14",
  check: "m5 12 5 5 9-10",
  folder: "M3 7a1 1 0 0 1 1-1h5l2 2h9a1 1 0 0 1 1 1v9a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1z",
} as const;

export type IconName = keyof typeof PATHS;

export function Icon({ name }: { name: IconName }) {
  return (
    <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
      <path d={PATHS[name]} />
    </svg>
  );
}
