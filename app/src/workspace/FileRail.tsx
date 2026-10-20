import { tagOf } from "./fileTags";
import { Icon } from "./icons";
import type { OpenedFile } from "./types";
import "./FileRail.css";

interface FileRailProps {
  files: OpenedFile[];
  /** Pages of each file still in the plan. A file with none is hidden (its pages were deleted; undo brings it back). */
  counts: number[];
  onAdd: () => void;
  onRemove: (source: number) => void;
}

const pagesText = (count: number) => (count === 1 ? "1 page" : `${count} pages`);

/** The open files, each with its color tag, how many of its pages are used, and a way to take it out. */
export function FileRail({ files, counts, onAdd, onRemove }: FileRailProps) {
  const shown = files.map((file, source) => ({ file, source })).filter(({ source }) => counts[source] > 0);
  const onlyOne = shown.length <= 1;
  return (
    <aside className="rail" aria-label="Open files">
      <h2>Files</h2>
      <ul>
        {shown.map(({ file, source }) => (
          <li key={file.path} title={file.path}>
            <i className="tag" data-tag={tagOf(source)} />
            <span className="info">
              <span className="name">{file.name}</span>
              <span className="count">
                {pagesText(counts[source])}
                {counts[source] !== file.pages.length && <span> of {file.pages.length}</span>}
              </span>
            </span>
            {!onlyOne && (
              <button type="button" className="remove" title="Take this file out" aria-label={`Remove ${file.name}`} onClick={() => onRemove(source)}>
                <Icon name="close" />
              </button>
            )}
          </li>
        ))}
      </ul>
      <button type="button" className="add" onClick={onAdd}>
        <Icon name="plus" /> Add PDFs
      </button>
      <p className="hint">or drop PDFs on this window</p>
    </aside>
  );
}
