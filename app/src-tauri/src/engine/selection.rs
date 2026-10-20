//! Which wedges the ring shows for the files being dragged.
//!
//! One file gets its own conversions. Several files get what they have in common,
//! plus "Merge" when they can all become pages of one PDF. A selection with
//! nothing in common gets no ring at all, instead of a ring that cannot do anything.

use super::registry;
use super::target::{Engine, Target};
use std::path::{Path, PathBuf};

/// The action id for "make one PDF out of these files".
pub const MERGE: &str = "merge";
/// The action id for "open this PDF in the page editor".
pub const ORGANIZE: &str = "organize";

/// Picture formats that can become a PDF page.
const PICTURE_EXTS: &[&str] = &["png", "jpg", "gif", "tiff", "bmp", "webp"];

pub fn is_picture(ext: &str) -> bool {
    PICTURE_EXTS.contains(&ext)
}

/// Whether a file of this type can become pages of a merged PDF.
pub fn can_merge(ext: &str) -> bool {
    ext == "pdf" || is_picture(ext)
}

/// "Merge" for PDFs (and PDFs with pictures); "Combine" when every file is a picture.
fn merge_label(exts: &[String]) -> &'static str {
    if exts.iter().all(|e| is_picture(e)) {
        "COMBINE"
    } else {
        "MERGE"
    }
}

/// Wedges for the dragged files. `supported` lets the caller drop conversions the
/// installed converters cannot do. Empty means: show no ring.
pub fn ring_targets(paths: &[PathBuf], supported: impl Fn(Engine, &str) -> bool + Copy) -> Vec<Target> {
    let exts: Vec<String> = match paths.iter().map(|p| registry::normalized_ext(p)).collect::<Option<Vec<_>>>() {
        Some(exts) if !exts.is_empty() => exts,
        _ => return Vec::new(),
    };
    targets_for_types(&exts, supported)
}

/// The same rules on file types alone, so they can be tested without files.
pub fn targets_for_types(exts: &[String], supported: impl Fn(Engine, &str) -> bool + Copy) -> Vec<Target> {
    if exts.is_empty() {
        return Vec::new();
    }
    // One list per file; a type nobody can convert makes the whole selection ringless.
    let lists: Vec<Vec<Target>> = exts.iter().map(|e| registry::targets_for_filtered(e, supported)).collect();
    if lists.iter().any(|l| l.is_empty()) && !exts.iter().all(|e| can_merge(e)) {
        return Vec::new();
    }
    if lists.iter().any(|l| l.is_empty()) {
        // Every file can merge, even if one of them has no conversions of its own.
        return merge_only(exts);
    }

    // What every file can be converted to.
    let mut common: Vec<Target> = lists[0].clone();
    common.retain(|t| lists.iter().all(|l| l.iter().any(|x| x.id == t.id)));

    let mut wedges = Vec::new();
    if exts.len() > 1 && exts.iter().all(|e| can_merge(e)) {
        wedges.push(merge_wedge(exts));
    }
    wedges.extend(common);
    wedges.truncate(registry::MAX_TARGETS - 1);
    // The page editor takes any number of PDFs (several open side by side in one editor).
    if exts.iter().all(|e| e == "pdf") {
        wedges.push(Target { id: ORGANIZE.to_string(), label: "ORGANIZE".to_string(), tool: true });
    }
    wedges
}

fn merge_only(exts: &[String]) -> Vec<Target> {
    if exts.len() > 1 { vec![merge_wedge(exts)] } else { Vec::new() }
}

fn merge_wedge(exts: &[String]) -> Target {
    Target { id: MERGE.to_string(), label: merge_label(exts).to_string(), tool: true }
}

/// What the ring calls the dragged files: one file's name, or "5 files".
pub fn selection_name(paths: &[PathBuf]) -> String {
    match paths {
        [one] => file_name(one),
        many => format!("{} files", many.len()),
    }
}

fn file_name(path: &Path) -> String {
    path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exts(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    fn ids(list: &[&str]) -> Vec<String> {
        targets_for_types(&exts(list), |_, _| true).into_iter().map(|t| t.id).collect()
    }

    #[test]
    fn one_file_gets_its_own_conversions() {
        assert_eq!(ids(&["pdf"]), vec!["png", "jpg", "txt", "docx", "odt", "organize"]);
        assert!(!ids(&["mp3"]).contains(&"merge".to_string()));
    }

    #[test]
    fn a_single_pdf_is_not_offered_a_merge() {
        assert!(!ids(&["pdf"]).contains(&"merge".to_string()));
    }

    #[test]
    fn only_pdfs_are_offered_the_page_editor() {
        let wedges = targets_for_types(&exts(&["pdf"]), |_, _| true);
        let organize = wedges.last().unwrap();
        assert_eq!((organize.id.as_str(), organize.tool), (ORGANIZE, true));
        assert!(ids(&["pdf", "pdf"]).contains(&ORGANIZE.to_string()));
        assert!(!ids(&["pdf", "png"]).contains(&ORGANIZE.to_string()));
        assert!(!ids(&["png"]).contains(&ORGANIZE.to_string()));
    }

    #[test]
    fn several_pdfs_get_merge_first_then_what_they_share() {
        assert_eq!(ids(&["pdf", "pdf"]), vec!["merge", "png", "jpg", "txt", "docx", "odt", "organize"]);
    }

    #[test]
    fn several_pictures_get_combine_and_their_shared_conversions() {
        let wedges = targets_for_types(&exts(&["png", "png"]), |_, _| true);
        assert_eq!(wedges[0].id, "merge");
        assert_eq!(wedges[0].label, "COMBINE", "pictures are combined, not merged");
        assert!(wedges.iter().any(|t| t.id == "jpg") && wedges.iter().any(|t| t.id == "pdf"));
    }

    #[test]
    fn pdfs_and_pictures_together_can_still_merge() {
        let wedges = targets_for_types(&exts(&["pdf", "png"]), |_, _| true);
        assert_eq!(wedges[0].id, "merge");
        assert_eq!(wedges[0].label, "MERGE");
        // The only conversion both can do is to JPG.
        assert_eq!(wedges.iter().skip(1).map(|t| t.id.as_str()).collect::<Vec<_>>(), vec!["jpg"]);
    }

    #[test]
    fn different_kinds_with_nothing_in_common_get_no_ring() {
        assert!(ids(&["pdf", "mp4"]).is_empty());
        assert!(ids(&["docx", "mp3"]).is_empty());
        assert!(ids(&["png", "mp3"]).is_empty());
    }

    #[test]
    fn several_files_of_one_kind_share_their_conversions() {
        assert_eq!(ids(&["mp3", "wav"]), vec!["flac", "aac", "m4a", "ogg"]);
        assert!(!ids(&["mp3", "mp3"]).contains(&"merge".to_string()));
    }

    #[test]
    fn unknown_types_get_no_ring() {
        assert!(ids(&["xyz"]).is_empty());
        assert!(ids(&["pdf", "xyz"]).is_empty());
        assert!(ids(&[]).is_empty());
    }

    #[test]
    fn the_tool_wedge_is_marked_and_conversions_are_not() {
        let wedges = targets_for_types(&exts(&["pdf", "pdf"]), |_, _| true);
        assert!(wedges[0].tool);
        let tools: Vec<&str> = wedges.iter().filter(|t| t.tool).map(|t| t.id.as_str()).collect();
        assert_eq!(tools, vec!["merge", "organize"], "conversions are never marked as tools");
    }

    #[test]
    fn unsupported_conversions_are_dropped_from_a_shared_list() {
        let wedges = targets_for_types(&exts(&["mp3", "wav"]), |_, id| id != "ogg");
        assert!(!wedges.iter().any(|t| t.id == "ogg"));
    }

    #[test]
    fn the_ring_never_exceeds_its_limit() {
        assert!(ids(&["pdf", "pdf"]).len() <= registry::MAX_TARGETS);
    }

    #[test]
    fn files_without_an_extension_make_the_selection_ringless() {
        let paths = vec![PathBuf::from("/a/report.pdf"), PathBuf::from("/a/README")];
        assert!(ring_targets(&paths, |_, _| true).is_empty());
    }

    #[test]
    fn the_selection_is_named_for_the_hub() {
        assert_eq!(selection_name(&[PathBuf::from("/a/x.pdf")]), "x.pdf");
        assert_eq!(selection_name(&[PathBuf::from("/a/x.pdf"), PathBuf::from("/a/y.pdf")]), "2 files");
    }
}
