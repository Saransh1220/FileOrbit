export interface Wedge {
  id: string;
  label: string;
  /** An action on the whole selection (Merge), not a format to convert to. */
  tool?: boolean;
}

export interface Point {
  x: number;
  y: number;
}

/** Payload of the "ring-opened" event sent by the Rust overlay module. */
export interface RingOpened {
  name: string;
  targets: Wedge[];
  /** The dragged files; needed on Windows, where the drop event carries no paths. */
  paths: string[];
}

/** Payload of the "ring-drag" event (Windows): pointer in physical pixels, overlay-relative. */
export interface RingDrag {
  kind: "enter" | "over" | "leave" | "drop";
  x: number;
  y: number;
}
