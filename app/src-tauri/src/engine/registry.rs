//! Which conversions exist. Pure data plus lookups, so it is trivially testable.

use super::target::{Engine, Target};
use std::path::Path;

use Engine::{Ffmpeg, LibreOffice, Pandoc, Pdfium};

struct Group {
    /// Pack that supplies ffmpeg for this group (ignored for other engines).
    media_pack: &'static str,
    inputs: &'static [&'static str],
    targets: &'static [(&'static str, Engine)],
}

/// The ring shows at most this many wedges; extra targets are dropped for now.
pub const MAX_TARGETS: usize = 8;

const GROUPS: &[Group] = &[
    Group {
        media_pack: "pdf",
        inputs: &["pdf"],
        targets: &[
            ("png", Pdfium),
            ("jpg", Pdfium),
            ("txt", Pdfium),
            ("docx", LibreOffice),
            ("odt", LibreOffice),
        ],
    },
    Group {
        media_pack: "documents",
        inputs: &["doc", "rtf"],
        targets: &[
            ("pdf", LibreOffice),
            ("docx", LibreOffice),
            ("odt", LibreOffice),
            ("txt", LibreOffice),
            ("html", LibreOffice),
        ],
    },
    Group {
        media_pack: "documents",
        inputs: &["docx", "odt"],
        targets: &[
            ("pdf", LibreOffice),
            ("docx", LibreOffice),
            ("odt", LibreOffice),
            ("txt", LibreOffice),
            ("html", LibreOffice),
            ("md", Pandoc),
        ],
    },
    Group {
        media_pack: "documents",
        inputs: &["ppt", "pptx", "odp"],
        targets: &[("pdf", LibreOffice), ("pptx", LibreOffice), ("odp", LibreOffice)],
    },
    Group {
        media_pack: "documents",
        inputs: &["xls", "xlsx", "ods", "csv"],
        targets: &[
            ("pdf", LibreOffice),
            ("xlsx", LibreOffice),
            ("ods", LibreOffice),
            ("csv", LibreOffice),
        ],
    },
    Group {
        media_pack: "text",
        inputs: &["md", "markdown", "html", "htm", "epub", "rst", "tex", "txt"],
        targets: &[
            ("docx", Pandoc),
            ("odt", Pandoc),
            ("html", Pandoc),
            ("md", Pandoc),
            ("txt", Pandoc),
            ("epub", Pandoc),
        ],
    },
    Group {
        media_pack: "images",
        inputs: &["png", "jpg", "jpeg", "gif", "tif", "tiff", "bmp", "webp"],
        targets: &[
            ("png", Ffmpeg),
            ("jpg", Ffmpeg),
            ("webp", Ffmpeg),
            ("tiff", Ffmpeg),
            ("bmp", Ffmpeg),
            ("gif", Ffmpeg),
            ("pdf", Pdfium),
        ],
    },
    #[cfg(target_os = "macos")]
    Group {
        media_pack: "images",
        inputs: &["heic", "heif"],
        targets: &[("jpg", Engine::Sips), ("png", Engine::Sips)],
    },
    Group {
        media_pack: "audio",
        inputs: &["mp3", "wav", "flac", "aac", "m4a", "ogg", "opus", "wma"],
        targets: &[
            ("mp3", Ffmpeg),
            ("wav", Ffmpeg),
            ("flac", Ffmpeg),
            ("aac", Ffmpeg),
            ("m4a", Ffmpeg),
            ("ogg", Ffmpeg),
        ],
    },
    Group {
        media_pack: "video",
        inputs: &["mp4", "mov", "mkv", "avi", "webm", "m4v"],
        targets: &[
            ("mp4", Ffmpeg),
            ("mkv", Ffmpeg),
            ("webm", Ffmpeg),
            ("gif", Ffmpeg),
            ("mp3", Ffmpeg),
            ("wav", Ffmpeg),
        ],
    },
];

/// Lower-cased extension with aliases folded together ("JPEG" -> "jpg").
pub fn normalized_ext(path: &Path) -> Option<String> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    Some(match ext.as_str() {
        "jpeg" => "jpg".to_string(),
        "tif" => "tiff".to_string(),
        "heif" => "heic".to_string(),
        "htm" => "html".to_string(),
        "markdown" => "md".to_string(),
        _ => ext,
    })
}

fn group_for(ext: &str) -> Option<&'static Group> {
    GROUPS.iter().find(|g| {
        g.inputs
            .iter()
            .any(|i| normalized_ext(Path::new(&format!("x.{i}"))).as_deref() == Some(ext))
    })
}

/// Wedges for a file type, assuming every converter can do everything.
pub fn targets_for(ext: &str) -> Vec<Target> {
    targets_for_filtered(ext, |_, _| true)
}

/// Wedges for a file type, keeping only targets `supported` accepts.
pub fn targets_for_filtered(ext: &str, supported: impl Fn(Engine, &str) -> bool) -> Vec<Target> {
    let Some(group) = group_for(ext) else {
        return Vec::new();
    };
    group
        .targets
        .iter()
        .filter(|(id, engine)| *id != ext && supported(*engine, id))
        .take(MAX_TARGETS)
        .map(|(id, _)| Target {
            id: (*id).to_string(),
            label: id.to_ascii_uppercase(),
            tool: false,
        })
        .collect()
}

/// The pack (see `radial install`) that supplies the program for `from_ext -> target_id`.
pub fn pack_for(from_ext: &str, target_id: &str) -> Option<&'static str> {
    let group = group_for(from_ext)?;
    Some(match engine_for(from_ext, target_id)? {
        LibreOffice => "documents",
        Pandoc => "text",
        Engine::Sips => "images",
        Pdfium => "pdf",
        Ffmpeg => group.media_pack,
    })
}

/// The engine that performs `from_ext -> target_id`, if that route exists.
pub fn engine_for(from_ext: &str, target_id: &str) -> Option<Engine> {
    group_for(from_ext)?
        .targets
        .iter()
        .find(|(id, _)| *id == target_id && *id != from_ext)
        .map(|(_, engine)| *engine)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(ext: &str) -> Vec<String> {
        targets_for(ext).into_iter().map(|t| t.id).collect()
    }

    #[test]
    fn docx_offers_pdf_but_not_itself() {
        let ids = ids("docx");
        assert!(ids.contains(&"pdf".to_string()));
        assert!(!ids.contains(&"docx".to_string()));
    }

    #[test]
    fn aliases_fold() {
        assert_eq!(normalized_ext(Path::new("a/B.JPEG")).as_deref(), Some("jpg"));
        assert_eq!(ids("jpg"), ids("jpg"));
        assert!(!ids("jpg").is_empty());
    }

    #[test]
    fn unknown_extension_has_no_targets() {
        assert!(targets_for("xyz").is_empty());
        assert!(engine_for("xyz", "pdf").is_none());
    }

    #[test]
    fn routes_pick_the_expected_engine() {
        assert_eq!(engine_for("docx", "pdf"), Some(LibreOffice));
        assert_eq!(engine_for("docx", "md"), Some(Pandoc));
        assert_eq!(engine_for("mp3", "wav"), Some(Ffmpeg));
        assert_eq!(engine_for("mp3", "mp3"), None);
        assert_eq!(engine_for("pdf", "png"), Some(Pdfium));
        assert_eq!(engine_for("pdf", "txt"), Some(Pdfium));
        assert_eq!(engine_for("pdf", "docx"), Some(LibreOffice));
        assert_eq!(engine_for("png", "pdf"), Some(Pdfium));
        assert_eq!(engine_for("pdf", "pdf"), None);
    }

    #[test]
    fn a_pdf_offers_pictures_text_and_word_documents() {
        assert_eq!(ids("pdf"), vec!["png", "jpg", "txt", "docx", "odt"]);
    }

    #[test]
    fn pictures_offer_pdf_among_their_formats() {
        for ext in ["png", "jpg", "webp", "tiff", "bmp", "gif"] {
            assert!(ids(ext).contains(&"pdf".to_string()), "{ext} should offer PDF");
        }
    }

    #[test]
    fn pdf_conversions_name_the_right_packs() {
        assert_eq!(pack_for("pdf", "png"), Some("pdf"));
        assert_eq!(pack_for("pdf", "txt"), Some("pdf"));
        assert_eq!(pack_for("png", "pdf"), Some("pdf"));
        // Word output comes from LibreOffice's PDF import, so it needs the Documents pack.
        assert_eq!(pack_for("pdf", "docx"), Some("documents"));
    }

    #[test]
    fn each_conversion_names_the_pack_that_supplies_it() {
        assert_eq!(pack_for("docx", "pdf"), Some("documents"));
        assert_eq!(pack_for("docx", "md"), Some("text"));
        assert_eq!(pack_for("md", "docx"), Some("text"));
        assert_eq!(pack_for("mp3", "wav"), Some("audio"));
        assert_eq!(pack_for("mp4", "gif"), Some("video"));
        assert_eq!(pack_for("png", "jpg"), Some("images"));
        assert_eq!(pack_for("xyz", "pdf"), None);
    }

    #[test]
    fn every_pack_named_exists_in_the_installer_catalog() {
        for g in GROUPS {
            assert!(radial_packs::catalog::pack(g.media_pack).is_some(), "{}", g.media_pack);
        }
        for p in ["documents", "text", "pdf"] {
            assert!(radial_packs::catalog::pack(p).is_some());
        }
    }

    #[test]
    fn no_group_exceeds_the_ring_limit() {
        for g in GROUPS {
            assert!(g.targets.len() <= MAX_TARGETS);
        }
    }
}
