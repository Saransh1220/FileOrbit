import { describe, expect, it } from "vitest";
import { canRedo, canUndo, commit, duplicate, isEdited, move, pagesOfFile, redo, remove, rotate, startHistory, toSavedPages, undo } from "./planModel";
import type { PlanPage } from "./types";

const shape = { width: 595, height: 842 };
const pages = (count: number) => pagesOfFile(0, Array(count).fill(shape), 0);
const ids = (list: PlanPage[]) => list.map((p) => p.page);
const set = (...values: number[]) => new Set(values);

describe("the page list", () => {
  it("starts with every page upright and in order", () => {
    expect(pages(3).map((p) => [p.id, p.page, p.turn])).toEqual([[0, 0, 0], [1, 1, 0], [2, 2, 0]]);
  });

  it("rotates only the chosen pages and wraps around", () => {
    const turned = rotate(pages(3), set(1), 90);
    expect(turned.map((p) => p.turn)).toEqual([0, 90, 0]);
    expect(rotate(turned, set(1), 270)[1].turn).toBe(0);
    expect(rotate(pages(1), set(0), -90)[0].turn).toBe(270);
  });

  it("keeps the page list as it was when nothing is chosen", () => {
    const list = pages(2);
    expect(rotate(list, set(), 90)).toBe(list);
    expect(remove(list, set())).toBe(list);
  });

  it("removes pages but never all of them", () => {
    expect(ids(remove(pages(4), set(1, 2)))).toEqual([0, 3]);
    const list = pages(2);
    expect(remove(list, set(0, 1))).toBe(list);
  });

  it("duplicates a page right after itself with a new id", () => {
    const result = duplicate(pages(3), set(1), 3);
    expect(ids(result.pages)).toEqual([0, 1, 1, 2]);
    expect(result.pages.map((p) => p.id)).toEqual([0, 1, 3, 2]);
    expect(result.nextId).toBe(4);
  });

  it("a duplicate keeps the turn of its original", () => {
    const result = duplicate(rotate(pages(2), set(0), 90), set(0), 2);
    expect(result.pages.map((p) => p.turn)).toEqual([90, 90, 0]);
  });
});

describe("moving pages", () => {
  it("moves one page forward and back", () => {
    expect(ids(move(pages(5), set(0), 3))).toEqual([1, 2, 0, 3, 4]);
    expect(ids(move(pages(5), set(4), 1))).toEqual([0, 4, 1, 2, 3]);
  });

  it("moves to the very end", () => {
    expect(ids(move(pages(3), set(0), 3))).toEqual([1, 2, 0]);
  });

  it("moves several pages together and keeps their order", () => {
    expect(ids(move(pages(6), set(1, 3), 5))).toEqual([0, 2, 4, 1, 3, 5]);
    expect(ids(move(pages(6), set(4, 1), 0))).toEqual([1, 4, 0, 2, 3, 5]);
  });

  it("dropping a page where it already is changes nothing", () => {
    const list = pages(4);
    expect(move(list, set(1), 1)).toBe(list);
    expect(move(list, set(1), 2)).toBe(list);
  });

  it("ignores a move of pages that are not there", () => {
    const list = pages(2);
    expect(move(list, set(99), 0)).toBe(list);
  });
});

describe("undo and redo", () => {
  it("steps back and forward through edits", () => {
    let h = startHistory(pages(3));
    h = commit(h, rotate(h.present, set(0), 90));
    h = commit(h, remove(h.present, set(2)));
    expect(h.present.length).toBe(2);
    h = undo(h);
    expect(h.present.length).toBe(3);
    expect(canRedo(h)).toBe(true);
    h = undo(h);
    expect(h.present[0].turn).toBe(0);
    expect(canUndo(h)).toBe(false);
    h = redo(redo(h));
    expect(h.present.length).toBe(2);
  });

  it("a new edit after an undo drops the redo branch", () => {
    let h = startHistory(pages(3));
    h = commit(h, remove(h.present, set(0)));
    h = undo(h);
    h = commit(h, rotate(h.present, set(1), 90));
    expect(canRedo(h)).toBe(false);
  });

  it("an edit that changes nothing leaves no step to undo", () => {
    const h = startHistory(pages(2));
    expect(canUndo(commit(h, remove(h.present, set())))).toBe(false);
  });

  it("new page ids never repeat after a duplicate is undone", () => {
    let h = startHistory(pages(2));
    const made = duplicate(h.present, set(0), h.nextId);
    h = commit(h, made.pages, made.nextId);
    h = undo(h);
    expect(h.nextId).toBe(3);
  });
});

describe("saving", () => {
  it("sends only what the engine needs", () => {
    expect(toSavedPages(rotate(pages(1), set(0), 90))).toEqual([{ source: 0, page: 0, turn: 90 }]);
  });

  it("knows when the plan differs from the original", () => {
    const original = pages(3);
    expect(isEdited(original, original)).toBe(false);
    expect(isEdited(rotate(original, set(0), 90), original)).toBe(true);
    expect(isEdited(remove(original, set(2)), original)).toBe(true);
    expect(isEdited(move(original, set(0), 2), original)).toBe(true);
  });
});
