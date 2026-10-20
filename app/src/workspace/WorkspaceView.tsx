import { useState } from "react";
import { BottomBar } from "./BottomBar";
import { FileRail } from "./FileRail";
import { PageGrid } from "./PageGrid";
import { SplitPanel } from "./SplitPanel";
import { Toolbar } from "./Toolbar";
import type { WorkspaceViewProps } from "./workspaceProps";
import "./WorkspaceView.css";

const titleOf = (names: string[]) => (names.length === 1 ? names[0] : `${names.length} files`);

/** The page editor, all of it, driven by props so it can be judged in a browser without the app. */
export function WorkspaceView(props: WorkspaceViewProps) {
  const { files, pages, selected } = props;
  const [splitting, setSplitting] = useState(false);
  const chosen = pages.filter((p) => selected.has(p.id)).map((p) => p.id);
  const ready = !props.loading && !props.problem && files.length > 0;

  return (
    <div className="workspace">
      <Toolbar
        title={files.length > 0 ? titleOf(files.map((f) => f.name)) : "Organize pages"}
        canUndo={props.canUndo}
        canRedo={props.canRedo}
        hasSelection={chosen.length > 0}
        cell={props.cell}
        onUndo={props.onUndo}
        onRedo={props.onRedo}
        onRotateLeft={() => props.onRotate(-90, chosen)}
        onRotateRight={() => props.onRotate(90, chosen)}
        onDuplicate={() => props.onDuplicate(chosen)}
        onInsertBlank={() => props.onInsertBlank(chosen)}
        onRemove={() => props.onRemove(chosen)}
        onCellChange={props.onCellChange}
      />

      {props.loading && <p className="notice">Opening the PDF…</p>}
      {props.problem && <p className="notice problem">{props.problem}</p>}
      {ready && (
        <div className="body">
          <FileRail files={files} counts={props.fileCounts} onAdd={props.onAddFiles} onRemove={props.onRemoveFile} />
          <PageGrid
          files={files}
          pages={pages}
          selected={selected}
          cell={props.cell}
          pixelRatio={props.pixelRatio}
          load={props.load}
          onChoose={props.onChoose}
          onClearSelection={props.onClearSelection}
          onMove={props.onMove}
          onRotate={props.onRotate}
          onRemove={props.onRemove}
          />
          {props.dropping && <div className="drop-hint">Drop PDFs to add their pages</div>}
        </div>
      )}

      {splitting && ready && (
        <SplitPanel
          total={pages.length}
          onClose={() => setSplitting(false)}
          onSplit={(groups) => {
            setSplitting(false);
            props.onSplit(groups);
          }}
        />
      )}
      <BottomBar
        total={pages.length}
        selected={chosen.length}
        edited={props.edited}
        status={props.status}
        onSave={props.onSave}
        onExtract={() => props.onExtract(chosen)}
        splitOpen={splitting}
        onToggleSplit={() => setSplitting((open) => !open)}
        onShowSaved={props.onShowSaved}
      />
    </div>
  );
}
