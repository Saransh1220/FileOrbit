import type { PackInfo } from "./types";

export type PackGroup = "documents" | "media";

export interface PackMeta {
  /** A format id the icon set knows (see ring/formatCategories). */
  icon: string;
  /** The file-kind color token to tint the icon tile with: `--k-<kind>` in theme.css. */
  kind: string;
  group: PackGroup;
}

/** How each pack is shown. A pack missing here still renders, with a generic look. */
export const PACK_META: Record<string, PackMeta> = {
  pdf: { icon: "pdf", kind: "pdf", group: "documents" },
  documents: { icon: "docx", kind: "doc", group: "documents" },
  text: { icon: "md", kind: "markdown", group: "documents" },
  audio: { icon: "mp3", kind: "audio", group: "media" },
  video: { icon: "mp4", kind: "video", group: "media" },
  images: { icon: "png", kind: "image", group: "media" },
};

const FALLBACK: PackMeta = { icon: "docx", kind: "text", group: "documents" };

export const GROUP_ORDER: PackGroup[] = ["documents", "media"];

export const GROUP_TITLES: Record<PackGroup, string> = {
  documents: "Documents and text",
  media: "Audio, video and images",
};

export const metaFor = (packId: string): PackMeta => PACK_META[packId] ?? FALLBACK;

export interface PackSection {
  group: PackGroup;
  title: string;
  packs: PackInfo[];
}

/** Packs split into their display groups, in a fixed group order; empty groups are left out. */
export function groupPacks(packs: PackInfo[]): PackSection[] {
  return GROUP_ORDER.map((group) => ({
    group,
    title: GROUP_TITLES[group],
    packs: packs.filter((p) => metaFor(p.id).group === group),
  })).filter((section) => section.packs.length > 0);
}

/** How many packs can be used right now. */
export const readyCount = (packs: PackInfo[]): number => packs.filter((p) => p.installed).length;
