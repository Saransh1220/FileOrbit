import type { ReactNode } from "react";
import { iconKindFor, type IconKind } from "./formatCategories";

const PAGE = "M7 3h7l5 5v13H7z M14 3v5h5";

const PATHS: Record<IconKind, ReactNode> = {
  pdf: (
    <>
      <path d={PAGE} />
      <rect x="9" y="13" width="8" height="4.5" rx="1" />
    </>
  ),
  doc: (
    <>
      <path d={PAGE} />
      <path d="M9.5 12h6 M9.5 15h6 M9.5 18h4" />
    </>
  ),
  text: (
    <>
      <path d={PAGE} />
      <path d="M9.5 13h6 M9.5 16.5h3.5" />
    </>
  ),
  code: <path d="M9 7 4 12l5 5 M15 7l5 5-5 5 M13.2 5.5l-2.4 13" />,
  markdown: (
    <>
      <rect x="2.5" y="6" width="19" height="12" rx="2.5" />
      <path d="M6 15V9l3 3 3-3v6 M17 9v6 M15 13l2 2 2-2" />
    </>
  ),
  image: (
    <>
      <rect x="3" y="4.5" width="18" height="15" rx="2.5" />
      <circle cx="9" cy="10" r="1.6" />
      <path d="m21 16-5-5-8.5 8.5" />
    </>
  ),
  audio: <path d="M4 10v4 M8 7v10 M12 4v16 M16 8v8 M20 11v2" />,
  video: (
    <>
      <rect x="3" y="6" width="13" height="12" rx="2.5" />
      <path d="m16 10.5 5-3v9l-5-3" />
    </>
  ),
  sheet: (
    <>
      <rect x="3" y="4.5" width="18" height="15" rx="2.5" />
      <path d="M3 10h18 M3 14.8h18 M9.5 4.5v15" />
    </>
  ),
  slides: (
    <>
      <rect x="3" y="4" width="18" height="12" rx="2.5" />
      <path d="M8 20h8 M12 16v4" />
    </>
  ),
  merge: (
    <>
      <rect x="3" y="3.5" width="10" height="13" rx="2" />
      <path d="M8 7.5h0 M16 7.5h2.5a2 2 0 0 1 2 2V19a2 2 0 0 1-2 2H9a2 2 0 0 1-2-2v-2.5" />
      <path d="M8 9.5v4 M6 11.5h4" />
    </>
  ),
  // A small grid of pages: the page editor.
  organize: (
    <>
      <rect x="3.5" y="3.5" width="7" height="9" rx="1.5" />
      <rect x="13.5" y="3.5" width="7" height="9" rx="1.5" />
      <rect x="3.5" y="15" width="7" height="5.5" rx="1.5" />
      <rect x="13.5" y="15" width="7" height="5.5" rx="1.5" />
    </>
  ),
  // A frame with a corner arrow: change the size.
  resize: (
    <>
      <rect x="3.5" y="3.5" width="17" height="17" rx="2.5" />
      <path d="M9 15l6-6 M10.5 9H15v4.5" />
    </>
  ),
  book: (
    <>
      <path d="M4 5.5A2.5 2.5 0 0 1 6.5 3H19v15H6.5A2.5 2.5 0 0 0 4 20.5z" />
      <path d="M8 7.5h7" />
    </>
  ),
};

export function FormatIcon({ id }: { id: string }) {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.7"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden
    >
      {PATHS[iconKindFor(id)]}
    </svg>
  );
}
