import type { DesignChoice } from "./types";

export interface ThemeInfo {
  id: string;
  label: string;
  blurb: string;
}

/** The themes the manager offers. The app may know more; only these are designed for this window. */
export const MANAGER_THEMES: ThemeInfo[] = [
  { id: "current", label: "Current", blurb: "Follows your system's light or dark setting." },
  { id: "bold", label: "Bold", blurb: "Dark graphite with a blue accent." },
  { id: "brutal", label: "Brutal", blurb: "Pale yellow, black outlines, hard edges." },
];

/**
 * The manager's themes that the app also offers. Before the app's list has loaded
 * (empty), all of them are shown rather than a blank page.
 */
export function visibleThemes(offered: DesignChoice[]): ThemeInfo[] {
  if (offered.length === 0) return MANAGER_THEMES;
  const ids = new Set(offered.map(([id]) => id));
  return MANAGER_THEMES.filter((t) => ids.has(t.id));
}
