import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { useCallback, useEffect, useRef, useState } from "react";
import { onHub, wedgeIndexAt } from "./geometry";
import { dragPositionToCss, IS_MAC } from "./platform";
import { ORGANIZE } from "./ringState";
import type { Point, RingDrag, RingOpened, Wedge } from "./types";

const log = (message: string) => void invoke("log_event", { message });

/**
 * Tracks the OS drag over the overlay window and reports drops on a wedge.
 * `hovered` is the wedge under the pointer (null over the hub or outside);
 * `confirmed` is the wedge that just received a drop, shown until `finish`;
 * `hub` is true while the pointer is over the center and dropping there opens
 * the page editor (only offered when the ring has an Organize wedge).
 */
export function useRingDrag() {
  const [hovered, setHovered] = useState<number | null>(null);
  const [confirmed, setConfirmed] = useState<number | null>(null);
  const [hub, setHub] = useState(false);
  const [fileName, setFileName] = useState<string>("");
  const [wedges, setWedges] = useState<Wedge[]>([]);
  const wedgesRef = useRef<Wedge[]>([]);
  const hoveredRef = useRef<number | null>(null);
  const hubRef = useRef(false);
  const pathsRef = useRef<string[]>([]);

  const finish = useCallback(() => {
    setConfirmed(null);
    void invoke("hide_overlay");
  }, []);

  useEffect(() => {
    const setHover = (i: number | null) => {
      hoveredRef.current = i;
      setHovered(i);
    };

    const setOverHub = (on: boolean) => {
      hubRef.current = on;
      setHub(on);
    };

    // The center disc opens the page editor, but only when this ring has that wedge.
    const hubAt = (css: Point) => wedgesRef.current.some((w) => w.id === ORGANIZE) && onHub(css);

    const hoverAt = (pos: Point) => {
      const css = dragPositionToCss(pos);
      setHover(wedgeIndexAt(css, wedgesRef.current.length));
      setOverHub(hubAt(css));
    };

    const drop = (pos: Point, paths: string[]) => {
      const css = dragPositionToCss(pos);
      const index = wedgeIndexAt(css, wedgesRef.current.length);
      const onCenter = hubAt(css);
      log(`drop at ${pos.x},${pos.y} -> wedge ${index}${onCenter ? " (center)" : ""}`);
      setHover(null);
      setOverHub(false);
      if (index === null) {
        if (onCenter) void invoke("start_conversion", { paths, target: ORGANIZE });
        finish();
        return;
      }
      void invoke("start_conversion", { paths, target: wedgesRef.current[index].id });
      setConfirmed(index);
    };

    // macOS: the webview reports the OS drag. Windows: the native drop target
    // in overlay_drop.rs reports it, because Explorer's drag starts before
    // this window exists.
    const unlistenDrag = IS_MAC
      ? getCurrentWebview().onDragDropEvent((event) => {
          const p = event.payload;
          if (p.type === "enter") {
            setConfirmed(null);
            hoverAt(p.position);
            log(`drag enter at ${p.position.x},${p.position.y}`);
          } else if (p.type === "over") {
            hoverAt(p.position);
          } else if (p.type === "leave") {
            setHover(null);
            setOverHub(false);
            log("drag leave");
          } else if (p.type === "drop") {
            drop(p.position, p.paths);
          }
        })
      : listen<RingDrag>("ring-drag", (event) => {
          const { kind, x, y } = event.payload;
          if (kind === "enter") {
            setConfirmed(null);
            hoverAt({ x, y });
          } else if (kind === "over") {
            hoverAt({ x, y });
          } else if (kind === "leave") {
            setHover(null);
            setOverHub(false);
          } else {
            drop({ x, y }, pathsRef.current);
          }
        });

    // Button released while the pointer was not over a wedge: no drop is
    // coming, so close. If a wedge (or the center) is hovered, the drop event closes us.
    const unlistenEnded = listen("drag-ended", () => {
      log(`drag ended, hovered=${hoveredRef.current} hub=${hubRef.current}`);
      if (hoveredRef.current === null && !hubRef.current) finish();
    });

    const unlistenOpened = listen<RingOpened>("ring-opened", (e) => {
      wedgesRef.current = e.payload.targets;
      pathsRef.current = e.payload.paths;
      setWedges(e.payload.targets);
      setFileName(e.payload.name);
      setConfirmed(null);
      setHover(null);
      setOverHub(false);
      log(`ring opened for ${e.payload.name}: ${e.payload.targets.length} wedges`);
    });

    return () => {
      void unlistenDrag.then((f) => f());
      void unlistenEnded.then((f) => f());
      void unlistenOpened.then((f) => f());
    };
  }, [finish]);

  return { hovered, confirmed, hub, fileName, wedges, finish };
}
