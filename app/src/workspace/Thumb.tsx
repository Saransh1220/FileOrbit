import { useEffect, useRef, useState } from "react";
import type { ThumbFetcher } from "./thumbnails";

interface ThumbProps {
  /** The PDF the page comes from. */
  path: string;
  page: number;
  /** Width to request, in pixels. */
  width: number;
  load: ThumbFetcher;
  /** The picture's upright size on screen. */
  pictureWidth: number;
  pictureHeight: number;
  turn: number;
}

/**
 * One page picture. It asks for its image only once it is near the viewport, so a long
 * file does not draw every page at once, and it keeps showing the old picture while a new
 * size loads.
 */
export function Thumb({ path, page, width, load, pictureWidth, pictureHeight, turn }: ThumbProps) {
  const holder = useRef<HTMLDivElement>(null);
  const [near, setNear] = useState(false);
  const [url, setUrl] = useState<string | null>(null);
  const [failed, setFailed] = useState(false);

  useEffect(() => {
    const element = holder.current;
    if (!element || near) return;
    const watcher = new IntersectionObserver(
      (entries) => {
        if (entries.some((entry) => entry.isIntersecting)) {
          setNear(true);
          watcher.disconnect();
        }
      },
      { rootMargin: "400px" },
    );
    watcher.observe(element);
    return () => watcher.disconnect();
  }, [near]);

  useEffect(() => {
    if (!near) return;
    let current = true;
    load(path, page, width).then(
      (next) => {
        if (current) {
          setUrl(next);
          setFailed(false);
        }
      },
      () => current && setFailed(true),
    );
    return () => {
      current = false;
    };
  }, [near, load, path, page, width]);

  return (
    <div ref={holder} className="picture-holder">
      <div
        className={`picture ${url ? "ready" : ""} ${failed ? "failed" : ""}`}
        style={{ width: pictureWidth, height: pictureHeight, transform: `translate(-50%, -50%) rotate(${turn}deg)` }}
      >
        {url && <img src={url} alt="" draggable={false} />}
        {failed && <span>Cannot draw</span>}
      </div>
    </div>
  );
}
