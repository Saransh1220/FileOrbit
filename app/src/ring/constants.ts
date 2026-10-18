/** Must match the overlay window size in tauri.conf.json and overlay.rs. */
export const RING_SIZE = 264;
/** Drawn dial: a donut between these radii. */
export const DIAL_INNER = 54;
export const DIAL_OUTER = 116;
/** Radius of the solid label disc in the middle. */
export const HUB_RADIUS = 44;
/** Hit-testing band: a little more forgiving than the drawn dial. */
export const INNER_RADIUS = 42;
export const OUTER_RADIUS = 132;
/** Constant pixel gap between neighbouring segments (edges are parallel). */
export const SEGMENT_GAP_PX = 6;
/** Rounded corner radius of each segment, in px. */
export const CORNER_PX = 7;
