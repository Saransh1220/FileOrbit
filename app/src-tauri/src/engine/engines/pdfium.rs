//! Conversions done by PDFium: PDF pages to pictures and text, and a picture to a PDF.

use crate::engine::pdf::{pictures, render, text, Library};
use std::path::{Path, PathBuf};

/// Runs one PDFium conversion. Returns what it wrote: a file, or a folder when a
/// many-page PDF became many pictures.
pub fn convert(input: &Path, from_ext: &str, output: &Path, to_ext: &str) -> Result<PathBuf, String> {
    let library = Library::load()?;
    match (from_ext, to_ext) {
        ("pdf", "png" | "jpg") => pages_to_pictures(&library, input, output, to_ext),
        ("pdf", "txt") => {
            let document = library.open(input, None)?;
            let contents = text::extract(&document)?;
            std::fs::write(output, contents).map_err(|e| format!("cannot write {}: {e}", output.display()))?;
            Ok(output.to_path_buf())
        }
        (_, "pdf") => {
            pictures::picture_to_pdf(&library, input, output)?;
            Ok(output.to_path_buf())
        }
        _ => Err(format!("cannot convert .{from_ext} to .{to_ext} here")),
    }
}

/// One page becomes one file at `output`. Several pages become a folder of numbered
/// pictures next to it, because one path cannot hold many images.
fn pages_to_pictures(library: &Library, input: &Path, output: &Path, to_ext: &str) -> Result<PathBuf, String> {
    let document = library.open(input, None)?;
    let count = document.pages().len() as usize;
    if count == 0 {
        return Err("this PDF has no pages".into());
    }
    if count == 1 {
        save_picture(&render::render_page(&document.pages().get(0).map_err(|e| e.to_string())?)?, output, to_ext)?;
        return Ok(output.to_path_buf());
    }

    // The empty file reserved for a single result is not needed; the folder takes its place.
    let _ = std::fs::remove_file(output);
    let folder = make_folder(output)?;
    let result = (|| -> Result<(), String> {
        let digits = count.to_string().len().max(2);
        let stem = folder.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        for (index, page) in document.pages().iter().enumerate() {
            let file = folder.join(format!("{stem} - page {:0width$}.{to_ext}", index + 1, width = digits));
            save_picture(&render::render_page(&page)?, &file, to_ext)?;
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_dir_all(&folder);
    }
    result.map(|()| folder)
}

/// A new, empty folder next to `output`, named after it: "report pages", then "report pages (1)".
fn make_folder(output: &Path) -> Result<PathBuf, String> {
    let dir = output.parent().unwrap_or_else(|| Path::new("."));
    let stem = output.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    for attempt in 0..10_000u32 {
        let name = if attempt == 0 { format!("{stem} pages") } else { format!("{stem} pages ({attempt})") };
        let candidate = dir.join(name);
        match std::fs::create_dir(&candidate) {
            Ok(()) => return Ok(candidate),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(format!("cannot create {}: {e}", candidate.display())),
        }
    }
    Err("too many folders with that name".into())
}

fn save_picture(picture: &image::DynamicImage, path: &Path, ext: &str) -> Result<(), String> {
    match ext {
        "jpg" => {
            // JPEG has no transparency; the page is already on white, so dropping alpha is safe.
            let file = std::fs::File::create(path).map_err(|e| format!("cannot create {}: {e}", path.display()))?;
            let mut writer = std::io::BufWriter::new(file);
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut writer, 90);
            picture.to_rgb8().write_with_encoder(encoder).map_err(|e| format!("cannot write {}: {e}", path.display()))
        }
        _ => picture.save_with_format(path, image::ImageFormat::Png).map_err(|e| format!("cannot write {}: {e}", path.display())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folders_are_numbered_when_the_name_is_taken() {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("report.png");
        let first = make_folder(&output).unwrap();
        let second = make_folder(&output).unwrap();
        assert_eq!(first, dir.path().join("report pages"));
        assert_eq!(second, dir.path().join("report pages (1)"));
    }
}
