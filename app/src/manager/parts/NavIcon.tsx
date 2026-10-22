import type { SectionId } from "../sections";

const PATHS: Record<SectionId, React.ReactNode> = {
  converters: (
    <>
      <rect x="3.5" y="3.5" width="7" height="7" rx="1.8" />
      <rect x="13.5" y="3.5" width="7" height="7" rx="1.8" />
      <rect x="3.5" y="13.5" width="7" height="7" rx="1.8" />
      <rect x="13.5" y="13.5" width="7" height="7" rx="1.8" />
    </>
  ),
  appearance: (
    <>
      <path d="M12 3.5a8.5 8.5 0 1 0 0 17c1.1 0 1.8-.8 1.8-1.8 0-.6-.3-1-.6-1.4-.3-.4-.5-.8-.5-1.3 0-1 .8-1.7 1.8-1.7H17a3.5 3.5 0 0 0 3.5-3.5C20.5 6.6 16.8 3.5 12 3.5z" />
      <circle cx="7.8" cy="11.2" r="1" />
      <circle cx="10.2" cy="7.6" r="1" />
      <circle cx="14.4" cy="7.8" r="1" />
    </>
  ),
  storage: (
    <>
      <rect x="3.5" y="13" width="17" height="7" rx="2" />
      <path d="m5.5 13 2-7.5h9l2 7.5" />
      <circle cx="16.8" cy="16.5" r="0.9" />
    </>
  ),
  shortcut: <path d="M12 4 20 13h-5v7H9v-7H4z" />,
};

/** A 20px outline icon for a sidebar section. */
export function NavIcon({ id }: { id: SectionId }) {
  return (
    <svg className="nav-icon" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.7" strokeLinecap="round" strokeLinejoin="round" aria-hidden>
      {PATHS[id]}
    </svg>
  );
}
