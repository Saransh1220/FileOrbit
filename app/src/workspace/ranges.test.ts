import { describe, expect, it } from "vitest";
import { describeGroup, everyN, parseRanges } from "./ranges";

const groups = (r: ReturnType<typeof parseRanges>) => (r.ok ? r.groups : r.error);

describe("page ranges", () => {
  it("reads single pages, ranges and open ends", () => {
    expect(groups(parseRanges("1-3, 5, 8-", 10))).toEqual([[0, 1, 2], [4], [7, 8, 9]]);
  });

  it("allows spaces and a trailing comma", () => {
    expect(groups(parseRanges(" 2 - 3 ,", 5))).toEqual([[1, 2]]);
  });

  it("explains what is wrong", () => {
    expect(groups(parseRanges("", 5))).toMatch(/Type page ranges/);
    expect(groups(parseRanges("1-x", 5))).toMatch(/not a page or a range/);
    expect(groups(parseRanges("0", 5))).toMatch(/start at 1/);
    expect(groups(parseRanges("4-9", 5))).toMatch(/5 pages/);
    expect(groups(parseRanges("4-2", 5))).toMatch(/backwards/);
    expect(groups(parseRanges("-3", 5))).toMatch(/not a page/);
  });

  it("accepts the last page and rejects one past it", () => {
    expect(parseRanges("5", 5).ok).toBe(true);
    expect(parseRanges("6", 5).ok).toBe(false);
  });
});

describe("splitting every N pages", () => {
  it("makes equal parts and a shorter last one", () => {
    expect(groups(everyN(7, 3))).toEqual([[0, 1, 2], [3, 4, 5], [6]]);
  });

  it("refuses a part as big as the file, or a bad size", () => {
    expect(everyN(4, 4).ok).toBe(false);
    expect(everyN(4, 0).ok).toBe(false);
    expect(everyN(4, 1.5).ok).toBe(false);
  });
});

describe("describing a part", () => {
  it("says page or pages", () => {
    expect(describeGroup([4])).toBe("page 5");
    expect(describeGroup([0, 1, 2])).toBe("pages 1-3");
  });
});
