export type IconKind =
  | "pdf"
  | "doc"
  | "text"
  | "code"
  | "markdown"
  | "image"
  | "audio"
  | "video"
  | "sheet"
  | "slides"
  | "book";

const BY_FORMAT: Record<string, IconKind> = {
  pdf: "pdf",
  docx: "doc",
  doc: "doc",
  odt: "doc",
  rtf: "doc",
  txt: "text",
  html: "code",
  md: "markdown",
  png: "image",
  jpg: "image",
  webp: "image",
  tiff: "image",
  bmp: "image",
  gif: "image",
  mp3: "audio",
  wav: "audio",
  flac: "audio",
  aac: "audio",
  m4a: "audio",
  ogg: "audio",
  mp4: "video",
  mkv: "video",
  webm: "video",
  xlsx: "sheet",
  ods: "sheet",
  csv: "sheet",
  pptx: "slides",
  odp: "slides",
  epub: "book",
};

export const iconKindFor = (formatId: string): IconKind => BY_FORMAT[formatId] ?? "doc";

/** One muted color per kind of file, defined as --k-<kind> in theme.css (and per design). */
export const PALETTE: Record<IconKind, string> = {
  pdf: "var(--k-pdf)",
  doc: "var(--k-doc)",
  text: "var(--k-text)",
  code: "var(--k-code)",
  markdown: "var(--k-markdown)",
  image: "var(--k-image)",
  audio: "var(--k-audio)",
  video: "var(--k-video)",
  sheet: "var(--k-sheet)",
  slides: "var(--k-slides)",
  book: "var(--k-book)",
};
