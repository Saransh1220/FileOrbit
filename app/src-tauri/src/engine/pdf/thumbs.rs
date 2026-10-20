//! Small page pictures for the page editor, and the size of each page.

use super::Library;
use image::codecs::jpeg::JpegEncoder;
use pdfium_render::prelude::*;
use serde::Serialize;
use std::path::Path;

/// JPEG quality for thumbnails: small files, still readable text.
const QUALITY: u8 = 80;
/// Widest thumbnail anyone can ask for (the large preview), so a bad request cannot render a poster.
const MAX_WIDTH_PX: u32 = 2000;

/// A page's shape in points, as it displays (its own rotation already applied).
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PageShape {
    pub width: f32,
    pub height: f32,
}

/// What the editor needs to lay out a file: its pages' shapes, in order.
pub fn shapes(library: &Library, path: &Path, password: Option<&str>) -> Result<Vec<PageShape>, String> {
    let document = library.open(path, password)?;
    Ok(document.pages().iter().map(|page| PageShape { width: page.width().value, height: page.height().value }).collect())
}

/// The width a request is clamped to.
pub fn clamp_width(width_px: u32) -> u32 {
    width_px.clamp(16, MAX_WIDTH_PX)
}

/// One page drawn `width_px` wide, as JPEG bytes.
pub fn thumbnail(library: &Library, path: &Path, number: u32, width_px: u32) -> Result<Vec<u8>, String> {
    let document = library.open(path, None)?;
    let page = document
        .pages()
        .get(number as PdfPageIndex)
        .map_err(|_| format!("page {} does not exist", number + 1))?;
    let config = PdfRenderConfig::new().set_target_width(clamp_width(width_px) as i32).rotate_if_landscape(PdfPageRenderRotation::None, false);
    let picture = page
        .render_with_config(&config)
        .map_err(|e| format!("cannot draw page {}: {e}", number + 1))?
        .as_image()
        .map_err(|e| format!("cannot draw the page: {e}"))?
        .to_rgb8();
    let mut bytes = Vec::new();
    JpegEncoder::new_with_quality(&mut bytes, QUALITY)
        .encode_image(&picture)
        .map_err(|e| format!("cannot encode the thumbnail: {e}"))?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requested_widths_are_kept_within_bounds() {
        assert_eq!(clamp_width(0), 16);
        assert_eq!(clamp_width(180), 180);
        assert_eq!(clamp_width(1_000_000), MAX_WIDTH_PX);
    }
}
