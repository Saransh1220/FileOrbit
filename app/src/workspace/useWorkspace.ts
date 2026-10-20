import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  append,
  canRedo,
  canUndo,
  chosenPages,
  commit,
  countsBySource,
  duplicate,
  insertBlank,
  isEdited,
  move,
  pagesOfFile,
  redo,
  remove,
  removeFile,
  rotate,
  startHistory,
  toSavedPages,
  undo,
  type History,
} from "./planModel";
import { shapeOfPage, swapsSides } from "./layout";
import type { Groups } from "./ranges";
import { choose, prune, type Chosen } from "./selection";
import type { ThumbFetcher } from "./thumbnails";
import type { Modifiers, OpenedFile, PlanPage, Saved, SavedPage, SaveKind } from "./types";
import type { SaveStatus, WorkspaceViewProps } from "./workspaceProps";

export const DEFAULT_CELL = 160;

/** What the editor needs from outside: pictures, files, saving. */
export interface WorkspaceSource {
  /** The files the session starts with; a different list starts a new session. */
  initial: OpenedFile[];
  load: ThumbFetcher;
  /** Opens more PDFs (rejects with a sentence when one cannot be opened). */
  openMore: (paths: string[]) => Promise<OpenedFile[]>;
  /** Lets the user pick PDFs; resolves to nothing when they cancel. */
  pickFiles: () => Promise<string[]>;
  save: (files: string[], pages: SavedPage[], kind: SaveKind, part?: number) => Promise<Saved>;
  showInFolder: (path: string) => void;
  problem: string | null;
  loading: boolean;
  dropping: boolean;
  /** Paths dropped on the window; the editor adds them. */
  dropped: string[];
}

/** The pages of every open file, one file after another, with their first ids. */
function everyPage(files: OpenedFile[]): PlanPage[] {
  const pages: PlanPage[] = [];
  files.forEach((file, source) => pages.push(...pagesOfFile(source, file.pages, pages.length)));
  return pages;
}

const baseName = (path: string) => path.split(/[\\/]/).pop() ?? path;
const message = (error: unknown) => (typeof error === "string" ? error : error instanceof Error ? error.message : "Something went wrong.");
const filesText = (count: number) => (count === 1 ? "1 file" : `${count} files`);

/** All the editor's state and behaviour; the view only draws it. */
export function useWorkspace(source: WorkspaceSource): WorkspaceViewProps {
  const original = useMemo(() => everyPage(source.initial), [source.initial]);
  const [files, setFiles] = useState<OpenedFile[]>(source.initial);
  const [history, setHistory] = useState<History>(() => startHistory(original));
  const [chosen, setChosen] = useState<Chosen>({ selected: new Set(), anchor: null });
  const [cell, setCell] = useState(DEFAULT_CELL);
  const [status, setStatus] = useState<SaveStatus>({ kind: "idle" });
  const [saved, setSaved] = useState<PlanPage[] | null>(null);
  const filesRef = useRef(files);
  filesRef.current = files;

  // A different set of files starts a new editing session.
  useEffect(() => {
    setFiles(source.initial);
    setHistory(startHistory(original));
    setChosen({ selected: new Set(), anchor: null });
    setStatus({ kind: "idle" });
    setSaved(null);
  }, [source.initial, original]);

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

  /** Opens PDFs that are not open yet and adds their pages at the end. */
  const addPaths = useCallback(
    async (paths: string[]) => {
      const fresh = paths.filter((path) => !filesRef.current.some((f) => f.path === path));
      if (fresh.length === 0) return;
      try {
        const opened = await source.openMore(fresh);
        const base = filesRef.current.length;
        filesRef.current = [...filesRef.current, ...opened];
        setFiles(filesRef.current);
        setHistory((h) => {
          let nextId = h.nextId;
          const more: PlanPage[] = [];
          opened.forEach((file, i) => {
            more.push(...pagesOfFile(base + i, file.pages, nextId));
            nextId += file.pages.length;
          });
          return commit(h, append(h.present, more), nextId);
        });
        setStatus({ kind: "idle" });
      } catch (error) {
        setStatus({ kind: "error", message: message(error) });
      }
    },
    [source],
  );

  // PDFs dropped on the window are added as they arrive.
  useEffect(() => {
    if (source.dropped.length > 0) void addPaths(source.dropped);
  }, [source.dropped, addPaths]);

  const idsOf = (list: number[]) => new Set(list);

  const actions = {
    onChoose: (id: number, mods: Modifiers) => setChosen((c) => choose(order, c, id, mods)),
    onSelectAll: () => setChosen((c) => ({ selected: new Set(order), anchor: c.anchor })),
    onClearSelection: () => setChosen({ selected: new Set(), anchor: null }),
    onMove: (ids: number[], beforeIndex: number) => edit(move(pages, idsOf(ids), beforeIndex)),
    onRotate: (degrees: number, ids: number[]) => edit(rotate(pages, idsOf(ids), degrees)),
    onRemove: (ids: number[]) => edit(remove(pages, idsOf(ids))),
    onDuplicate: (ids: number[]) => {
      const made = duplicate(pages, idsOf(ids), history.nextId);
      edit(made.pages, made.nextId);
    },
    onInsertBlank: (ids: number[]) => {
      // The blank page matches its neighbour: the last chosen page, or else the last page.
      const reference = pages.filter((p) => ids.includes(p.id)).pop() ?? pages[pages.length - 1];
      const shape = shapeOfPage(filesRef.current, reference);
      const upright: [number, number] = swapsSides(reference.turn) ? [shape.height, shape.width] : [shape.width, shape.height];
      const made = insertBlank(pages, idsOf(ids), upright, history.nextId);
      edit(made.pages, made.nextId);
    },
    onUndo: () => setHistory(undo),
    onRedo: () => setHistory(redo),
    onCellChange: setCell,
    onRemoveFile: (sourceIndex: number) => edit(removeFile(pages, sourceIndex)),
    onAddFiles: () => void source.pickFiles().then(addPaths),
  };

  const edited = isEdited(pages, original) && pages !== saved;
  const paths = files.map((f) => f.path);

  const onSave = useCallback(async () => {
    if (!edited || status.kind === "saving") return;
    setStatus({ kind: "saving" });
    const snapshot = pages;
    try {
      const result = await source.save(paths, toSavedPages(snapshot), "edited");
      setSaved(snapshot);
      setStatus({ kind: "saved", name: baseName(result.path), path: result.path, pages: result.pages });
    } catch (error) {
      setStatus({ kind: "error", message: message(error) });
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [edited, pages, source, status.kind, files]);

  const onExtract = useCallback(
    async (ids: number[]) => {
      const picked = chosenPages(pages, new Set(ids));
      if (picked.length === 0 || status.kind === "saving") return;
      setStatus({ kind: "saving" });
      try {
        const result = await source.save(paths, toSavedPages(picked), "extract");
        setStatus({ kind: "saved", name: baseName(result.path), path: result.path, pages: result.pages });
      } catch (error) {
        setStatus({ kind: "error", message: message(error) });
      }
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [pages, source, status.kind, files],
  );

  /** Saves each group of pages as its own file ("name (part 1).pdf", ...). */
  const onSplit = useCallback(
    async (groups: Groups) => {
      if (groups.length === 0 || status.kind === "saving") return;
      setStatus({ kind: "saving" });
      try {
        const results: Saved[] = [];
        for (const [index, group] of groups.entries()) {
          results.push(await source.save(paths, toSavedPages(group.map((position) => pages[position])), "edited", index + 1));
        }
        const total = results.reduce((sum, r) => sum + r.pages, 0);
        setStatus({ kind: "saved", name: filesText(results.length), path: results[0].path, pages: total });
      } catch (error) {
        setStatus({ kind: "error", message: message(error) });
      }
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [pages, source, status.kind, files],
  );

  // Keyboard: delete, undo/redo, rotate, select all, duplicate, save.
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      const tag = (event.target as HTMLElement | null)?.tagName;
      if (tag === "INPUT" || tag === "SELECT" || tag === "TEXTAREA") return;
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
    fileCounts: countsBySource(pages, files.length),
    cell,
    canUndo: canUndo(history),
    canRedo: canRedo(history),
    edited,
    status,
    problem: source.problem,
    loading: source.loading,
    dropping: source.dropping,
    load: source.load,
    pixelRatio: typeof window === "undefined" ? 1 : window.devicePixelRatio || 1,
    onSave: () => void onSave(),
    onExtract: (ids: number[]) => void onExtract(ids),
    onSplit: (groups: Groups) => void onSplit(groups),
    onShowSaved: source.showInFolder,
  };
}
