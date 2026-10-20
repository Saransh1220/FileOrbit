import { describe, expect, it } from "vitest";
import { limitedLoader, thumbWidth } from "./thumbnails";

/** A fetcher whose results are released by hand, so the test controls the timing. */
function manualFetcher() {
  const calls: Array<{ key: string; finish: (url: string) => void; fail: (error: Error) => void }> = [];
  const fetchOne = (path: string, page: number, width: number) =>
    new Promise<string>((resolve, reject) => {
      calls.push({ key: `${path}:${page}:${width}`, finish: resolve, fail: reject });
    });
  return { calls, fetchOne };
}

const flush = () => new Promise<void>((resolve) => queueMicrotask(() => resolve()));

describe("the thumbnail loader", () => {
  it("runs only as many fetches at once as allowed, then continues in order", async () => {
    const { calls, fetchOne } = manualFetcher();
    const load = limitedLoader(fetchOne, 2);
    const results = [0, 1, 2, 3].map((page) => load("a.pdf", page, 160));
    expect(calls.map((c) => c.key)).toEqual(["a.pdf:0:160", "a.pdf:1:160"]);

    calls[0].finish("a");
    await flush();
    await flush();
    expect(calls.map((c) => c.key)).toEqual(["a.pdf:0:160", "a.pdf:1:160", "a.pdf:2:160"]);
    calls[1].finish("b");
    calls[2].finish("c");
    await flush();
    await flush();
    calls[3].finish("d");
    expect(await Promise.all(results)).toEqual(["a", "b", "c", "d"]);
  });

  it("asks for the same picture only once", async () => {
    const { calls, fetchOne } = manualFetcher();
    const load = limitedLoader(fetchOne, 3);
    const first = load("a.pdf", 4, 160);
    const second = load("a.pdf", 4, 160);
    expect(first).toBe(second);
    expect(calls).toHaveLength(1);
    calls[0].finish("x");
    expect(await first).toBe("x");
  });

  it("treats another width or file as another picture", () => {
    const { calls, fetchOne } = manualFetcher();
    const load = limitedLoader(fetchOne, 5);
    void load("a.pdf", 0, 160);
    void load("a.pdf", 0, 240);
    void load("b.pdf", 0, 160);
    expect(calls).toHaveLength(3);
  });

  it("lets a failed picture be asked for again, and keeps going", async () => {
    const { calls, fetchOne } = manualFetcher();
    const load = limitedLoader(fetchOne, 1);
    const bad = load("a.pdf", 0, 160);
    const next = load("a.pdf", 1, 160);
    calls[0].fail(new Error("no"));
    await expect(bad).rejects.toThrow("no");
    await flush();
    await flush();
    expect(calls).toHaveLength(2);
    calls[1].finish("ok");
    expect(await next).toBe("ok");
    await flush();
    await flush();
    void load("a.pdf", 0, 160);
    expect(calls).toHaveLength(3);
  });
});

describe("the width to ask for", () => {
  it("rounds up to a step and allows for sharp screens", () => {
    expect(thumbWidth(150, 1)).toBe(160);
    expect(thumbWidth(160, 1)).toBe(160);
    expect(thumbWidth(161, 1)).toBe(240);
    expect(thumbWidth(160, 2)).toBe(320);
  });

  it("never goes below one pixel per pixel", () => {
    expect(thumbWidth(160, 0.5)).toBe(160);
  });
});
