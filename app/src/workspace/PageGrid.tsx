import { useRef } from "react";
import { PageCard } from "./PageCard";
import { shapeOfPage } from "./layout";
import type { ThumbFetcher } from "./thumbnails";
import type { Modifiers, OpenedFile, PlanPage } from "./types";
import { usePageDrag } from "./usePageDrag";
import "./PageGrid.css";

interface PageGridProps {
  files: OpenedFile[];
  pages: PlanPage[];
  selected: ReadonlySet<number>;
  cell: number;
  pixelRatio: number;
  load: ThumbFetcher;
  onChoose: (id: number, mods: Modifiers) => void;
  onClearSelection: () => void;
  onMove: (ids: number[], beforeIndex: number) => void;
  onRotate: (degrees: number, ids: number[]) => void;
  onRemove: (ids: number[]) => void;
}

/** The pages in order. Click to select, drag to reorder; a line shows where a drop will land. */
export function PageGrid({ files, pages, selected, cell, pixelRatio, load, onChoose, onClearSelection, onMove, onRotate, onRemove }: PageGridProps) {
  const scroller = useRef<HTMLDivElement>(null);
  const grid = useRef<HTMLDivElement>(null);
  const { drag, insertion, press } = usePageDrag({
    grid,
    scroller,
    order: pages.map((p) => p.id),
    selected,
    onChoose,
    onMove,
  });

  return (
    <div ref={scroller} className="grid-scroll" onPointerDown={(event) => event.target === event.currentTarget && onClearSelection()}>
      <div ref={grid} className="grid" onPointerDown={(event) => event.target === event.currentTarget && onClearSelection()}>
        {pages.map((page, index) => (
          <PageCard
            key={page.id}
            number={index + 1}
            page={page}
            shape={shapeOfPage(files, page)}
            path={page.blank ? "" : files[page.source].path}
            tag={files.length > 1 && !page.blank ? page.source : null}
            fileName={page.blank ? "Blank page" : files[page.source].name}
            cell={cell}
            pixelRatio={pixelRatio}
            selected={selected.has(page.id)}
            lifted={drag?.ids.includes(page.id) ?? false}
            load={load}
            onPress={press}
            onRotate={(id) => onRotate(90, [id])}
            onRemove={(id) => onRemove([id])}
          />
        ))}
        {insertion && <div className="insertion" style={{ left: insertion.left - 2, top: insertion.top, height: insertion.height }} />}
      </div>
      {drag && (
        <div className="drag-chip" style={{ left: drag.x + 14, top: drag.y + 14 }}>
          {drag.ids.length === 1 ? "1 page" : `${drag.ids.length} pages`}
        </div>
      )}
    </div>
  );
}
