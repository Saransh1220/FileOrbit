//! What the window shows about a picture before anything is done to it.

use super::orient;
use image::{ImageDecoder, ImageReader};
use serde::Serialize;
use std::path::Path;

/// A picture as listed: its name and its size as a viewer shows it (camera rotation applied).
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Info {
    pub path: String,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub bytes: u64,
}

/// Reads the size from the file header, without decoding the picture.
pub fn read(path: &Path) -> Result<Info, String> {
    let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let reader = ImageReader::open(path).map_err(|e| format!("cannot open {name}: {e}"))?.with_guessed_format().map_err(|e| format!("cannot read {name}: {e}"))?;
    let mut decoder = reader.into_decoder().map_err(|e| format!("{name} is not a picture this tool can read: {e}"))?;
    let (mut width, mut height) = decoder.dimensions();
    // A quarter turn (orientation 5 to 8) swaps the sides.
    if let Ok(o) = decoder.orientation() {
        if matches!(o, image::metadata::Orientation::Rotate90 | image::metadata::Orientation::Rotate270 | image::metadata::Orientation::Rotate90FlipH | image::metadata::Orientation::Rotate270FlipH) {
            std::mem::swap(&mut width, &mut height);
        }
    }
    let bytes = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    Ok(Info { path: path.to_string_lossy().into_owned(), name, width, height, bytes })
}

/// A small JPEG of the picture, for lists and previews, no wider or taller than `longest` pixels.
pub fn thumbnail(path: &Path, longest: u32) -> Result<Vec<u8>, String> {
    let picture = orient::open_upright(path)?;
    let small = picture.thumbnail(longest.clamp(16, 2000), longest.clamp(16, 2000));
    super::encode::Codec::Jpg.encode(&small, 80)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_size_without_decoding() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.png");
        image::RgbImage::new(30, 20).save(&path).unwrap();
        let info = read(&path).unwrap();
        assert_eq!((info.width, info.height, info.name.as_str()), (30, 20, "a.png"));
        assert!(info.bytes > 0);
    }

    #[test]
    fn a_text_file_is_refused_with_its_name() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("notes.png");
        std::fs::write(&path, "not a picture").unwrap();
        assert!(read(&path).unwrap_err().contains("notes.png"));
    }

    #[test]
    fn a_thumbnail_is_a_small_jpeg() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("big.png");
        image::RgbImage::new(800, 400).save(&path).unwrap();
        let bytes = thumbnail(&path, 100).unwrap();
        let back = image::load_from_memory(&bytes).unwrap();
        assert_eq!((back.width(), back.height()), (100, 50));
    }
}
