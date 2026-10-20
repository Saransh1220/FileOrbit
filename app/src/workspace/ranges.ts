/** Page positions (from 0) of each part a split makes. */
export type Groups = number[][];

export type RangeResult = { ok: true; groups: Groups } | { ok: false; error: string };

const span = (from: number, to: number) => Array.from({ length: to - from + 1 }, (_, i) => from + i);

/**
 * Parses "1-3, 5, 8-" for a document of `total` pages into one group per item. Pages count from
 * 1; "8-" runs to the last page; "-3" is not allowed. Each item becomes one file when splitting.
 */
export function parseRanges(text: string, total: number): RangeResult {
  const items = text.split(",").map((item) => item.trim()).filter((item) => item !== "");
  if (items.length === 0) return { ok: false, error: "Type page ranges such as 1-3, 5, 8-" };
  const groups: Groups = [];
  for (const item of items) {
    const match = /^(\d+)\s*(?:(-)\s*(\d*))?$/.exec(item);
    if (!match) return { ok: false, error: `"${item}" is not a page or a range` };
    const from = Number(match[1]);
    const to = match[2] ? (match[3] === "" ? total : Number(match[3])) : from;
    if (from < 1 || to < 1) return { ok: false, error: "Pages start at 1" };
    if (from > total || to > total) return { ok: false, error: `This file has ${total} pages, so "${item}" goes too far` };
    if (to < from) return { ok: false, error: `"${item}" runs backwards` };
    groups.push(span(from - 1, to - 1));
  }
  return { ok: true, groups };
}

/** Parts of `size` pages each; the last one takes what is left. */
export function everyN(total: number, size: number): RangeResult {
  if (!Number.isInteger(size) || size < 1) return { ok: false, error: "Pages per file must be 1 or more" };
  if (size >= total) return { ok: false, error: `This file has ${total} pages, so one file of ${size} would be the whole thing` };
  const groups: Groups = [];
  for (let from = 0; from < total; from += size) groups.push(span(from, Math.min(total, from + size) - 1));
  return { ok: true, groups };
}

/** "pages 1-3", "page 5": how a part reads in the preview. */
export function describeGroup(group: number[]): string {
  const first = group[0] + 1;
  const last = group[group.length - 1] + 1;
  return first === last ? `page ${first}` : `pages ${first}-${last}`;
}
