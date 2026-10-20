//! A page plan: the pages of a new PDF, each one a reference to a page of an
//! input file plus an extra turn. The page editor edits this list and nothing
//! else; the PDF is written once, on Save, so the originals are never touched.

use super::Library;
use pdfium_render::prelude::*;
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// One page of the new PDF.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PageRef {
    /// Index into the list of input files.
    pub source: usize,
    /// Page number in that file, counting from 0.
    pub page: u32,
    /// Extra clockwise turn on top of the page's own, in degrees: 0, 90, 180 or 270.
    pub turn: u16,
    /// A new empty page of this width and height in points; `source` and `page` are ignored.
    #[serde(default)]
    pub blank: Option<[f32; 2]>,
}

/// What a saved file is called next to the first input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SaveKind {
    /// All the planned pages: "name (edited).pdf".
    Edited,
    /// A selection saved on its own: "name (extract).pdf".
    Extract,
}

impl SaveKind {
    fn suffix(self) -> &'static str {
        match self {
            SaveKind::Edited => " (edited)",
            SaveKind::Extract => " (extract)",
        }
    }
}

/// The rotation a page ends up with: its own plus the extra turn.
pub fn combined_turn(own_degrees: u16, extra_degrees: u16) -> u16 {
    (own_degrees + extra_degrees) % 360
}

fn rotation_of(degrees: u16) -> PdfPageRenderRotation {
    match degrees % 360 {
        90 => PdfPageRenderRotation::Degrees90,
        180 => PdfPageRenderRotation::Degrees180,
        270 => PdfPageRenderRotation::Degrees270,
        _ => PdfPageRenderRotation::None,
    }
}

/// Checks a plan against the page counts of its inputs, so a bad plan is refused
/// with a sentence before any file is written.
pub fn validate(page_counts: &[u32], pages: &[PageRef]) -> Result<(), String> {
    if pages.is_empty() {
        return Err("there are no pages to save; keep at least one".into());
    }
    // PDFium numbers pages with 16 bits.
    if pages.len() > u16::MAX as usize {
        return Err("a PDF can hold at most 65,535 pages".into());
    }
    for (position, page) in pages.iter().enumerate() {
        if let Some([width, height]) = page.blank {
            if !(width.is_finite() && height.is_finite() && width >= 1.0 && height >= 1.0 && width <= 14400.0 && height <= 14400.0) {
                return Err(format!("page {} is a blank page of an impossible size", position + 1));
            }
            continue;
        }
        let Some(count) = page_counts.get(page.source) else {
            return Err(format!("page {} points at a file that is not open", position + 1));
        };
        if page.page >= *count {
            return Err(format!("page {} points past the end of its file", position + 1));
        }
        if page.turn % 90 != 0 || page.turn >= 360 {
            return Err(format!("page {} has an unsupported turn of {} degrees", position + 1, page.turn));
        }
    }
    Ok(())
}

/// Writes `output` holding the planned pages in order. Nothing is written if the
/// plan is invalid or a file cannot be read. Returns the page count.
pub fn write(library: &Library, inputs: &[PathBuf], pages: &[PageRef], output: &Path) -> Result<u32, String> {
    let sources = inputs.iter().map(|input| library.open(input, None)).collect::<Result<Vec<_>, _>>()?;
    let counts: Vec<u32> = sources.iter().map(|document| document.pages().len() as u32).collect();
    validate(&counts, pages)?;

    let mut document = library.create()?;
    for (position, page) in pages.iter().enumerate() {
        let at = position as PdfPageIndex;
        if let Some([width, height]) = page.blank {
            document
                .pages_mut()
                .create_page_at_index(PdfPagePaperSize::Custom(PdfPoints::new(width), PdfPoints::new(height)), at)
                .map_err(|e| format!("cannot add a blank page at {}: {e}", position + 1))?;
            continue;
        }
        document
            .pages_mut()
            .copy_page_from_document(&sources[page.source], page.page as PdfPageIndex, at)
            .map_err(|e| format!("cannot copy page {}: {e}", position + 1))?;
        if page.turn != 0 {
            let mut copied = document.pages().get(at).map_err(|e| format!("cannot reach page {}: {e}", position + 1))?;
            let own = copied.rotation().map(|r| r.as_degrees() as u16).unwrap_or(0);
            copied.set_rotation(rotation_of(combined_turn(own, page.turn)));
        }
    }
    document.save_to_file(output).map_err(|e| format!("cannot write the PDF: {e}"))?;
    Ok(pages.len() as u32)
}

/// Saves the plan next to the first input as "<name> (edited).pdf" or "(extract)" (then "(1)",
/// never overwriting anything). Leaves nothing behind if it fails. Returns the file and its page count.
pub fn save_next_to_first(library: &Library, inputs: &[PathBuf], pages: &[PageRef], kind: SaveKind) -> Result<(PathBuf, u32), String> {
    save_with_suffix(library, inputs, pages, kind.suffix())
}

/// One part of a split: "<name> (part 2).pdf".
pub fn save_part(library: &Library, inputs: &[PathBuf], pages: &[PageRef], part: u32) -> Result<(PathBuf, u32), String> {
    save_with_suffix(library, inputs, pages, &format!(" (part {part})"))
}

fn save_with_suffix(library: &Library, inputs: &[PathBuf], pages: &[PageRef], suffix: &str) -> Result<(PathBuf, u32), String> {
    let first = inputs.first().ok_or_else(|| "there is no file to save from".to_string())?;
    let output = crate::engine::naming::reserve_with_suffix(first, suffix, "pdf")?;
    match write(library, inputs, pages, &output) {
        Ok(count) => Ok((output, count)),
        Err(message) => {
            let _ = std::fs::remove_file(&output);
            Err(message)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(source: usize, page: u32, turn: u16) -> PageRef {
        PageRef { source, page, turn, blank: None }
    }

    #[test]
    fn a_blank_page_needs_no_file_but_a_sane_size() {
        let blank = |w, h| PageRef { source: 0, page: 0, turn: 0, blank: Some([w, h]) };
        assert!(validate(&[], &[blank(595.0, 842.0)]).is_ok());
        assert!(validate(&[], &[blank(0.0, 842.0)]).is_err());
        assert!(validate(&[], &[blank(f32::NAN, 842.0)]).is_err());
    }

    #[test]
    fn turns_add_up_and_wrap() {
        assert_eq!(combined_turn(0, 90), 90);
        assert_eq!(combined_turn(270, 90), 0);
        assert_eq!(combined_turn(180, 270), 90);
    }

    #[test]
    fn a_good_plan_passes() {
        assert!(validate(&[3, 2], &[page(0, 2, 0), page(1, 0, 90), page(0, 0, 270)]).is_ok());
    }

    #[test]
    fn an_empty_plan_is_refused() {
        assert!(validate(&[3], &[]).unwrap_err().contains("at least one"));
    }

    #[test]
    fn a_page_past_the_end_is_refused_by_position() {
        let error = validate(&[3], &[page(0, 0, 0), page(0, 3, 0)]).unwrap_err();
        assert!(error.contains("page 2"), "{error}");
    }

    #[test]
    fn an_unknown_file_is_refused() {
        assert!(validate(&[3], &[page(1, 0, 0)]).is_err());
    }

    #[test]
    fn a_turn_that_is_not_a_quarter_is_refused() {
        assert!(validate(&[3], &[page(0, 0, 45)]).is_err());
        assert!(validate(&[3], &[page(0, 0, 360)]).is_err());
    }
}
