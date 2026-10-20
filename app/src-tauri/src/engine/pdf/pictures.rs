//! Making a PDF from a picture.

use super::Library;
use pdfium_render::prelude::*;
use std::path::Path;

/// The longer side of the new page, in points: A4's long edge (297 mm).
const LONG_EDGE_PT: f32 = 842.0;

/// Page size for a picture of `width_px` x `height_px`: the picture's own proportions,
/// so there are no white bars, with the longer side at A4's length so it prints sensibly.
pub fn page_size(width_px: u32, height_px: u32) -> (f32, f32) {
    let longer = width_px.max(height_px).max(1) as f32;
    let scale = LONG_EDGE_PT / longer;
    (width_px as f32 * scale, height_px as f32 * scale)
}

/// Writes a one-page PDF holding the picture at `image_path`.
pub fn picture_to_pdf(library: &Library, image_path: &Path, output: &Path) -> Result<(), String> {
    let mut document = library.create()?;
    append_picture(&mut document, image_path)?;
    document.save_to_file(output).map_err(|e| format!("cannot write the PDF: {e}"))
}

/// Adds the picture at `image_path` to the end of `document` as one page of its own shape.
pub fn append_picture(document: &mut PdfDocument<'_>, image_path: &Path) -> Result<(), String> {
    let name = image_path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let picture = image::open(image_path).map_err(|e| format!("cannot read the picture {name}: {e}"))?;
    let (width, height) = page_size(picture.width(), picture.height());

    let mut page = document
        .pages_mut()
        .create_page_at_end(PdfPagePaperSize::Custom(PdfPoints::new(width), PdfPoints::new(height)))
        .map_err(|e| format!("cannot add a page: {e}"))?;
    page.objects_mut()
        .create_image_object(PdfPoints::ZERO, PdfPoints::ZERO, &picture, Some(PdfPoints::new(width)), Some(PdfPoints::new(height)))
        .map_err(|e| format!("cannot place the picture on the page: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_longer_side_is_a4_long_edge() {
        let (w, h) = page_size(2000, 1000);
        assert!((w - 842.0).abs() < 0.01 && (h - 421.0).abs() < 0.01);
        let (w, h) = page_size(1000, 2000);
        assert!((h - 842.0).abs() < 0.01 && (w - 421.0).abs() < 0.01);
    }

    #[test]
    fn a_square_stays_square() {
        let (w, h) = page_size(500, 500);
        assert_eq!(w, h);
    }
}
