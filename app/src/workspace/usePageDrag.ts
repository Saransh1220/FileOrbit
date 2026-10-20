import { useCallback, useEffect, useRef, useState, type PointerEvent as ReactPointerEvent, type RefObject } from "react";
import { dropIndex, movedFar } from "./dragMath";
import type { Modifiers, Rect } from "./types";

/** How close to the top or bottom of the list the pointer must be before it scrolls. */
const EDGE_PX = 56;
const MAX_SCROLL_PX_PER_FRAME = 18;

export interface DragState {
  ids: number[];
  /** Pointer position, in the window. */
  x: number;
  y: number;
  /** Index of the page the dragged ones would go before. */
  dropAt: number;
}

/** The thin line that shows where a dragged page will land, in the grid's own coordinates. */
export interface Insertion {
  left: number;
  top: number;
  height: number;
}

interface Options {
  grid: RefObject<HTMLElement | null>;
  scroller: RefObject<HTMLElement | null>;
  /** Page ids in display order. */
  order: readonly number[];
  selected: ReadonlySet<number>;
  onChoose: (id: number, mods: Modifiers) => void;
  onMove: (ids: number[], beforeIndex: number) => void;
}

const modifiersOf = (event: { shiftKey: boolean; ctrlKey: boolean; metaKey: boolean }): Modifiers => ({
  shift: event.shiftKey,
  toggle: event.ctrlKey || event.metaKey,
});

/**
 * Click, Ctrl/Shift-click and drag-to-reorder for the page grid, with pointer events: the
 * browser's own drag and drop does not work while the window accepts dropped files.
 */
export function usePageDrag({ grid, scroller, order, selected, onChoose, onMove }: Options) {
  const [drag, setDrag] = useState<DragState | null>(null);
  const [insertion, setInsertion] = useState<Insertion | null>(null);
  const latest = useRef({ order, selected, onChoose, onMove });
  latest.current = { order, selected, onChoose, onMove };
  const stop = useRef<(() => void) | null>(null);

  const cardRects = useCallback((): Rect[] => {
    const cards = grid.current?.querySelectorAll<HTMLElement>("[data-page-id]") ?? [];
    return Array.from(cards, (card) => {
      const r = card.getBoundingClientRect();
      return { left: r.left, top: r.top, width: r.width, height: r.height };
    });
  }, [grid]);

  const place = useCallback(
    (x: number, y: number, ids: number[]) => {
      const rects = cardRects();
      const dropAt = dropIndex(rects, x, y);
      setDrag({ ids, x, y, dropAt });
      const origin = grid.current?.getBoundingClientRect();
      if (!origin || rects.length === 0) return setInsertion(null);
      const before = dropAt < rects.length ? rects[dropAt] : rects[rects.length - 1];
      const edge = dropAt < rects.length ? before.left : before.left + before.width;
      setInsertion({ left: edge - origin.left, top: before.top - origin.top, height: before.height });
    },
    [cardRects, grid],
  );

  const press = useCallback(
    (event: ReactPointerEvent, id: number) => {
      if (event.button !== 0 || (event.target as HTMLElement).closest("button")) return;
      stop.current?.();
      const start = { x: event.clientX, y: event.clientY };
      const mods = modifiersOf(event);
      let ids: number[] | null = null;
      let pointer = start;
      let frame = 0;

      const edgeScroll = () => {
        const box = scroller.current?.getBoundingClientRect();
        if (box && ids) {
          const fromTop = pointer.y - box.top;
          const fromBottom = box.bottom - pointer.y;
          const push = fromTop < EDGE_PX ? -(1 - fromTop / EDGE_PX) : fromBottom < EDGE_PX ? 1 - fromBottom / EDGE_PX : 0;
          if (push !== 0) {
            scroller.current?.scrollBy(0, Math.round(Math.max(-1, Math.min(1, push)) * MAX_SCROLL_PX_PER_FRAME));
            place(pointer.x, pointer.y, ids);
          }
        }
        frame = requestAnimationFrame(edgeScroll);
      };

      const move = (e: PointerEvent) => {
        pointer = { x: e.clientX, y: e.clientY };
        if (!ids) {
          if (!movedFar(start.x, start.y, e.clientX, e.clientY)) return;
          const now = latest.current;
          if (now.selected.has(id)) {
            ids = now.order.filter((page) => now.selected.has(page));
          } else {
            ids = [id];
            now.onChoose(id, { shift: false, toggle: false });
          }
          frame = requestAnimationFrame(edgeScroll);
        }
        place(e.clientX, e.clientY, ids);
      };

      const finish = () => {
        cancelAnimationFrame(frame);
        window.removeEventListener("pointermove", move);
        window.removeEventListener("pointerup", up);
        window.removeEventListener("keydown", key);
        stop.current = null;
        setDrag(null);
        setInsertion(null);
        return ids;
      };
      const up = (e: PointerEvent) => {
        const moved = finish();
        if (!moved) {
          latest.current.onChoose(id, mods);
          return;
        }
        const index = dropIndex(cardRects(), e.clientX, e.clientY);
        latest.current.onMove(moved, index);
      };
      const key = (e: KeyboardEvent) => {
        if (e.key === "Escape") finish();
      };

      window.addEventListener("pointermove", move);
      window.addEventListener("pointerup", up);
      window.addEventListener("keydown", key);
      stop.current = () => void finish();
    },
    [cardRects, place, scroller],
  );

  // Leaving the window must not leave listeners behind.
  useEffect(() => () => stop.current?.(), []);

  return { drag, insertion, press };
}
