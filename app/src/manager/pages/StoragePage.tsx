import { Button } from "../parts/Button";
import { Page } from "../parts/Page";
import { Row, RowGroup } from "../parts/Row";
import { formatBytes } from "../format";
import { SECTIONS } from "../sections";
import type { Overview } from "../types";
import "./StoragePage.css";

export interface StoragePageProps {
  overview: Overview | null;
  onClearDownloads: () => void;
  onOpenFolder: () => void;
}

const BLURB = SECTIONS.find((s) => s.id === "storage")?.blurb;

/** Installed size and kept downloads side by side as one bar, so the split is visible at a glance. */
function UsageBar({ installed, kept }: { installed: number; kept: number }) {
  const total = installed + kept;
  const share = (part: number) => (total === 0 ? 0 : (part / total) * 100);
  return (
    <div className="usage">
      <div className="usage-bar" aria-hidden>
        <i className="installed" style={{ width: `${share(installed)}%` }} />
        <i className="kept" style={{ width: `${share(kept)}%` }} />
      </div>
      <p className="usage-legend">
        <span className="installed">Converters {formatBytes(installed)}</span>
        <span className="kept">Kept downloads {formatBytes(kept)}</span>
      </p>
    </div>
  );
}

/** What the converters take up, and the two things you can do about it. */
export function StoragePage({ overview, onClearDownloads, onOpenFolder }: StoragePageProps) {
  if (overview === null) {
    return (
      <Page title="Storage" blurb={BLURB}>
        <p className="loading">Loading…</p>
      </Page>
    );
  }
  return (
    <Page title="Storage" blurb={BLURB}>
      <UsageBar installed={overview.installed_bytes} kept={overview.cache_bytes} />
      <RowGroup>
        <Row label="Installed converters" hint="The programs that do the converting.">
          {formatBytes(overview.installed_bytes)}
        </Row>
        <Row label="Kept downloads" hint="Kept so a repair or reinstall does not download again.">
          {formatBytes(overview.cache_bytes)}
          <Button disabled={overview.cache_bytes === 0} onClick={onClearDownloads}>
            Clear
          </Button>
        </Row>
        <Row label="Location" hint={overview.folder}>
          <Button onClick={onOpenFolder}>Open folder</Button>
        </Row>
      </RowGroup>
    </Page>
  );
}
