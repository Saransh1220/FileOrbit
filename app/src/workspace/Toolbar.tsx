import { Icon, type IconName } from "./icons";
import "./Toolbar.css";

/** Card width limits for the size slider, in CSS pixels. */
export const MIN_CELL = 96;
export const MAX_CELL = 260;

interface ToolbarProps {
  title: string;
  canUndo: boolean;
  canRedo: boolean;
  /** Whether any page is chosen; most buttons act on the chosen pages. */
  hasSelection: boolean;
  cell: number;
  onUndo: () => void;
  onRedo: () => void;
  onRotateLeft: () => void;
  onRotateRight: () => void;
  onDuplicate: () => void;
  onRemove: () => void;
  onCellChange: (cell: number) => void;
}

function Tool({ icon, label, disabled, onClick }: { icon: IconName; label: string; disabled: boolean; onClick: () => void }) {
  return (
    <button type="button" className="tool" title={label} aria-label={label} disabled={disabled} onClick={onClick}>
      <Icon name={icon} />
    </button>
  );
}

export function Toolbar(props: ToolbarProps) {
  const { hasSelection } = props;
  return (
    <header className="toolbar">
      <h1 title={props.title}>{props.title}</h1>
      <div className="tools">
        <Tool icon="undo" label="Undo (Ctrl+Z)" disabled={!props.canUndo} onClick={props.onUndo} />
        <Tool icon="redo" label="Redo (Ctrl+Y)" disabled={!props.canRedo} onClick={props.onRedo} />
        <span className="divider" />
        <Tool icon="rotate-left" label="Rotate left (Shift+R)" disabled={!hasSelection} onClick={props.onRotateLeft} />
        <Tool icon="rotate-right" label="Rotate right (R)" disabled={!hasSelection} onClick={props.onRotateRight} />
        <Tool icon="duplicate" label="Duplicate (Ctrl+D)" disabled={!hasSelection} onClick={props.onDuplicate} />
        <Tool icon="trash" label="Delete (Del)" disabled={!hasSelection} onClick={props.onRemove} />
      </div>
      <label className="size">
        <span>Size</span>
        <input
          type="range"
          min={MIN_CELL}
          max={MAX_CELL}
          value={props.cell}
          aria-label="Page size"
          onChange={(event) => props.onCellChange(Number(event.target.value))}
        />
      </label>
    </header>
  );
}
