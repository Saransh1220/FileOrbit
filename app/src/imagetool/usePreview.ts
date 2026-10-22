import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";
import { decodePreview } from "./decodePreview";
import type { Estimate, Job } from "./types";

export interface PreviewState {
  estimate: Estimate | null;
  resultUrl: string | null;
  /** True while the engine is working on the latest request. */
  busy: boolean;
  error: string | null;
}

const NONE: PreviewState = { estimate: null, resultUrl: null, busy: false, error: null };

/**
 * Asks the engine what `job` does to the picture at `path`, always for the latest settings.
 * Only one request runs at a time; while it runs, newer settings replace each other, so
 * dragging a slider never queues a pile of full-size renders (and no timers are needed).
 */
export function usePreview(path: string | null, job: Job | null): PreviewState {
  const [state, setState] = useState<PreviewState>(NONE);
  const wanted = useRef<{ path: string; job: Job } | null>(null);
  const running = useRef(false);
  const url = useRef<string | null>(null);

  const jobKey = JSON.stringify(job);
  useEffect(() => {
    if (!path || !job) {
      wanted.current = null;
      setState(NONE);
      return;
    }
    wanted.current = { path, job };
    if (running.current) return;
    running.current = true;
    void (async () => {
      setState((s) => ({ ...s, busy: true }));
      while (wanted.current) {
        const request = wanted.current;
        wanted.current = null;
        try {
          const decoded = decodePreview(await invoke<ArrayBuffer>("imagetool_preview", request));
          if (url.current) URL.revokeObjectURL(url.current);
          url.current = URL.createObjectURL(decoded.blob);
          if (!wanted.current) setState({ estimate: decoded.estimate, resultUrl: url.current, busy: false, error: null });
        } catch (e) {
          if (!wanted.current) setState({ estimate: null, resultUrl: null, busy: false, error: typeof e === "string" ? e : "Could not make the preview." });
        }
      }
      running.current = false;
    })();
    // jobKey stands for `job`: the effect should run when its contents change, not its identity.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [path, jobKey]);

  useEffect(
    () => () => {
      if (url.current) URL.revokeObjectURL(url.current);
    },
    [],
  );
  return state;
}
