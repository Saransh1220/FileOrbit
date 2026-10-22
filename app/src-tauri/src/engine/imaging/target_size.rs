//! "As good as possible, but under N bytes": try qualities, then smaller sizes.

use super::encode::Codec;
use super::resize;
use image::DynamicImage;

/// Below this a JPG looks bad enough that shrinking the picture is the better trade.
const MIN_QUALITY: u8 = 45;
const MAX_QUALITY: u8 = 92;
/// Sizes tried, as a percentage of the original, until the file fits.
const SCALES: &[u32] = &[100, 90, 80, 70, 60, 50, 40, 30, 20, 10];

/// The best attempt, and whether it fits.
pub struct Fitted {
    pub bytes: Vec<u8>,
    pub quality: u8,
    pub scale_percent: u32,
    pub reached: bool,
}

/// Highest quality in range whose JPG is no larger than `max_bytes`, or None if even the lowest is too big.
fn best_quality(picture: &DynamicImage, codec: Codec, max_bytes: u64) -> Result<Option<(Vec<u8>, u8)>, String> {
    let floor = codec.encode(picture, MIN_QUALITY)?;
    if floor.len() as u64 > max_bytes {
        return Ok(None);
    }
    let (mut low, mut high) = (MIN_QUALITY, MAX_QUALITY);
    let mut best = (floor, MIN_QUALITY);
    while low < high {
        let middle = low + (high - low).div_ceil(2);
        let bytes = codec.encode(picture, middle)?;
        if bytes.len() as u64 <= max_bytes {
            best = (bytes, middle);
            low = middle;
        } else {
            high = middle - 1;
        }
    }
    Ok(Some(best))
}

/// Finds the least damaging way to get under `max_bytes`: first quality alone (JPG),
/// then shrinking the picture. If nothing fits, returns the smallest attempt with `reached: false`.
pub fn fit(picture: &DynamicImage, codec: Codec, max_bytes: u64) -> Result<Fitted, String> {
    let mut smallest: Option<Fitted> = None;
    for &percent in SCALES {
        let candidate = if percent == 100 { picture.clone() } else { resize::shrunk(picture, percent) };
        if codec.has_quality() {
            if let Some((bytes, quality)) = best_quality(&candidate, codec, max_bytes)? {
                return Ok(Fitted { bytes, quality, scale_percent: percent, reached: true });
            }
        }
        let bytes = codec.encode(&candidate, MIN_QUALITY)?;
        if bytes.len() as u64 <= max_bytes {
            return Ok(Fitted { bytes, quality: MIN_QUALITY, scale_percent: percent, reached: true });
        }
        if smallest.as_ref().is_none_or(|s| bytes.len() < s.bytes.len()) {
            smallest = Some(Fitted { bytes, quality: MIN_QUALITY, scale_percent: percent, reached: false });
        }
        if candidate.width() <= 1 || candidate.height() <= 1 {
            break;
        }
    }
    smallest.ok_or_else(|| "nothing to compress".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgb, RgbImage};

    fn busy(w: u32, h: u32) -> DynamicImage {
        DynamicImage::ImageRgb8(RgbImage::from_fn(w, h, |x, y| {
            let n = x.wrapping_mul(2654435761).wrapping_add(y.wrapping_mul(40503)) >> 7;
            Rgb([(n & 255) as u8, ((n >> 3) & 255) as u8, ((x ^ y) & 255) as u8])
        }))
    }

    #[test]
    fn a_generous_limit_keeps_full_size_and_high_quality() {
        let picture = busy(200, 200);
        let fitted = fit(&picture, Codec::Jpg, 10_000_000).unwrap();
        assert!(fitted.reached);
        assert_eq!((fitted.scale_percent, fitted.quality), (100, MAX_QUALITY));
    }

    #[test]
    fn a_limit_reachable_by_quality_alone_does_not_shrink_the_picture() {
        let picture = busy(300, 300);
        let high = Codec::Jpg.encode(&picture, 85).unwrap().len() as u64;
        let fitted = fit(&picture, Codec::Jpg, high).unwrap();
        assert!(fitted.reached && fitted.bytes.len() as u64 <= high);
        assert_eq!(fitted.scale_percent, 100);
        assert!(fitted.quality <= 85);
    }

    #[test]
    fn a_tight_limit_shrinks_the_picture() {
        let picture = busy(400, 400);
        let floor = Codec::Jpg.encode(&picture, MIN_QUALITY).unwrap().len() as u64;
        let fitted = fit(&picture, Codec::Jpg, floor / 3).unwrap();
        assert!(fitted.reached);
        assert!(fitted.scale_percent < 100);
        assert!(fitted.bytes.len() as u64 <= floor / 3);
    }

    #[test]
    fn png_gets_smaller_only_by_shrinking() {
        let picture = busy(300, 300);
        let full = Codec::Png.encode(&picture, 90).unwrap().len() as u64;
        let fitted = fit(&picture, Codec::Png, full / 2).unwrap();
        assert!(fitted.reached && fitted.scale_percent < 100);
    }

    #[test]
    fn an_impossible_limit_says_so_and_returns_the_smallest_try() {
        let fitted = fit(&busy(100, 100), Codec::Jpg, 10).unwrap();
        assert!(!fitted.reached);
        assert!(!fitted.bytes.is_empty());
    }
}
