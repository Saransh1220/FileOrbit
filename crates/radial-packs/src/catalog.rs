//! What can be installed: engines (programs) and packs (features that need them).

use std::collections::BTreeSet;

/// An external converter program.
pub struct EngineDef {
    pub id: &'static str,
    pub name: &'static str,
    /// Executable name without extension, as found on a system PATH.
    pub exe: &'static str,
    /// Argument that makes it print its version and exit (used by `doctor`).
    /// Empty for a library (such as PDFium), which is loaded by the app, not run.
    pub version_arg: &'static str,
}

/// A feature the user picks; it lists the engines it depends on.
pub struct PackDef {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub engines: &'static [&'static str],
}

pub const ENGINES: &[EngineDef] = &[
    EngineDef { id: "ffmpeg", name: "ffmpeg", exe: "ffmpeg", version_arg: "-version" },
    EngineDef { id: "pandoc", name: "pandoc", exe: "pandoc", version_arg: "--version" },
    EngineDef { id: "libreoffice", name: "LibreOffice", exe: "soffice", version_arg: "--version" },
    EngineDef { id: "pdfium", name: "PDFium", exe: "pdfium", version_arg: "" },
];

pub const PACKS: &[PackDef] = &[
    PackDef {
        id: "pdf",
        title: "PDF tools",
        description: "PDF pages to PNG, JPG and text; merge, reorder and combine",
        engines: &["pdfium"],
    },
    PackDef {
        id: "documents",
        title: "Documents",
        description: "Word, Excel, PowerPoint, RTF to PDF and to each other",
        engines: &["libreoffice"],
    },
    PackDef {
        id: "text",
        title: "Text & Markup",
        description: "Markdown, HTML, EPUB, TeX, plain text",
        engines: &["pandoc"],
    },
    PackDef {
        id: "audio",
        title: "Audio",
        description: "MP3, WAV, FLAC, AAC, M4A, OGG",
        engines: &["ffmpeg"],
    },
    PackDef {
        id: "video",
        title: "Video",
        description: "MP4, MOV, MKV, WEBM, GIF, extract audio",
        engines: &["ffmpeg"],
    },
    PackDef {
        id: "images",
        title: "Images",
        description: "PNG, JPG, WEBP, TIFF, BMP, GIF",
        engines: &["ffmpeg"],
    },
];

pub fn engine(id: &str) -> Option<&'static EngineDef> {
    ENGINES.iter().find(|e| e.id == id)
}

pub fn pack(id: &str) -> Option<&'static PackDef> {
    PACKS.iter().find(|p| p.id == id)
}

/// Engines needed by a set of packs, each listed once however many packs share it.
pub fn engines_for<'a>(pack_ids: impl IntoIterator<Item = &'a str>) -> BTreeSet<&'static str> {
    pack_ids
        .into_iter()
        .filter_map(pack)
        .flat_map(|p| p.engines.iter().copied())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packs_sharing_an_engine_need_it_once() {
        let engines = engines_for(["audio", "images", "video"]);
        assert_eq!(engines.into_iter().collect::<Vec<_>>(), vec!["ffmpeg"]);
    }

    #[test]
    fn every_pack_engine_exists() {
        for p in PACKS {
            for e in p.engines {
                assert!(engine(e).is_some(), "{} needs unknown engine {e}", p.id);
            }
        }
    }

    #[test]
    fn unknown_packs_are_ignored_here() {
        assert!(engines_for(["nope"]).is_empty());
    }
}
