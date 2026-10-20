import { Icon } from "./icons";
import type { SaveStatus } from "./workspaceProps";
import "./BottomBar.css";

interface BottomBarProps {
  total: number;
  selected: number;
  edited: boolean;
  status: SaveStatus;
  onSave: () => void;
  onExtract: () => void;
  onToggleSplit: () => void;
  splitOpen: boolean;
  onShowSaved: (path: string) => void;
}

const pagesText = (count: number) => (count === 1 ? "1 page" : `${count} pages`);

/** What there is to save, how it went, and the one main button. */
export function BottomBar({ total, selected, edited, status, onSave, onExtract, onToggleSplit, splitOpen, onShowSaved }: BottomBarProps) {
  const saving = status.kind === "saving";
  return (
    <footer className="bottombar">
      <p className="summary">
        {pagesText(total)}
        {selected > 0 && <span> · {selected} selected</span>}
        {!edited && status.kind !== "saved" && <span className="faint"> · no changes yet</span>}
      </p>

      <div className="result" role="status">
        {status.kind === "saved" && (
          <>
            <Icon name="check" />
            <span>
              Saved <strong>{status.name}</strong> ({pagesText(status.pages)})
            </span>
            <button type="button" className="link" onClick={() => onShowSaved(status.path)}>
              <Icon name="folder" /> Show in folder
            </button>
          </>
        )}
        {status.kind === "error" && <span className="error">{status.message}</span>}
      </div>

      <button type="button" className="secondary" disabled={selected === 0 || saving} onClick={onExtract} title="Saves the chosen pages as a separate PDF">
        <Icon name="extract" /> Extract
      </button>
      <button type="button" className={`secondary ${splitOpen ? "on" : ""}`} disabled={total < 2 || saving} onClick={onToggleSplit} title="Cuts the pages into several PDFs">
        <Icon name="scissors" /> Split…
      </button>

      <button type="button" className="primary" disabled={saving || !edited} onClick={onSave} title="Writes a new PDF next to the original (Ctrl+S)">
        <Icon name="save" />
        {saving ? "Saving…" : "Save as new PDF"}
      </button>
    </footer>
  );
}
