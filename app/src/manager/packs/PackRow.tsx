import { useState, type CSSProperties } from "react";
import { FormatIcon } from "../../ring/FormatIcon";
import { Button } from "../parts/Button";
import { Chip } from "../parts/Chip";
import { Meter } from "../parts/Meter";
import { formatBytes } from "../format";
import { metaFor } from "../packMeta";
import type { PackInfo, PackUpdate } from "../types";
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

/** The quiet line under the description: what it runs on, and its size. */
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

/** What the action area shows for a pack that is not being installed right now. */
function Action({ pack, failed, confirming, setConfirming, onInstall, onRemove }: {
  pack: PackInfo;
  failed: boolean;
  confirming: boolean;
  setConfirming: (value: boolean) => void;
  onInstall: () => void;
  onRemove: () => void;
}) {
  if (!pack.installed) {
    return (
      <Button variant="primary" disabled={pack.download_bytes === null} onClick={onInstall}>
        {failed ? "Try again" : pack.download_bytes === null ? "Unavailable" : "Install"}
      </Button>
    );
  }
  // Usable thanks to a program the computer already has: nothing of ours to remove.
  if (!pack.managed) return <Chip tone="ok">On this computer</Chip>;
  if (confirming) {
    return (
      <>
        <Button onClick={() => setConfirming(false)}>Cancel</Button>
        <Button
          variant="danger"
          onClick={() => {
            setConfirming(false);
            onRemove();
          }}
        >
          Remove
        </Button>
      </>
    );
  }
  return (
    <>
      <Chip tone="ok">Installed</Chip>
      <Button onClick={() => setConfirming(true)}>Remove</Button>
    </>
  );
}

/** One converter pack: icon, what it does, what it runs on, and its one action. */
export function PackRow({ pack, update, onInstall, onRemove }: PackRowProps) {
  const [confirming, setConfirming] = useState(false);
  const meta = metaFor(pack.id);
  const working = pack.busy || update?.state === "installing";
  const failed = update?.state === "failed";
  const tile = { "--tint": `var(--k-${meta.kind})` } as CSSProperties;

  return (
    <li className={`pack${working ? " working" : ""}`}>
      <span className="tile" style={tile} aria-hidden>
        <FormatIcon id={meta.icon} />
      </span>
      <div className="pack-text">
        <h3>{pack.title}</h3>
        <p className="what">{pack.description}</p>
        {working ? (
          <p className="progress-text">{update?.message ?? "Starting"}</p>
        ) : failed ? (
          <p className="problem" role="alert">{update?.error}</p>
        ) : (
          <p className="details">{details(pack)}</p>
        )}
      </div>
      <div className="pack-action">
        {working ? (
          <Chip tone="neutral">Installing</Chip>
        ) : (
          <Action pack={pack} failed={failed} confirming={confirming} setConfirming={setConfirming} onInstall={onInstall} onRemove={onRemove} />
        )}
      </div>
      {working && (
        <div className="pack-meter">
          <Meter percent={update?.percent ?? null} />
        </div>
      )}
    </li>
  );
}
