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

/** One muted color per kind of file. Used for the icon, the hover tint and the hover outline. */
export const PALETTE: Record<IconKind, string> = {
  pdf: "#c26a52",
  doc: "#4f7bb8",
  text: "#7a8794",
  code: "#bf8d3f",
  markdown: "#7f6cb0",
  image: "#4f9a80",
  audio: "#b5658c",
  video: "#3f98a8",
  sheet: "#6b9a49",
  slides: "#c47d4c",
  book: "#6070b5",
};
