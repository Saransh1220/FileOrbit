use serde::Serialize;

/// The external program that performs a conversion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Engine {
    Pandoc,
    LibreOffice,
    Ffmpeg,
    /// PDF pages to pictures and text, and pictures to PDF.
    Pdfium,
    /// macOS-only image tool, used for formats ffmpeg cannot read (HEIC).
    Sips,
}

/// One wedge on the ring: what the user can convert the dragged file into.
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct Target {
    /// Output extension, also the wedge id ("pdf", "mp3", ...).
    pub id: String,
    pub label: String,
    /// True for an action that works on the whole selection (Merge), not a format to convert to.
    pub tool: bool,
}
