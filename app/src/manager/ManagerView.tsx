import { formatBytes } from "./format";
import { STATUS_HELP, STATUS_LABEL } from "./hookStatus";
import { PackRow } from "./PackRow";
import type { DesignChoice, HookStatus, Overview, PackUpdate } from "./types";
import "./ManagerView.css";

export interface ManagerViewProps {
  status: HookStatus;
  /** null until the first load finishes. */
  overview: Overview | null;
  updates: Record<string, PackUpdate>;
  designs: DesignChoice[];
  design: string;
  /** A problem with the last action, shown quietly at the top of the list. */
  problem: string | null;
  onInstall: (pack: string) => void;
  onRemove: (pack: string) => void;
  onClearDownloads: () => void;
  onOpenFolder: () => void;
  onDesign: (id: string) => void;
}

/** One label and value on a hairline, with an optional action at the end. */
function Line({ label, value, action }: { label: string; value: string; action?: React.ReactNode }) {
  return (
    <div className="line">
      <span className="label">{label}</span>
      <span className="value">{value}</span>
      <span className="end">{action}</span>
    </div>
  );
}

/**
 * The manager window. State comes from props so every state can be previewed
 * in the browser without Tauri.
 */
export function ManagerView(props: ManagerViewProps) {
  const { status, overview, updates, designs, design, problem } = props;
  const help = STATUS_HELP[status];

  return (
    <main className="manager">
      <header>
        <h1>Radial Convert</h1>
        <p className={`status ${status}`}>
          <i aria-hidden />
          {STATUS_LABEL[status]}
        </p>
      </header>
      {help && <p className="help">{help}</p>}

      <section>
        <h2>Converters</h2>
        <p className="intro">Hold Shift, drag a file, and drop it on a format. Each converter downloads once, when you choose it.</p>
        {problem && <p className="problem">{problem}</p>}
        {overview === null ? (
          <p className="loading">Loading…</p>
        ) : (
          <ul>
            {overview.packs.map((pack) => (
              <PackRow
                key={pack.id}
                pack={pack}
                update={updates[pack.id]}
                onInstall={() => props.onInstall(pack.id)}
                onRemove={() => props.onRemove(pack.id)}
              />
            ))}
          </ul>
        )}
      </section>

      {overview && (
        <section>
          <h2>Storage</h2>
          <Line label="Installed converters" value={formatBytes(overview.installed_bytes)} />
          <Line
            label="Kept downloads"
            value={formatBytes(overview.cache_bytes)}
            action={
              overview.cache_bytes > 0 && (
                <button className="link" onClick={props.onClearDownloads}>
                  Clear
                </button>
              )
            }
          />
          <Line
            label="Location"
            value={overview.folder}
            action={
              <button className="link" onClick={props.onOpenFolder}>
                Open
              </button>
            }
          />
        </section>
      )}

      <section>
        <h2>Appearance</h2>
        <div className="line">
          <label className="label" htmlFor="design">
            Theme
          </label>
          <span className="value" />
          <span className="end">
            <select id="design" value={design} onChange={(e) => props.onDesign(e.target.value)}>
              {designs.map(([id, label]) => (
                <option key={id} value={id}>
                  {label}
                </option>
              ))}
            </select>
          </span>
        </div>
      </section>
    </main>
  );
}
