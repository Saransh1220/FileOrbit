import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

export const DEFAULT_DESIGN = "current";

/**
 * The active design id. It follows the tray's Theme menu live, and is also
 * written to `<html data-design>` so design-specific CSS (the toast skins) applies.
 */
export function useDesign(): string {
  const [design, setDesign] = useState(DEFAULT_DESIGN);

  useEffect(() => {
    // The choice may have been made before this window existed.
    void invoke<string>("get_design").then(setDesign);
    const unlisten = listen<string>("design-changed", (e) => setDesign(e.payload));
    return () => void unlisten.then((f) => f());
  }, []);

  useEffect(() => {
    document.documentElement.dataset.design = design;
  }, [design]);

  return design;
}
