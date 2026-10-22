//! Opening a picture the way a viewer shows it: with the camera's rotation applied.
//! The `image` crate ignores the EXIF rotation tag, so a phone photo would come out sideways.

use image::{DynamicImage, ImageDecoder, ImageReader};
use std::path::Path;

pub fn open_upright(path: &Path) -> Result<DynamicImage, String> {
    let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let reader = ImageReader::open(path)
        .map_err(|e| format!("cannot open {name}: {e}"))?
        .with_guessed_format()
        .map_err(|e| format!("cannot read {name}: {e}"))?;
    let mut decoder = reader.into_decoder().map_err(|e| format!("cannot read the picture {name}: {e}"))?;
    let orientation = decoder.orientation().ok();
    let mut picture = DynamicImage::from_decoder(decoder).map_err(|e| format!("cannot read the picture {name}: {e}"))?;
    if let Some(orientation) = orientation {
        picture.apply_orientation(orientation);
    }
    Ok(picture)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgb, RgbImage};

    /// A JPEG of `w` x `h` with an EXIF block that says "turn 90 degrees clockwise to view" (orientation 6).
    fn sideways_jpeg(path: &Path, w: u32, h: u32) {
        let mut plain = Vec::new();
        DynamicImage::ImageRgb8(RgbImage::from_pixel(w, h, Rgb([10, 200, 30]))).write_to(&mut std::io::Cursor::new(&mut plain), image::ImageFormat::Jpeg).unwrap();
        let tiff: Vec<u8> = [
            b"II*\0".as_slice(), &[8, 0, 0, 0], &[1, 0], // header, first directory at 8, one entry
            &[0x12, 0x01, 3, 0, 1, 0, 0, 0, 6, 0, 0, 0], // tag 0x0112 (orientation), SHORT, 1 value: 6
            &[0, 0, 0, 0],
        ]
        .concat();
        let mut app1 = b"Exif\0\0".to_vec();
        app1.extend(tiff);
        let mut bytes = vec![0xFF, 0xD8, 0xFF, 0xE1];
        bytes.extend(((app1.len() + 2) as u16).to_be_bytes());
        bytes.extend(app1);
        bytes.extend(&plain[2..]);
        std::fs::write(path, bytes).unwrap();
    }

    #[test]
    fn a_photo_marked_sideways_opens_upright() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("phone.jpg");
        sideways_jpeg(&path, 40, 20);
        let picture = open_upright(&path).unwrap();
        assert_eq!((picture.width(), picture.height()), (20, 40));
    }

    #[test]
    fn a_picture_without_the_tag_is_left_alone() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("plain.png");
        RgbImage::from_pixel(40, 20, Rgb([1, 2, 3])).save(&path).unwrap();
        let picture = open_upright(&path).unwrap();
        assert_eq!((picture.width(), picture.height()), (40, 20));
    }
}
