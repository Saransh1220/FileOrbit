import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open as chooseFiles } from "@tauri-apps/plugin-dialog";
import { useCallback, useEffect, useRef, useState } from "react";
import type { ImageFile } from "./types";

/** How large the original's picture is asked for: sharp in the preview, small to send. */
const SHOWN_LONGEST_PX = 900;
const PICTURE_EXTENSIONS = ["png", "jpg", "jpeg", "gif", "webp", "tiff", "tif", "bmp"];

interface Listed {
  pictures: Omit<ImageFile, "previewUrl">[];
  errors: string[];
}

/** The pictures in the window: those it was opened for, those added later, and any that could not be read. */
export function usePictures() {
  const [files, setFiles] = useState<ImageFile[]>([]);
  const [selected, setSelected] = useState(0);
  const [problem, setProblem] = useState<string | null>(null);
  const urls = useRef<string[]>([]);

  const add = useCallback(async (paths: string[], replace: boolean) => {
    if (paths.length === 0) return;
    const listed = await invoke<Listed>("imagetool_info", { paths });
    const loaded = await Promise.all(
      listed.pictures.map(async (picture) => {
        const bytes = await invoke<ArrayBuffer>("imagetool_thumbnail", { path: picture.path, longest: SHOWN_LONGEST_PX });
        const previewUrl = URL.createObjectURL(new Blob([bytes], { type: "image/jpeg" }));
        urls.current.push(previewUrl);
        return { ...picture, previewUrl };
      }),
    );
    setProblem(listed.errors[0] ?? null);
    setFiles((current) => {
      const base = replace ? [] : current.filter((c) => !loaded.some((l) => l.path === c.path));
      return [...base, ...loaded];
    });
    if (replace) setSelected(0);
  }, []);

  useEffect(() => {
    void invoke<string[]>("imagetool_files").then((paths) => add(paths, true));
    const opened = listen<string[]>("imagetool-open", (event) => void add(event.payload, true));
    return () => {
      void opened.then((off) => off());
      urls.current.forEach((u) => URL.revokeObjectURL(u));
    };
  }, [add]);

  const choose = useCallback(async () => {
    const picked = await chooseFiles({ multiple: true, filters: [{ name: "Pictures", extensions: PICTURE_EXTENSIONS }] });
    if (picked) await add(Array.isArray(picked) ? picked : [picked], false);
  }, [add]);

  const remove = useCallback((index: number) => {
    setFiles((current) => current.filter((_, i) => i !== index));
    setSelected((s) => (index < s ? s - 1 : s));
  }, []);

  return { files, selected, setSelected, problem, choose, remove };
}
