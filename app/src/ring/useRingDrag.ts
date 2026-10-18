import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { useCallback, useEffect, useRef, useState } from "react";
import { wedgeIndexAt } from "./geometry";
import { dragPositionToCss } from "./platform";
import type { RingOpened, Wedge } from "./types";

const log = (message: string) => void invoke("log_event", { message });

/**
 * Tracks the OS drag over the overlay window and reports drops on a wedge.
 * `hovered` is the wedge under the pointer (null over the hub or outside);
 * `confirmed` is the wedge that just received a drop, shown until `finish`.
 */
export function useRingDrag() {
  const [hovered, setHovered] = useState<number | null>(null);
  const [confirmed, setConfirmed] = useState<number | null>(null);
  const [fileName, setFileName] = useState<string>("");
  const [wedges, setWedges] = useState<Wedge[]>([]);
  const wedgesRef = useRef<Wedge[]>([]);
  const hoveredRef = useRef<number | null>(null);

  const finish = useCallback(() => {
    setConfirmed(null);
    void invoke("hide_overlay");
  }, []);

  useEffect(() => {
    const setHover = (i: number | null) => {
      hoveredRef.current = i;
      setHovered(i);
    };

    const unlistenDrag = getCurrentWebview().onDragDropEvent((event) => {
      const p = event.payload;
      if (p.type === "enter") {
        setConfirmed(null);
        setHover(wedgeIndexAt(dragPositionToCss(p.position), wedgesRef.current.length));
        log(`drag enter at ${p.position.x},${p.position.y}`);
      } else if (p.type === "over") {
        setHover(wedgeIndexAt(dragPositionToCss(p.position), wedgesRef.current.length));
      } else if (p.type === "leave") {
        setHover(null);
        log("drag leave");
      } else if (p.type === "drop") {
        const index = wedgeIndexAt(dragPositionToCss(p.position), wedgesRef.current.length);
        log(`drop at ${p.position.x},${p.position.y} -> wedge ${index}`);
        setHover(null);
        if (index === null) {
          finish();
          return;
        }
        void invoke("start_conversion", {
          paths: p.paths,
          target: wedgesRef.current[index].id,
        });
        setConfirmed(index);
      }
    });

    // Button released while the pointer was not over a wedge: no drop is
    // coming, so close. If a wedge is hovered, the drop event closes us.
    const unlistenEnded = listen("drag-ended", () => {
      log(`drag ended, hovered=${hoveredRef.current}`);
      if (hoveredRef.current === null) finish();
    });

    const unlistenOpened = listen<RingOpened>("ring-opened", (e) => {
      wedgesRef.current = e.payload.targets;
      setWedges(e.payload.targets);
      setFileName(e.payload.name);
      setConfirmed(null);
      setHover(null);
      log(`ring opened for ${e.payload.name}: ${e.payload.targets.length} wedges`);
    });

    return () => {
      void unlistenDrag.then((f) => f());
      void unlistenEnded.then((f) => f());
      void unlistenOpened.then((f) => f());
    };
  }, [finish]);

  return { hovered, confirmed, fileName, wedges, finish };
}
