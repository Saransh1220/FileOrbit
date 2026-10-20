import { describe, expect, it } from "vitest";
import { choose, prune } from "./selection";

const order = [10, 11, 12, 13, 14];
const none = { selected: new Set<number>(), anchor: null };
const plain = { shift: false, toggle: false };
const sorted = (set: Set<number>) => [...set].sort((a, b) => a - b);

describe("choosing pages", () => {
  it("a plain click picks one page and starts a range", () => {
    const result = choose(order, { selected: new Set([10, 11]), anchor: 10 }, 12, plain);
    expect(sorted(result.selected)).toEqual([12]);
    expect(result.anchor).toBe(12);
  });

  it("Ctrl adds and removes single pages", () => {
    let result = choose(order, none, 11, { shift: false, toggle: true });
    result = choose(order, result, 13, { shift: false, toggle: true });
    expect(sorted(result.selected)).toEqual([11, 13]);
    result = choose(order, result, 11, { shift: false, toggle: true });
    expect(sorted(result.selected)).toEqual([13]);
  });

  it("Shift picks the range from the last plain click, in either direction", () => {
    const start = choose(order, none, 11, plain);
    expect(sorted(choose(order, start, 13, { shift: true, toggle: false }).selected)).toEqual([11, 12, 13]);
    const backwards = choose(order, choose(order, none, 13, plain), 11, { shift: true, toggle: false });
    expect(sorted(backwards.selected)).toEqual([11, 12, 13]);
  });

  it("Shift with nothing chosen yet just picks the page", () => {
    expect(sorted(choose(order, none, 12, { shift: true, toggle: false }).selected)).toEqual([12]);
  });

  it("Ctrl+Shift adds the range to what is already chosen", () => {
    const start = { selected: new Set([10, 11]), anchor: 11 };
    const result = choose(order, start, 13, { shift: true, toggle: true });
    expect(sorted(result.selected)).toEqual([10, 11, 12, 13]);
  });

  it("forgets pages that no longer exist", () => {
    expect(sorted(prune(new Set([10, 99]), order))).toEqual([10]);
  });
});
