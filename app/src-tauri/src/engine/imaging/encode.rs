//! Turning pixels into JPG or PNG bytes.

use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::{DynamicImage, ExtendedColorType, ImageEncoder, Rgba, RgbaImage};

/// The formats this tool writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Codec {
    Jpg,
    Png,
}

impl Codec {
    pub fn ext(self) -> &'static str {
        match self {
            Codec::Jpg => "jpg",
            Codec::Png => "png",
        }
    }

    /// Whether a quality number changes the result (PNG is always lossless).
    pub fn has_quality(self) -> bool {
        self == Codec::Jpg
    }

    pub fn encode(self, picture: &DynamicImage, quality: u8) -> Result<Vec<u8>, String> {
        match self {
            Codec::Jpg => jpg(picture, quality),
            Codec::Png => png(picture),
        }
    }
}

/// Whether any pixel is see-through (so a JPG would lose something).
pub fn has_transparency(picture: &DynamicImage) -> bool {
    picture.color().has_alpha() && picture.to_rgba8().pixels().any(|p| p.0[3] < 255)
}

/// Lays the picture on white: JPG has no transparency.
fn flattened(picture: &DynamicImage) -> image::RgbImage {
    if !picture.color().has_alpha() {
        return picture.to_rgb8();
    }
    let mut canvas = RgbaImage::from_pixel(picture.width(), picture.height(), Rgba([255, 255, 255, 255]));
    image::imageops::overlay(&mut canvas, &picture.to_rgba8(), 0, 0);
    DynamicImage::ImageRgba8(canvas).to_rgb8()
}

fn jpg(picture: &DynamicImage, quality: u8) -> Result<Vec<u8>, String> {
    let rgb = flattened(picture);
    let mut bytes = Vec::new();
    JpegEncoder::new_with_quality(&mut bytes, quality.clamp(1, 100))
        .write_image(rgb.as_raw(), rgb.width(), rgb.height(), ExtendedColorType::Rgb8)
        .map_err(|e| format!("cannot write the JPG: {e}"))?;
    Ok(bytes)
}

fn png(picture: &DynamicImage) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    let encoder = PngEncoder::new_with_quality(&mut bytes, CompressionType::Best, FilterType::Adaptive);
    let result = if picture.color().has_alpha() {
        let rgba = picture.to_rgba8();
        encoder.write_image(rgba.as_raw(), rgba.width(), rgba.height(), ExtendedColorType::Rgba8)
    } else {
        let rgb = picture.to_rgb8();
        encoder.write_image(rgb.as_raw(), rgb.width(), rgb.height(), ExtendedColorType::Rgb8)
    };
    result.map_err(|e| format!("cannot write the PNG: {e}"))?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Noise-like content, so quality visibly changes the size.
    pub fn busy(w: u32, h: u32) -> DynamicImage {
        DynamicImage::ImageRgb8(image::RgbImage::from_fn(w, h, |x, y| {
            let n = x.wrapping_mul(2654435761).wrapping_add(y.wrapping_mul(40503)) >> 7;
            image::Rgb([(n & 255) as u8, ((n >> 3) & 255) as u8, ((x + y) & 255) as u8])
        }))
    }

    #[test]
    fn lower_quality_makes_a_smaller_jpg() {
        let picture = busy(200, 200);
        let high = Codec::Jpg.encode(&picture, 95).unwrap().len();
        let low = Codec::Jpg.encode(&picture, 30).unwrap().len();
        assert!(low < high, "{low} should be below {high}");
    }

    #[test]
    fn png_ignores_quality() {
        let picture = busy(50, 50);
        assert_eq!(Codec::Png.encode(&picture, 10).unwrap(), Codec::Png.encode(&picture, 90).unwrap());
    }

    #[test]
    fn a_transparent_picture_becomes_white_in_a_jpg() {
        let picture = DynamicImage::ImageRgba8(RgbaImage::from_pixel(8, 8, Rgba([0, 0, 0, 0])));
        assert!(has_transparency(&picture));
        let bytes = Codec::Jpg.encode(&picture, 90).unwrap();
        let back = image::load_from_memory(&bytes).unwrap().to_rgb8();
        assert!(back.get_pixel(4, 4).0.iter().all(|c| *c > 245));
    }

    #[test]
    fn an_opaque_picture_with_an_alpha_channel_is_not_transparent() {
        assert!(!has_transparency(&DynamicImage::ImageRgba8(RgbaImage::from_pixel(4, 4, Rgba([1, 2, 3, 255])))));
    }
}
