import type { PageShape, PlanPage, SavedPage } from "./types";

/** The pages of the new PDF plus how to get back: every edit is a new list, so undo is cheap. */
export interface History {
  past: PlanPage[][];
  present: PlanPage[];
  future: PlanPage[][];
  /** The next unused page id. */
  nextId: number;
}

/** Pages of one open file, in their own order. */
export function pagesOfFile(source: number, shapes: PageShape[], firstId: number): PlanPage[] {
  return shapes.map((_, page) => ({ id: firstId + page, source, page, turn: 0 }));
}

export function startHistory(pages: PlanPage[]): History {
  const nextId = pages.reduce((highest, p) => Math.max(highest, p.id), -1) + 1;
  return { past: [], present: pages, future: [], nextId };
}

/** Makes `pages` the present, remembering the old one. A change that changes nothing is not remembered. */
export function commit(history: History, pages: PlanPage[], nextId = history.nextId): History {
  if (pages === history.present) return history;
  return { past: [...history.past, history.present], present: pages, future: [], nextId };
}

export const canUndo = (h: History) => h.past.length > 0;
export const canRedo = (h: History) => h.future.length > 0;

export function undo(history: History): History {
  if (!canUndo(history)) return history;
  const past = history.past.slice(0, -1);
  return { ...history, past, present: history.past[history.past.length - 1], future: [history.present, ...history.future] };
}

export function redo(history: History): History {
  if (!canRedo(history)) return history;
  const [next, ...future] = history.future;
  return { ...history, past: [...history.past, history.present], present: next, future };
}

/** Turns the chosen pages by `degrees` (90 is clockwise, -90 anticlockwise). */
export function rotate(pages: PlanPage[], ids: ReadonlySet<number>, degrees: number): PlanPage[] {
  if (ids.size === 0) return pages;
  return pages.map((p) => (ids.has(p.id) ? { ...p, turn: (((p.turn + degrees) % 360) + 360) % 360 } : p));
}

/** Drops the chosen pages. Refuses to drop them all: a PDF needs a page. */
export function remove(pages: PlanPage[], ids: ReadonlySet<number>): PlanPage[] {
  const kept = pages.filter((p) => !ids.has(p.id));
  return kept.length === 0 || kept.length === pages.length ? pages : kept;
}

/** Puts a copy of each chosen page right after it. Returns the new list and the next free id. */
export function duplicate(pages: PlanPage[], ids: ReadonlySet<number>, nextId: number): { pages: PlanPage[]; nextId: number } {
  if (ids.size === 0) return { pages, nextId };
  const result: PlanPage[] = [];
  let id = nextId;
  for (const p of pages) {
    result.push(p);
    if (ids.has(p.id)) result.push({ ...p, id: id++ });
  }
  return { pages: result, nextId: id };
}

/**
 * Moves the chosen pages (keeping their order) so they sit before the page at `beforeIndex`
 * of the current list; `pages.length` means the end.
 */
export function move(pages: PlanPage[], ids: ReadonlySet<number>, beforeIndex: number): PlanPage[] {
  const moving = pages.filter((p) => ids.has(p.id));
  if (moving.length === 0) return pages;
  // Pages before the drop point that are leaving shift the point up.
  const leaving = pages.slice(0, beforeIndex).filter((p) => ids.has(p.id)).length;
  const rest = pages.filter((p) => !ids.has(p.id));
  const at = Math.max(0, Math.min(rest.length, beforeIndex - leaving));
  const result = [...rest.slice(0, at), ...moving, ...rest.slice(at)];
  const unchanged = result.every((p, i) => p === pages[i]);
  return unchanged ? pages : result;
}

/** What `workspace_save` needs. */
export function toSavedPages(pages: PlanPage[]): SavedPage[] {
  return pages.map(({ source, page, turn }) => ({ source, page, turn }));
}

/** Whether the plan differs from simply "every page of the file, upright, in order". */
export function isEdited(pages: PlanPage[], original: PlanPage[]): boolean {
  return pages.length !== original.length || pages.some((p, i) => p.source !== original[i].source || p.page !== original[i].page || p.turn !== original[i].turn);
}
