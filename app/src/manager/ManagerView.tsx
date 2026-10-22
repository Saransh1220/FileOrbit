import { useState } from "react";
import { AppearancePage } from "./pages/AppearancePage";
import { ConvertersPage } from "./pages/ConvertersPage";
import { ShortcutPage } from "./pages/ShortcutPage";
import { StoragePage } from "./pages/StoragePage";
import { DEFAULT_SECTION, type SectionId } from "./sections";
import { Sidebar } from "./shell/Sidebar";
import type { DesignChoice, HookStatus, Overview, PackUpdate } from "./types";
import "./skins/tokens.css";
import "./ManagerView.css";

export interface ManagerViewProps {
  status: HookStatus;
  /** null until the first load finishes. */
  overview: Overview | null;
  updates: Record<string, PackUpdate>;
  designs: DesignChoice[];
  design: string;
  /** A problem with the last action, shown above the converter list. */
  problem: string | null;
  /** The page to open first (the app opens on Converters). */
  initialSection?: SectionId;
  onInstall: (pack: string) => void;
  onRemove: (pack: string) => void;
  onClearDownloads: () => void;
  onOpenFolder: () => void;
  onDesign: (id: string) => void;
}

/**
 * The manager window: a sidebar and one page at a time. State comes from props
 * so every state can be previewed in the browser without Tauri.
 */
export function ManagerView(props: ManagerViewProps) {
  const [section, setSection] = useState<SectionId>(props.initialSection ?? DEFAULT_SECTION);

  return (
    <div className="mgr">
      <Sidebar section={section} status={props.status} onSelect={setSection} />
      {/* Keyed by section so each page fades in as it opens. */}
      <main key={section}>
        {section === "converters" && (
          <ConvertersPage overview={props.overview} updates={props.updates} problem={props.problem} onInstall={props.onInstall} onRemove={props.onRemove} />
        )}
        {section === "appearance" && <AppearancePage designs={props.designs} design={props.design} onDesign={props.onDesign} />}
        {section === "storage" && <StoragePage overview={props.overview} onClearDownloads={props.onClearDownloads} onOpenFolder={props.onOpenFolder} />}
        {section === "shortcut" && <ShortcutPage status={props.status} />}
      </main>
    </div>
  );
}
