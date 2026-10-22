import { formatBytes } from "../manager/format";
import type { ImageFile } from "./types";
import "./FileList.css";

/** The pictures being worked on; the chosen one drives the preview. */
export function FileList({ files, selected, onSelect, onRemove, onAdd }: { files: ImageFile[]; selected: number; onSelect: (i: number) => void; onRemove: (i: number) => void; onAdd: () => void }) {
  return (
    <aside className="files" aria-label="Pictures">
      <header>
        <h2>Pictures</h2>
        <button type="button" className="add" onClick={onAdd}>
          Add…
        </button>
      </header>
      <ul>
        {files.map((file, i) => (
          <li key={file.path} className={i === selected ? "on" : ""}>
            <button type="button" className="pick" onClick={() => onSelect(i)} aria-current={i === selected}>
              <img src={file.previewUrl} alt="" />
              <span className="meta">
                <strong title={file.name}>{file.name}</strong>
                <small>
                  {file.width} × {file.height} · {formatBytes(file.bytes)}
                </small>
              </span>
            </button>
            {files.length > 1 && (
              <button type="button" className="drop" aria-label={`Remove ${file.name}`} onClick={() => onRemove(i)}>
                ×
              </button>
            )}
          </li>
        ))}
      </ul>
    </aside>
  );
}
