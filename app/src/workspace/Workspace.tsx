import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useDesign } from "../design/useDesign";
import { limitedLoader } from "./thumbnails";
import type { OpenedFile, Saved, SavedPage } from "./types";
import { useWorkspace } from "./useWorkspace";
import { WorkspaceView } from "./WorkspaceView";
import "../manager/ManagerSkins.css";
import "./WorkspaceSkins.css";

/** How many pictures the PDF engine draws at once for the editor. */
const PARALLEL_PICTURES = 3;

const message = (error: unknown) => (typeof error === "string" ? error : error instanceof Error ? error.message : "Something went wrong.");

/** The page editor window: opens the files it was given and connects the view to the engine. */
export function Workspace() {
  useDesign();
  const [files, setFiles] = useState<OpenedFile[]>([]);
  const [problem, setProblem] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const unsaved = useRef(false);

  const open = useCallback(async (paths: string[]) => {
    if (unsaved.current && !window.confirm("Open another file? The changes you have not saved will be lost.")) return;
    setLoading(true);
    setProblem(null);
    try {
      setFiles(await Promise.all(paths.map((path) => invoke<OpenedFile>("workspace_open", { path }))));
    } catch (error) {
      setFiles([]);
      setProblem(message(error));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    // The files were chosen before this window existed, so ask for them; later ones arrive as events.
    void invoke<string[]>("workspace_files").then(open);
    const opened = listen<string[]>("workspace-open", (event) => void open(event.payload));
    return () => void opened.then((stop) => stop());
  }, [open]);

  const load = useMemo(
    () =>
      limitedLoader(async (source, page, width) => {
        const bytes = await invoke<ArrayBuffer>("workspace_thumbnail", { path: files[source].path, page, width });
        return URL.createObjectURL(new Blob([bytes], { type: "image/jpeg" }));
      }, PARALLEL_PICTURES),
    [files],
  );

  const save = useCallback((pages: SavedPage[]) => invoke<Saved>("workspace_save", { files: files.map((f) => f.path), pages }), [files]);
  const showInFolder = useCallback((path: string) => void invoke("reveal_in_file_manager", { path }), []);

  const view = useWorkspace({ files, load, save, showInFolder, problem, loading });
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
