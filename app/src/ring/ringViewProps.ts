import type { Wedge } from "./types";

/** What every dial design needs: all state comes from props so a design can be previewed without Tauri. */
export interface RingViewProps {
  wedges: Wedge[];
  fileName: string;
  hovered: number | null;
  confirmed: number | null;
  /** Called when the chosen segment's confirmation animation has played. */
  onConfirmed: () => void;
}
