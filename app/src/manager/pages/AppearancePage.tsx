import { Page } from "../parts/Page";
import { SECTIONS } from "../sections";
import { visibleThemes } from "../themes";
import type { DesignChoice } from "../types";
import "./AppearancePage.css";

export interface AppearancePageProps {
  /** Every theme the app offers: [id, label]. */
  designs: DesignChoice[];
  /** The active theme's id. */
  design: string;
  onDesign: (id: string) => void;
}

const BLURB = SECTIONS.find((s) => s.id === "appearance")?.blurb;

/**
 * The theme picker. Each preview carries its own `data-design`, so it paints itself
 * from that theme's tokens whatever theme the window is using right now.
 */
export function AppearancePage({ designs, design, onDesign }: AppearancePageProps) {
  const themes = visibleThemes(designs);
  const active = designs.find(([id]) => id === design);
  const unlisted = active && !themes.some((t) => t.id === design);

  return (
    <Page title="Appearance" blurb={BLURB}>
      <div className="themes" role="radiogroup" aria-label="Theme">
        {themes.map((theme) => (
          <button
            key={theme.id}
            type="button"
            role="radio"
            aria-checked={design === theme.id}
            className={`theme${design === theme.id ? " active" : ""}`}
            onClick={() => onDesign(theme.id)}
          >
            <span className="preview" data-design={theme.id} aria-hidden>
              <i className="p-side">
                <b />
                <b />
                <b />
              </i>
              <i className="p-main">
                <b className="p-title" />
                <b className="p-line" />
                <b className="p-line short" />
                <b className="p-button" />
              </i>
            </span>
            <span className="theme-name">
              {theme.label}
              <svg className="check" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round" aria-hidden>
                <path d="m5 12.5 4.5 4.5L19 7.5" />
              </svg>
            </span>
            <span className="theme-blurb">{theme.blurb}</span>
          </button>
        ))}
      </div>
      {unlisted && <p className="theme-note">The tray menu is set to “{active[1]}”, which has no preview here. Choose one above to switch.</p>}
    </Page>
  );
}
