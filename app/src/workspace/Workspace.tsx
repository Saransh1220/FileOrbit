import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { open as chooseFiles } from "@tauri-apps/plugin-dialog";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useDesign } from "../design/useDesign";
import { limitedLoader } from "./thumbnails";
import type { OpenedFile, Saved, SavedPage, SaveKind } from "./types";
import { useWorkspace } from "./useWorkspace";
import { WorkspaceView } from "./WorkspaceView";
import "../manager/ManagerSkins.css";
import "./WorkspaceSkins.css";

/** How many pictures the PDF engine draws at once for the editor. */
const PARALLEL_PICTURES = 3;

const message = (error: unknown) => (typeof error === "string" ? error : error instanceof Error ? error.message : "Something went wrong.");
const isPdf = (path: string) => path.toLowerCase().endsWith(".pdf");

/** The page editor window: opens the files it was given and connects the view to the engine. */
export function Workspace() {
  useDesign();
  const [initial, setInitial] = useState<OpenedFile[]>([]);
  const [problem, setProblem] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [dropping, setDropping] = useState(false);
  const [dropped, setDropped] = useState<string[]>([]);
  const unsaved = useRef(false);

  const openMore = useCallback((paths: string[]) => Promise.all(paths.map((path) => invoke<OpenedFile>("workspace_open", { path }))), []);

  const open = useCallback(
    async (paths: string[]) => {
      if (unsaved.current && !window.confirm("Open another file? The changes you have not saved will be lost.")) return;
      setLoading(true);
      setProblem(null);
      try {
        setInitial(await openMore(paths));
      } catch (error) {
        setInitial([]);
        setProblem(message(error));
      } finally {
        setLoading(false);
      }
    },
    [openMore],
  );

  useEffect(() => {
    // The files were chosen before this window existed, so ask for them; later ones arrive as events.
    void invoke<string[]>("workspace_files").then(open);
    const opened = listen<string[]>("workspace-open", (event) => void open(event.payload));
    return () => void opened.then((stop) => stop());
  }, [open]);

  // PDFs dropped on the window are added to the pages.
  useEffect(() => {
    const watching = getCurrentWebview().onDragDropEvent((event) => {
      const p = event.payload;
      if (p.type === "enter" || p.type === "over") setDropping(true);
      else if (p.type === "leave") setDropping(false);
      else if (p.type === "drop") {
        setDropping(false);
        const pdfs = p.paths.filter(isPdf);
        if (pdfs.length > 0) setDropped(pdfs);
      }
    });
    return () => void watching.then((stop) => stop());
  }, []);

  const load = useMemo(
    () =>
      limitedLoader(async (path, page, width) => {
        const bytes = await invoke<ArrayBuffer>("workspace_thumbnail", { path, page, width });
        return URL.createObjectURL(new Blob([bytes], { type: "image/jpeg" }));
      }, PARALLEL_PICTURES),
    [],
  );

  const pickFiles = useCallback(async () => {
    const chosen = await chooseFiles({ multiple: true, filters: [{ name: "PDF", extensions: ["pdf"] }] });
    return chosen === null ? [] : Array.isArray(chosen) ? chosen : [chosen];
  }, []);

  const save = useCallback(
    (files: string[], pages: SavedPage[], kind: SaveKind, part?: number) => invoke<Saved>("workspace_save", { files, pages, kind, part: part ?? null }),
    [],
  );
  const showInFolder = useCallback((path: string) => void invoke("reveal_in_file_manager", { path }), []);

  const view = useWorkspace({ initial, load, openMore, pickFiles, save, showInFolder, problem, loading, dropping, dropped });
  unsaved.current = view.edited;

  // Closing with edits that were not saved asks first; the window closes itself otherwise.
  useEffect(() => {
    const closing = getCurrentWindow().onCloseRequested((event) => {
      if (unsaved.current && !window.confirm("Close without saving? Your changes will be lost.")) event.preventDefault();
    });
    return () => void closing.then((stop) => stop());
  }, []);

  return <WorkspaceView {...view} />;
}
