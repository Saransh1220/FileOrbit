import { useCallback, useEffect, useMemo, useState } from "react";
import { canRedo, canUndo, commit, duplicate, isEdited, move, pagesOfFile, redo, remove, rotate, startHistory, toSavedPages, undo, type History } from "./planModel";
import { choose, prune, type Chosen } from "./selection";
import type { ThumbFetcher } from "./thumbnails";
import type { Modifiers, OpenedFile, PlanPage, Saved, SavedPage } from "./types";
import type { SaveStatus, WorkspaceViewProps } from "./workspaceProps";

export const DEFAULT_CELL = 160;

/** What the editor needs from outside: pictures, saving, and the files themselves. */
export interface WorkspaceSource {
  files: OpenedFile[];
  load: ThumbFetcher;
  save: (pages: SavedPage[]) => Promise<Saved>;
  showInFolder: (path: string) => void;
  problem: string | null;
  loading: boolean;
}

/** The pages of every open file, one file after another, with their first ids. */
function everyPage(files: OpenedFile[]): PlanPage[] {
  const pages: PlanPage[] = [];
  files.forEach((file, source) => pages.push(...pagesOfFile(source, file.pages, pages.length)));
  return pages;
}

const baseName = (path: string) => path.split(/[\\/]/).pop() ?? path;
const message = (error: unknown) => (typeof error === "string" ? error : error instanceof Error ? error.message : "Something went wrong.");

/** All the editor's state and behaviour; the view only draws it. */
export function useWorkspace(source: WorkspaceSource): WorkspaceViewProps {
  const { files } = source;
  const original = useMemo(() => everyPage(files), [files]);
  const [history, setHistory] = useState<History>(() => startHistory(original));
  const [chosen, setChosen] = useState<Chosen>({ selected: new Set(), anchor: null });
  const [cell, setCell] = useState(DEFAULT_CELL);
  const [status, setStatus] = useState<SaveStatus>({ kind: "idle" });
  const [saved, setSaved] = useState<PlanPage[] | null>(null);

  // A different set of files starts a new editing session.
  useEffect(() => {
    setHistory(startHistory(original));
    setChosen({ selected: new Set(), anchor: null });
    setStatus({ kind: "idle" });
    setSaved(null);
  }, [original]);

  const pages = history.present;
  const order = useMemo(() => pages.map((p) => p.id), [pages]);

  const edit = useCallback((next: PlanPage[], nextId?: number) => {
    setHistory((h) => commit(h, next, nextId));
    setStatus((s) => (s.kind === "saved" || s.kind === "error" ? { kind: "idle" } : s));
  }, []);

  // After any change of the list, drop selected pages that are gone.
  useEffect(() => {
    setChosen((c) => {
      const kept = prune(c.selected, order);
      return kept.size === c.selected.size ? c : { selected: kept, anchor: c.anchor };
    });
  }, [order]);

  const actions = {
    onChoose: (id: number, mods: Modifiers) => setChosen((c) => choose(order, c, id, mods)),
    onSelectAll: () => setChosen((c) => ({ selected: new Set(order), anchor: c.anchor })),
    onClearSelection: () => setChosen({ selected: new Set(), anchor: null }),
    onMove: (ids: number[], beforeIndex: number) => edit(move(pages, new Set(ids), beforeIndex)),
    onRotate: (degrees: number, ids: number[]) => edit(rotate(pages, new Set(ids), degrees)),
    onRemove: (ids: number[]) => edit(remove(pages, new Set(ids))),
    onDuplicate: (ids: number[]) => {
      const made = duplicate(pages, new Set(ids), history.nextId);
      edit(made.pages, made.nextId);
    },
    onUndo: () => setHistory(undo),
    onRedo: () => setHistory(redo),
    onCellChange: setCell,
  };

  const edited = isEdited(pages, original) && pages !== saved;

  const onSave = useCallback(async () => {
    if (!edited || status.kind === "saving") return;
    setStatus({ kind: "saving" });
    const snapshot = pages;
    try {
      const result = await source.save(toSavedPages(snapshot));
      setSaved(snapshot);
      setStatus({ kind: "saved", name: baseName(result.path), path: result.path, pages: result.pages });
    } catch (error) {
      setStatus({ kind: "error", message: message(error) });
    }
  }, [edited, pages, source, status.kind]);

  // Keyboard: delete, undo/redo, rotate, select all, duplicate, save.
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if ((event.target as HTMLElement | null)?.tagName === "INPUT") return;
      const ids = pages.filter((p) => chosen.selected.has(p.id)).map((p) => p.id);
      const command = event.ctrlKey || event.metaKey;
      const key = event.key.toLowerCase();
      if (command && key === "z") {
        setHistory(event.shiftKey ? redo : undo);
      } else if (command && key === "y") {
        setHistory(redo);
      } else if (command && key === "a") {
        actions.onSelectAll();
      } else if (command && key === "d") {
        actions.onDuplicate(ids);
      } else if (command && key === "s") {
        void onSave();
      } else if (!command && (key === "delete" || key === "backspace")) {
        actions.onRemove(ids);
      } else if (!command && key === "r") {
        actions.onRotate(event.shiftKey ? -90 : 90, ids);
      } else if (key === "escape") {
        actions.onClearSelection();
      } else {
        return;
      }
      event.preventDefault();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });

  return {
    ...actions,
    files,
    pages,
    selected: chosen.selected,
    cell,
    canUndo: canUndo(history),
    canRedo: canRedo(history),
    edited,
    status,
    problem: source.problem,
    loading: source.loading,
    load: source.load,
    pixelRatio: typeof window === "undefined" ? 1 : window.devicePixelRatio || 1,
    onSave: () => void onSave(),
    onShowSaved: source.showInFolder,
  };
}
