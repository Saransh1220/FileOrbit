import type { Modifiers } from "./types";

export interface Chosen {
  selected: Set<number>;
  /** Where a Shift range starts from. */
  anchor: number | null;
}

/**
 * The selection after a click on page `id`, the way file managers do it: a plain click picks
 * one page, Ctrl or Cmd adds or removes it, Shift picks everything from the last plain click.
 * `order` is the page ids as displayed.
 */
export function choose(order: readonly number[], current: Chosen, id: number, mods: Modifiers): Chosen {
  if (mods.shift && current.anchor !== null && order.includes(current.anchor)) {
    const from = order.indexOf(current.anchor);
    const to = order.indexOf(id);
    const range = order.slice(Math.min(from, to), Math.max(from, to) + 1);
    const selected = mods.toggle ? new Set([...current.selected, ...range]) : new Set(range);
    return { selected, anchor: current.anchor };
  }
  if (mods.toggle) {
    const selected = new Set(current.selected);
    if (selected.has(id)) selected.delete(id);
    else selected.add(id);
    return { selected, anchor: id };
  }
  return { selected: new Set([id]), anchor: id };
}

/** Keeps only ids that still exist (after a delete or an undo). */
export function prune(selected: ReadonlySet<number>, order: readonly number[]): Set<number> {
  const alive = new Set(order);
  return new Set([...selected].filter((id) => alive.has(id)));
}
