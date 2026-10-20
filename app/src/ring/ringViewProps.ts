import type { Wedge } from "./types";

/** What every dial design needs: all state comes from props so a design can be previewed without Tauri. */
export interface RingViewProps {
  wedges: Wedge[];
  fileName: string;
  hovered: number | null;
  confirmed: number | null;
  /** A drag is over the center disc, where dropping opens the page editor. */
  hub?: boolean;
  /** Called when the chosen segment's confirmation animation has played. */
  onConfirmed: () => void;
}
