//! Drawing a picture into the shape `dimensions` worked out.

use super::dimensions::Layout;
use image::imageops::{self, FilterType};
use image::{DynamicImage, Rgba, RgbaImage};

const FILTER: FilterType = FilterType::Lanczos3;

/// Applies `layout`. `background` fills the bars of a padded picture.
pub fn apply(picture: &DynamicImage, layout: &Layout, background: Rgba<u8>) -> DynamicImage {
    match *layout {
        Layout::Scale { width, height } => scaled(picture, width, height),
        Layout::Cover { scaled: (w, h), crop: (x, y, cw, ch) } => scaled(picture, w, h).crop_imm(x, y, cw, ch),
        Layout::Contain { scaled: (w, h), canvas: (cw, ch), offset: (x, y) } => {
            let inner = scaled(picture, w, h).to_rgba8();
            let mut canvas = RgbaImage::from_pixel(cw, ch, background);
            imageops::overlay(&mut canvas, &inner, x as i64, y as i64);
            DynamicImage::ImageRgba8(canvas)
        }
    }
}

fn scaled(picture: &DynamicImage, width: u32, height: u32) -> DynamicImage {
    if picture.width() == width && picture.height() == height {
        picture.clone()
    } else {
        picture.resize_exact(width, height, FILTER)
    }
}

/// A smaller copy at `percent` of the size, used by the size search.
pub fn shrunk(picture: &DynamicImage, percent: u32) -> DynamicImage {
    let scale = percent as f64 / 100.0;
    scaled(picture, ((picture.width() as f64 * scale).round() as u32).max(1), ((picture.height() as f64 * scale).round() as u32).max(1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::imaging::dimensions::plan;
    use crate::engine::imaging::spec::{Fit, Resize};

    fn red(w: u32, h: u32) -> DynamicImage {
        DynamicImage::ImageRgba8(RgbaImage::from_pixel(w, h, Rgba([255, 0, 0, 255])))
    }

    fn done(source: &DynamicImage, resize: Resize, background: Rgba<u8>) -> DynamicImage {
        let layout = plan((source.width(), source.height()), &resize).unwrap();
        apply(source, &layout, background)
    }

    #[test]
    fn the_result_has_the_planned_size() {
        let out = done(&red(400, 300), Resize::Exact { width: 100, height: 100, fit: Fit::Crop }, Rgba([0, 0, 0, 0]));
        assert_eq!((out.width(), out.height()), (100, 100));
    }

    #[test]
    fn padding_shows_the_background_in_the_bars() {
        let out = done(&red(400, 100), Resize::Exact { width: 100, height: 100, fit: Fit::Pad }, Rgba([255, 255, 255, 255])).to_rgba8();
        assert_eq!(out.get_pixel(50, 2), &Rgba([255, 255, 255, 255]));
        assert_eq!(out.get_pixel(50, 50), &Rgba([255, 0, 0, 255]));
    }

    #[test]
    fn cropping_leaves_no_bars() {
        let out = done(&red(400, 100), Resize::Exact { width: 100, height: 100, fit: Fit::Crop }, Rgba([255, 255, 255, 255])).to_rgba8();
        assert!(out.pixels().all(|p| p == &Rgba([255, 0, 0, 255])));
    }
}
