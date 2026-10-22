import { PackRow } from "../packs/PackRow";
import { Page } from "../parts/Page";
import { groupPacks, readyCount } from "../packMeta";
import { SECTIONS } from "../sections";
import type { Overview, PackUpdate } from "../types";
import "./ConvertersPage.css";

export interface ConvertersPageProps {
  /** null until the first load finishes. */
  overview: Overview | null;
  updates: Record<string, PackUpdate>;
  /** A problem with the last action, shown above the list. */
  problem: string | null;
  onInstall: (pack: string) => void;
  onRemove: (pack: string) => void;
}

const BLURB = SECTIONS.find((s) => s.id === "converters")?.blurb;

/** The converter packs, grouped, each with its single action. */
export function ConvertersPage({ overview, updates, problem, onInstall, onRemove }: ConvertersPageProps) {
  return (
    <Page title="Converters" blurb={BLURB}>
      {problem && (
        <p className="banner" role="alert">
          {problem}
        </p>
      )}
      {overview === null ? (
        <p className="loading">Loading…</p>
      ) : (
        <>
          <p className="summary">
            <strong>{readyCount(overview.packs)}</strong> of {overview.packs.length} ready to use
          </p>
          {groupPacks(overview.packs).map((section) => (
            <div className="pack-group" key={section.group}>
              <h2>{section.title}</h2>
              <ul>
                {section.packs.map((pack) => (
                  <PackRow key={pack.id} pack={pack} update={updates[pack.id]} onInstall={() => onInstall(pack.id)} onRemove={() => onRemove(pack.id)} />
                ))}
              </ul>
            </div>
          ))}
        </>
      )}
    </Page>
  );
}
