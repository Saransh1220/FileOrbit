//! The size maths, with no pictures involved so it can be tested exhaustively.

use super::spec::{Fit, Resize};

/// Largest side a result may have, so a typo cannot ask for a poster-sized allocation.
pub const MAX_SIDE: u32 = 20_000;

/// Where the pixels go. All sizes in pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    /// Resample to exactly this size.
    Scale { width: u32, height: u32 },
    /// Resample to `scaled`, then cut out `crop` = (x, y, width, height).
    Cover { scaled: (u32, u32), crop: (u32, u32, u32, u32) },
    /// Resample to `scaled`, then centre it on a `canvas` of this size.
    Contain { scaled: (u32, u32), canvas: (u32, u32), offset: (u32, u32) },
}

impl Layout {
    /// The size of the finished picture.
    pub fn result(&self) -> (u32, u32) {
        match *self {
            Layout::Scale { width, height } => (width, height),
            Layout::Cover { crop, .. } => (crop.2, crop.3),
            Layout::Contain { canvas, .. } => canvas,
        }
    }
}

fn at_least_one(v: f64) -> u32 {
    (v.round() as u32).max(1)
}

fn check(width: u32, height: u32) -> Result<(), String> {
    if width == 0 || height == 0 {
        return Err("a size cannot be zero".into());
    }
    if width > MAX_SIDE || height > MAX_SIDE {
        return Err(format!("a side cannot be longer than {MAX_SIDE} pixels"));
    }
    Ok(())
}

fn cover(source: (u32, u32), target: (u32, u32)) -> Layout {
    let scale = (target.0 as f64 / source.0 as f64).max(target.1 as f64 / source.1 as f64);
    let scaled = (
        ((source.0 as f64 * scale).ceil() as u32).max(target.0),
        ((source.1 as f64 * scale).ceil() as u32).max(target.1),
    );
    Layout::Cover { scaled, crop: ((scaled.0 - target.0) / 2, (scaled.1 - target.1) / 2, target.0, target.1) }
}

fn contain(source: (u32, u32), canvas: (u32, u32)) -> Layout {
    let scale = (canvas.0 as f64 / source.0 as f64).min(canvas.1 as f64 / source.1 as f64);
    let scaled = (at_least_one(source.0 as f64 * scale).min(canvas.0), at_least_one(source.1 as f64 * scale).min(canvas.1));
    Layout::Contain { scaled, canvas, offset: ((canvas.0 - scaled.0) / 2, (canvas.1 - scaled.1) / 2) }
}

fn shaped(source: (u32, u32), target: (u32, u32), fit: Fit) -> Layout {
    match fit {
        Fit::Stretch => Layout::Scale { width: target.0, height: target.1 },
        Fit::Crop => cover(source, target),
        Fit::Pad => contain(source, target),
    }
}

/// Shrinks `source` by `scale` (at most 1: these options never enlarge).
fn shrink(source: (u32, u32), scale: f64) -> Layout {
    let scale = scale.min(1.0);
    Layout::Scale { width: at_least_one(source.0 as f64 * scale), height: at_least_one(source.1 as f64 * scale) }
}

/// How `resize` changes a picture of `source` pixels.
pub fn plan(source: (u32, u32), resize: &Resize) -> Result<Layout, String> {
    check(source.0, source.1)?;
    let layout = match *resize {
        Resize::Percent { percent } => {
            if percent == 0 {
                return Err("the percentage cannot be zero".into());
            }
            let scale = percent as f64 / 100.0;
            Layout::Scale { width: at_least_one(source.0 as f64 * scale), height: at_least_one(source.1 as f64 * scale) }
        }
        Resize::Fit { width, height } => {
            check(width, height)?;
            shrink(source, (width as f64 / source.0 as f64).min(height as f64 / source.1 as f64))
        }
        Resize::LongEdge { pixels } => {
            check(pixels, pixels)?;
            shrink(source, pixels as f64 / source.0.max(source.1) as f64)
        }
        Resize::Exact { width, height, fit } => {
            check(width, height)?;
            shaped(source, (width, height), fit)
        }
        Resize::Aspect { width, height, fit } => {
            check(width, height)?;
            let ratio = width as f64 / height as f64;
            let (sw, sh) = (source.0 as f64, source.1 as f64);
            let target = match fit {
                // Crop to the biggest frame of this shape inside the picture; pad to the smallest one around it.
                Fit::Crop | Fit::Stretch => {
                    if sw / sh > ratio { (at_least_one(sh * ratio), source.1) } else { (source.0, at_least_one(sw / ratio)) }
                }
                Fit::Pad => {
                    if sw / sh > ratio { (source.0, at_least_one(sw / ratio)) } else { (at_least_one(sh * ratio), source.1) }
                }
            };
            match fit {
                Fit::Stretch => shaped(source, target, Fit::Crop),
                other => shaped(source, target, other),
            }
        }
    };
    let (w, h) = layout.result();
    check(w, h)?;
    Ok(layout)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn size(source: (u32, u32), resize: Resize) -> (u32, u32) {
        plan(source, &resize).unwrap().result()
    }

    #[test]
    fn percent_scales_both_sides() {
        assert_eq!(size((4000, 3000), Resize::Percent { percent: 50 }), (2000, 1500));
        assert_eq!(size((3, 3), Resize::Percent { percent: 10 }), (1, 1));
    }

    #[test]
    fn fit_keeps_proportions_and_never_enlarges() {
        assert_eq!(size((4000, 3000), Resize::Fit { width: 1600, height: 1600 }), (1600, 1200));
        assert_eq!(size((800, 600), Resize::Fit { width: 1600, height: 1600 }), (800, 600));
        assert_eq!(size((3000, 4000), Resize::Fit { width: 1000, height: 1000 }), (750, 1000));
    }

    #[test]
    fn long_edge_shrinks_the_longer_side() {
        assert_eq!(size((4000, 3000), Resize::LongEdge { pixels: 1000 }), (1000, 750));
        assert_eq!(size((300, 500), Resize::LongEdge { pixels: 1000 }), (300, 500));
    }

    #[test]
    fn exact_stretch_ignores_proportions() {
        assert_eq!(size((4000, 3000), Resize::Exact { width: 500, height: 500, fit: Fit::Stretch }), (500, 500));
    }

    #[test]
    fn exact_crop_fills_the_frame_and_cuts_the_overflow() {
        let layout = plan((4000, 3000), &Resize::Exact { width: 500, height: 500, fit: Fit::Crop }).unwrap();
        assert_eq!(layout, Layout::Cover { scaled: (667, 500), crop: (83, 0, 500, 500) });
        assert_eq!(layout.result(), (500, 500));
    }

    #[test]
    fn exact_pad_keeps_everything_and_centres_it() {
        let layout = plan((4000, 3000), &Resize::Exact { width: 500, height: 500, fit: Fit::Pad }).unwrap();
        assert_eq!(layout, Layout::Contain { scaled: (500, 375), canvas: (500, 500), offset: (0, 62) });
    }

    #[test]
    fn aspect_crop_keeps_the_pixels_and_trims_the_edges() {
        assert_eq!(size((4000, 3000), Resize::Aspect { width: 1, height: 1, fit: Fit::Crop }), (3000, 3000));
        assert_eq!(size((3000, 4000), Resize::Aspect { width: 16, height: 9, fit: Fit::Crop }), (3000, 1688));
    }

    #[test]
    fn aspect_pad_grows_the_canvas() {
        assert_eq!(size((4000, 3000), Resize::Aspect { width: 1, height: 1, fit: Fit::Pad }), (4000, 4000));
        assert_eq!(size((3000, 4000), Resize::Aspect { width: 16, height: 9, fit: Fit::Pad }), (7111, 4000));
    }

    #[test]
    fn a_square_into_the_same_shape_changes_nothing() {
        assert_eq!(size((500, 500), Resize::Aspect { width: 1, height: 1, fit: Fit::Crop }), (500, 500));
        assert_eq!(size((500, 500), Resize::Aspect { width: 1, height: 1, fit: Fit::Pad }), (500, 500));
    }

    #[test]
    fn nonsense_is_refused() {
        assert!(plan((100, 100), &Resize::Percent { percent: 0 }).is_err());
        assert!(plan((100, 100), &Resize::Fit { width: 0, height: 10 }).is_err());
        assert!(plan((100, 100), &Resize::Exact { width: 99_999, height: 10, fit: Fit::Crop }).is_err());
        assert!(plan((100, 100), &Resize::Percent { percent: 100_000 }).is_err());
    }
}
