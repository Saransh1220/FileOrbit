//! Turning PDF pages into pictures.

use image::DynamicImage;
use pdfium_render::prelude::*;

/// Resolution for page pictures: sharp on screen and fine for printing.
pub const DPI: f32 = 150.0;
/// A PDF can declare a poster-sized page; cap the longer side so one page cannot use gigabytes.
const MAX_EDGE_PX: f32 = 7000.0;

/// How many pixels a page of `width_pt` x `height_pt` points becomes at `dpi`, capped.
/// A point is 1/72 inch. Pure so the sizing rules are tested without a PDF.
pub fn pixel_size(width_pt: f32, height_pt: f32, dpi: f32) -> (u32, u32) {
    let scale = dpi / 72.0;
    let (w, h) = (width_pt * scale, height_pt * scale);
    let shrink = (MAX_EDGE_PX / w.max(h)).min(1.0);
    (((w * shrink).round() as u32).max(1), ((h * shrink).round() as u32).max(1))
}

/// Renders one page on a white background.
pub fn render_page(page: &PdfPage<'_>) -> Result<DynamicImage, String> {
    let (width, height) = pixel_size(page.width().value, page.height().value, DPI);
    let config = PdfRenderConfig::new().set_target_size(width as i32, height as i32).rotate_if_landscape(PdfPageRenderRotation::None, false);
    page.render_with_config(&config)
        .map_err(|e| format!("cannot draw the page: {e}"))?
        .as_image()
        .map_err(|e| format!("cannot draw the page: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a4_at_150_dpi() {
        // 595 x 842 pt at 150 dpi is 1240 x 1754 px.
        assert_eq!(pixel_size(595.0, 842.0, 150.0), (1240, 1754));
    }

    #[test]
    fn a_poster_is_capped_on_its_longer_side() {
        let (w, h) = pixel_size(3000.0, 6000.0, 150.0);
        assert_eq!(h, 7000);
        assert_eq!(w, 3500, "the aspect ratio is kept");
    }

    #[test]
    fn a_tiny_page_is_never_zero_pixels() {
        assert_eq!(pixel_size(0.1, 0.1, 72.0), (1, 1));
    }
}
