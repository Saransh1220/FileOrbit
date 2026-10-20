/** A page's shape in points, as it displays. */
export interface PageShape {
  width: number;
  height: number;
}

/** A file the editor has open (reply of `workspace_open`). */
export interface OpenedFile {
  path: string;
  name: string;
  pages: PageShape[];
}

/** One page of the PDF being built: a reference into an open file, never a copy. */
export interface PlanPage {
  /** Stable while the page lives in the editor; a duplicate gets a new one. */
  id: number;
  /** Index into the open files. */
  source: number;
  /** Page number in that file, from 0. */
  page: number;
  /** Extra clockwise turn: 0, 90, 180 or 270. */
  turn: number;
}

/** What `workspace_save` receives for each page. */
export type SavedPage = Pick<PlanPage, "source" | "page" | "turn">;

/** Reply of `workspace_save`. */
export interface Saved {
  path: string;
  pages: number;
}

/** How a page card is drawn: its box, and the picture inside it. All in CSS pixels. */
export interface CardLayout {
  boxWidth: number;
  boxHeight: number;
  /** The picture is drawn upright at this size, then turned. */
  pictureWidth: number;
  pictureHeight: number;
}

export interface Modifiers {
  shift: boolean;
  toggle: boolean;
}

export interface Rect {
  left: number;
  top: number;
  width: number;
  height: number;
}
