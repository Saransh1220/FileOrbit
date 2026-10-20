import type { CardLayout, PageShape } from "./types";

/** Cards are at most this much taller than wide, so one very tall page cannot stretch a row. */
const MAX_HEIGHT_RATIO = 1.45;

/** Whether a turn swaps the page's width and height. */
export const swapsSides = (turn: number) => turn % 180 !== 0;

/**
 * Fits a page, after its extra turn, into a cell `cell` px wide. The picture is the upright
 * page; the card turns it, so the box and the picture differ for a quarter turn.
 */
export function cardLayout(shape: PageShape, turn: number, cell: number): CardLayout {
  const shownWidth = swapsSides(turn) ? shape.height : shape.width;
  const shownHeight = swapsSides(turn) ? shape.width : shape.height;
  const scale = Math.min(cell / shownWidth, (cell * MAX_HEIGHT_RATIO) / shownHeight);
  return {
    boxWidth: shownWidth * scale,
    boxHeight: shownHeight * scale,
    pictureWidth: shape.width * scale,
    pictureHeight: shape.height * scale,
  };
}
