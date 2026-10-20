import { useState } from "react";
import { formatBytes } from "./format";
import type { PackInfo, PackUpdate } from "./types";
import "./PackRow.css";

export interface PackRowProps {
  pack: PackInfo;
  /** Live progress or the last failure for this pack, if any. */
  update: PackUpdate | undefined;
  onInstall: () => void;
  onRemove: () => void;
}

/** The programs a pack uses, e.g. "LibreOffice 26.8.1". */
function programs(pack: PackInfo): string {
  return pack.engines.map((e) => (e.version && e.source === "downloaded" ? `${e.name} ${e.version}` : e.name)).join(" + ");
}

/** The quiet line under the description. */
function details(pack: PackInfo): string {
  const parts = [programs(pack)];
  if (pack.installed) {
    const system = pack.engines.some((e) => e.source === "system");
    parts.push(system ? "using the copy already on this computer" : `${formatBytes(pack.disk_bytes)} on disk`);
  } else if (pack.download_bytes === null) {
    parts.push("no download available for this system");
  } else {
    parts.push(`${formatBytes(pack.download_bytes)} download`);
  }
  if (pack.shared_with.length > 0) parts.push(`shared with ${pack.shared_with.join(" and ")}`);
  return parts.join(" · ");
}

/** One converter pack: what it does, what it needs, and its single action. */
export function PackRow({ pack, update, onInstall, onRemove }: PackRowProps) {
  const [confirming, setConfirming] = useState(false);
  const working = pack.busy || update?.state === "installing";
  const failed = update?.state === "failed";

  return (
    <li className={`pack${working ? " working" : ""}`}>
      <div className="pack-text">
        <h3>{pack.title}</h3>
        <p className="what">{pack.description}</p>
        {working ? (
          <p className="progress-text">{update?.message ?? "Starting"}</p>
        ) : failed ? (
          <p className="problem">{update?.error}</p>
        ) : (
          <p className="details">{details(pack)}</p>
        )}
      </div>

      <div className="pack-action">
        {working ? (
          <span className="state">Installing</span>
        ) : pack.installed && confirming ? (
          <>
            <button className="quiet" onClick={() => setConfirming(false)}>Cancel</button>
            <button
              className="danger"
              onClick={() => {
                setConfirming(false);
                onRemove();
              }}
            >
              Remove
            </button>
          </>
        ) : pack.installed ? (
          <>
            <span className="state done">Installed</span>
            <button className="quiet" onClick={() => setConfirming(true)}>Remove</button>
          </>
        ) : (
          <button className="primary" disabled={pack.download_bytes === null} onClick={onInstall}>
            {failed ? "Try again" : "Install"}
          </button>
        )}
      </div>

      {working && (
        <div className="meter" aria-hidden>
          <i
            className={update?.percent == null ? "indeterminate" : ""}
            style={update?.percent == null ? undefined : { width: `${update.percent}%` }}
          />
        </div>
      )}
    </li>
  );
}
