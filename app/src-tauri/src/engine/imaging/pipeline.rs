//! The whole job on one file: open, resize, compress, write next to the original.

use super::encode::{self, Codec};
use super::spec::{Compress, Job, OutFormat};
use super::{dimensions, orient, resize, target_size};
use crate::engine::{naming, selection};
use image::{DynamicImage, Rgba};
use serde::Serialize;
use std::path::{Path, PathBuf};

/// JPG quality when the user chose no quality.
const DEFAULT_QUALITY: u8 = 90;

/// A finished picture, not yet written.
pub struct Rendered {
    pub bytes: Vec<u8>,
    pub codec: Codec,
    pub width: u32,
    pub height: u32,
    /// Anything the user should know, such as a size limit that could not be met.
    pub notes: Vec<String>,
}

/// What `run` reports for one file.
#[derive(Debug, Serialize)]
pub struct Outcome {
    pub output: String,
    pub before_bytes: u64,
    pub after_bytes: u64,
    pub width: u32,
    pub height: u32,
    pub notes: Vec<String>,
}

/// Which format the result is in: what was asked, or the source's own where possible.
fn codec_for(source: &Path, picture: &DynamicImage, format: OutFormat) -> (Codec, Option<String>) {
    match format {
        OutFormat::Jpg => (Codec::Jpg, None),
        OutFormat::Png => (Codec::Png, None),
        OutFormat::Keep => match crate::engine::registry::normalized_ext(source).as_deref() {
            Some("jpg") => (Codec::Jpg, None),
            Some("png") => (Codec::Png, None),
            other => {
                let codec = if encode::has_transparency(picture) { Codec::Png } else { Codec::Jpg };
                let name = other.unwrap_or("this").to_uppercase();
                (codec, Some(format!("{name} files are saved as {}.", codec.ext().to_uppercase())))
            }
        },
    }
}

/// Does the job in memory. The preview and the real run both use this, so they agree.
pub fn render(source: &Path, job: &Job) -> Result<Rendered, String> {
    let mut picture = orient::open_upright(source)?;
    let (codec, mut note) = codec_for(source, &picture, job.format);
    let mut notes: Vec<String> = note.take().into_iter().collect();

    if let Some(change) = &job.resize {
        let layout = dimensions::plan((picture.width(), picture.height()), change)?;
        // A padded JPG gets white bars; a PNG keeps them see-through.
        let background = if codec == Codec::Jpg { Rgba([255, 255, 255, 255]) } else { Rgba([0, 0, 0, 0]) };
        picture = resize::apply(&picture, &layout, background);
    }
    if codec == Codec::Jpg && encode::has_transparency(&picture) {
        notes.push("Transparent areas become white in a JPG.".into());
    }

    let bytes = match &job.compress {
        Some(Compress::TargetSize { max_bytes }) => {
            let fitted = target_size::fit(&picture, codec, *max_bytes)?;
            if !fitted.reached {
                notes.push("Could not get under that size, this is the smallest it goes.".into());
            } else if fitted.scale_percent < 100 {
                notes.push(format!("Shrunk to {}% of the size to fit.", fitted.scale_percent));
            }
            if fitted.scale_percent < 100 {
                picture = resize::shrunk(&picture, fitted.scale_percent);
            }
            fitted.bytes
        }
        Some(Compress::Quality { quality }) => {
            if !codec.has_quality() {
                notes.push("PNG has no quality setting; choose JPG or a size limit to make it smaller.".into());
            }
            codec.encode(&picture, *quality)?
        }
        None => codec.encode(&picture, DEFAULT_QUALITY)?,
    };
    Ok(Rendered { bytes, codec, width: picture.width(), height: picture.height(), notes })
}

/// Does the job and writes the result next to `source` under a new name.
pub fn run(source: &Path, job: &Job) -> Result<Outcome, String> {
    let name = source.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let ext = crate::engine::registry::normalized_ext(source).unwrap_or_default();
    if !selection::is_picture(&ext) && ext != "jpeg" {
        return Err(format!("{name} is not a picture this tool can read"));
    }
    let rendered = render(source, job)?;
    let output: PathBuf = naming::reserve_with_suffix(source, job.suffix(), rendered.codec.ext())?;
    if let Err(e) = std::fs::write(&output, &rendered.bytes) {
        let _ = std::fs::remove_file(&output);
        return Err(format!("cannot write {}: {e}", output.display()));
    }
    Ok(Outcome {
        output: output.to_string_lossy().into_owned(),
        before_bytes: std::fs::metadata(source).map(|m| m.len()).unwrap_or(0),
        after_bytes: rendered.bytes.len() as u64,
        width: rendered.width,
        height: rendered.height,
        notes: rendered.notes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::imaging::spec::{Fit, Resize};
    use image::{Rgb, RgbImage};

    fn write_busy(dir: &Path, name: &str, w: u32, h: u32) -> PathBuf {
        let picture = DynamicImage::ImageRgb8(RgbImage::from_fn(w, h, |x, y| {
            let n = x.wrapping_mul(2654435761).wrapping_add(y.wrapping_mul(40503)) >> 7;
            Rgb([(n & 255) as u8, ((n >> 3) & 255) as u8, ((x ^ y) & 255) as u8])
        }));
        let path = dir.join(name);
        picture.save(&path).unwrap();
        path
    }

    fn job(resize: Option<Resize>, compress: Option<Compress>, format: OutFormat) -> Job {
        Job { resize, compress, format }
    }

    #[test]
    fn resize_alone_writes_a_new_file_and_keeps_the_original() {
        let dir = tempfile::tempdir().unwrap();
        let source = write_busy(dir.path(), "photo.jpg", 400, 300);
        let out = run(&source, &job(Some(Resize::LongEdge { pixels: 200 }), None, OutFormat::Keep)).unwrap();
        assert!(out.output.ends_with("photo (resized).jpg"));
        assert_eq!((out.width, out.height), (200, 150));
        assert!(source.exists());
    }

    #[test]
    fn a_second_run_numbers_the_name_instead_of_overwriting() {
        let dir = tempfile::tempdir().unwrap();
        let source = write_busy(dir.path(), "photo.png", 100, 100);
        let j = job(None, Some(Compress::Quality { quality: 50 }), OutFormat::Jpg);
        let first = run(&source, &j).unwrap();
        let second = run(&source, &j).unwrap();
        assert!(first.output.ends_with("photo (compressed).jpg"));
        assert!(second.output.ends_with("photo (compressed) (1).jpg"));
    }

    #[test]
    fn target_size_is_met_and_resizing_combines_with_it() {
        let dir = tempfile::tempdir().unwrap();
        let source = write_busy(dir.path(), "big.jpg", 800, 800);
        let j = job(
            Some(Resize::Exact { width: 600, height: 600, fit: Fit::Crop }),
            Some(Compress::TargetSize { max_bytes: 20_000 }),
            OutFormat::Keep,
        );
        let out = run(&source, &j).unwrap();
        assert!(out.after_bytes <= 20_000, "{} bytes", out.after_bytes);
        assert!(std::fs::metadata(&out.output).unwrap().len() <= 20_000);
        assert!(out.output.ends_with("big (resized, compressed).jpg"));
    }

    #[test]
    fn an_unreachable_limit_is_reported_in_the_notes() {
        let dir = tempfile::tempdir().unwrap();
        let source = write_busy(dir.path(), "p.jpg", 100, 100);
        let out = run(&source, &job(None, Some(Compress::TargetSize { max_bytes: 5 }), OutFormat::Keep)).unwrap();
        assert!(out.notes.iter().any(|n| n.contains("smallest")));
    }

    #[test]
    fn png_with_a_quality_says_it_has_none() {
        let dir = tempfile::tempdir().unwrap();
        let source = write_busy(dir.path(), "p.png", 60, 60);
        let out = run(&source, &job(None, Some(Compress::Quality { quality: 40 }), OutFormat::Keep)).unwrap();
        assert!(out.notes.iter().any(|n| n.contains("PNG has no quality")));
    }

    #[test]
    fn a_failed_job_leaves_no_file_behind() {
        let dir = tempfile::tempdir().unwrap();
        let source = write_busy(dir.path(), "p.png", 60, 60);
        assert!(run(&source, &job(Some(Resize::Percent { percent: 0 }), None, OutFormat::Keep)).is_err());
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    #[test]
    fn something_that_is_not_a_picture_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("notes.txt");
        std::fs::write(&path, "hi").unwrap();
        assert!(run(&path, &job(None, None, OutFormat::Jpg)).is_err());
    }
}
