import type { KeyboardEvent } from "react";
import { STATUS_SHORT, STATUS_TONE, needsAttention } from "../hookStatus";
import { Logo } from "../parts/Logo";
import { NavIcon } from "../parts/NavIcon";
import { SECTIONS, stepSection, type SectionId } from "../sections";
import type { HookStatus } from "../types";
import "./Sidebar.css";

export interface SidebarProps {
  section: SectionId;
  status: HookStatus;
  onSelect: (section: SectionId) => void;
}

/** The left column: brand, the sections, and a live Shift+drag status at the bottom. */
export function Sidebar({ section, status, onSelect }: SidebarProps) {
  // Up and Down move through the sections; focus follows so the keyboard keeps working.
  const onKeyDown = (event: KeyboardEvent<HTMLElement>) => {
    const next = stepSection(section, event.key);
    if (next === section) return;
    event.preventDefault();
    onSelect(next);
    requestAnimationFrame(() => document.getElementById(`nav-${next}`)?.focus());
  };

  return (
    <aside className="side">
      <div className="brand">
        <Logo />
        <span>Radial Convert</span>
      </div>

      <nav className="nav" aria-label="Sections" onKeyDown={onKeyDown}>
        {SECTIONS.map((item) => {
          const active = item.id === section;
          return (
            <button key={item.id} id={`nav-${item.id}`} type="button" className={active ? "active" : ""} aria-current={active ? "page" : undefined} onClick={() => onSelect(item.id)}>
              <NavIcon id={item.id} />
              <span>{item.label}</span>
              {item.id === "shortcut" && needsAttention(status) && <i className="attention" role="img" aria-label="Needs attention" />}
            </button>
          );
        })}
      </nav>

      <button type="button" className={`foot ${STATUS_TONE[status]}`} onClick={() => onSelect("shortcut")} title="Shift+drag status">
        <span className="foot-dot" aria-hidden />
        {STATUS_SHORT[status]}
      </button>
    </aside>
  );
}
