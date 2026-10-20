import type { PointerEvent as ReactPointerEvent } from "react";
import { Icon } from "./icons";
import { cardLayout } from "./layout";
import { Thumb } from "./Thumb";
import { thumbWidth, type ThumbFetcher } from "./thumbnails";
import type { PageShape, PlanPage } from "./types";
import "./PageCard.css";

interface PageCardProps {
  /** Position in the new PDF, from 1. */
  number: number;
  page: PlanPage;
  shape: PageShape;
  cell: number;
  pixelRatio: number;
  selected: boolean;
  /** Being dragged right now. */
  lifted: boolean;
  load: ThumbFetcher;
  onPress: (event: ReactPointerEvent, id: number) => void;
  onRotate: (id: number) => void;
  onRemove: (id: number) => void;
}

/** One page of the new PDF: its picture, its number, and quick actions on hover. */
export function PageCard({ number, page, shape, cell, pixelRatio, selected, lifted, load, onPress, onRotate, onRemove }: PageCardProps) {
  const layout = cardLayout(shape, page.turn, cell);
  return (
    <div
      className={`page-card ${selected ? "selected" : ""} ${lifted ? "lifted" : ""}`}
      data-page-id={page.id}
      onPointerDown={(event) => onPress(event, page.id)}
      style={{ width: cell + 20 }}
    >
      <div className="sheet" style={{ width: layout.boxWidth, height: layout.boxHeight }}>
        <Thumb
          source={page.source}
          page={page.page}
          width={thumbWidth(cell, pixelRatio)}
          load={load}
          pictureWidth={layout.pictureWidth}
          pictureHeight={layout.pictureHeight}
          turn={page.turn}
        />
        <div className="quick">
          <button type="button" title="Rotate" aria-label={`Rotate page ${number}`} onClick={() => onRotate(page.id)}>
            <Icon name="rotate-right" />
          </button>
          <button type="button" title="Delete" aria-label={`Delete page ${number}`} onClick={() => onRemove(page.id)}>
            <Icon name="trash" />
          </button>
        </div>
      </div>
      <span className="number">{number}</span>
    </div>
  );
}
