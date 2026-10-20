/** Fetches one page picture and returns a URL the page can show. */
export type ThumbFetcher = (source: number, page: number, width: number) => Promise<string>;

/**
 * Wraps a fetcher so each picture is fetched once (asking again returns the same promise)
 * and only `limit` fetches run at the same time: scrolling past a 500-page file asks for
 * pictures faster than the PDF engine can draw them.
 */
export function limitedLoader(fetchOne: ThumbFetcher, limit: number): ThumbFetcher {
  const cache = new Map<string, Promise<string>>();
  const waiting: Array<() => void> = [];
  let running = 0;

  const pump = () => {
    while (running < limit && waiting.length > 0) {
      running += 1;
      waiting.shift()?.();
    }
  };

  return (source, page, width) => {
    const key = `${source}:${page}:${width}`;
    const known = cache.get(key);
    if (known) return known;
    const promise = new Promise<string>((resolve, reject) => {
      waiting.push(() => {
        fetchOne(source, page, width)
          .then(resolve, reject)
          .finally(() => {
            running -= 1;
            pump();
          });
      });
      pump();
    });
    cache.set(key, promise);
    // A failed picture may be asked for again later.
    promise.catch(() => cache.delete(key));
    return promise;
  };
}

/** Picture widths are rounded up to these steps, so the slider does not ask for a new size at every pixel. */
const STEP_PX = 80;

/** The width to request for a card `cell` CSS px wide on a screen with `pixelRatio`. */
export function thumbWidth(cell: number, pixelRatio: number): number {
  return Math.ceil((cell * Math.max(1, pixelRatio)) / STEP_PX) * STEP_PX;
}
