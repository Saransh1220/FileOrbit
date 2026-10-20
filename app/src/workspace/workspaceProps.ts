import type { ThumbFetcher } from "./thumbnails";
import type { Modifiers, OpenedFile, PlanPage } from "./types";

/** Where saving stands, shown in the bottom bar. */
export type SaveStatus =
  | { kind: "idle" }
  | { kind: "saving" }
  | { kind: "saved"; name: string; path: string; pages: number }
  | { kind: "error"; message: string };

/** Everything the page editor can be asked to do. The view only calls these. */
export interface WorkspaceActions {
  onChoose: (id: number, mods: Modifiers) => void;
  onSelectAll: () => void;
  onClearSelection: () => void;
  onMove: (ids: number[], beforeIndex: number) => void;
  onRotate: (degrees: number, ids: number[]) => void;
  onRemove: (ids: number[]) => void;
  onDuplicate: (ids: number[]) => void;
  onUndo: () => void;
  onRedo: () => void;
  onCellChange: (cell: number) => void;
  onSave: () => void;
  onShowSaved: (path: string) => void;
}

export interface WorkspaceViewProps extends WorkspaceActions {
  files: OpenedFile[];
  pages: PlanPage[];
  selected: ReadonlySet<number>;
  /** Width of a page card, from the size slider. */
  cell: number;
  canUndo: boolean;
  canRedo: boolean;
  edited: boolean;
  status: SaveStatus;
  /** Why the file could not be opened, when it could not. */
  problem: string | null;
  loading: boolean;
  load: ThumbFetcher;
  pixelRatio: number;
}
