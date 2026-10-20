//! Real PDF conversions through the same entry point the ring uses.
//! Each test skips (passes) when the converter it needs is not installed.

use app_lib::engine::binaries::{self, Tool};
use app_lib::engine::jobs::convert_file;
use app_lib::engine::pdf::Library;
use pdfium_render::prelude::*;
use std::path::{Path, PathBuf};

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("radial-pdf-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn missing(tool: Tool) -> bool {
    let gone = binaries::find(tool).is_err();
    if gone {
        eprintln!("skipping: {tool:?} not installed");
    }
    gone
}

/// A PDF whose page N says "Page N of <pages>", made with the library itself.
fn make_pdf(path: &Path, pages: usize) {
    let library = Library::load().unwrap();
    let mut document = library.create().unwrap();
    for number in 1..=pages {
        let mut page = document.pages_mut().create_page_at_end(PdfPagePaperSize::a4()).unwrap();
        let font = document.fonts_mut().helvetica();
        page.objects_mut()
            .create_text_object(PdfPoints::new(72.0), PdfPoints::new(700.0), format!("Page {number} of {pages}"), font, PdfPoints::new(36.0))
            .unwrap();
    }
    document.save_to_file(path).unwrap();
}

fn files_in(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir).unwrap().map(|e| e.unwrap().file_name().to_string_lossy().into_owned()).collect();
    names.sort();
    names
}

#[test]
fn a_one_page_pdf_becomes_one_png() {
    if missing(Tool::Pdfium) {
        return;
    }
    let dir = scratch("one-png");
    make_pdf(&dir.join("note.pdf"), 1);
    let out = convert_file(&dir.join("note.pdf"), "png", 1).unwrap();
    assert_eq!(out, dir.join("note.png"));
    let bytes = std::fs::read(&out).unwrap();
    assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
    let picture = image::load_from_memory(&bytes).unwrap();
    // A4 at 150 dpi.
    assert_eq!((picture.width(), picture.height()), (1240, 1754));
}

#[test]
fn a_many_page_pdf_becomes_a_folder_of_numbered_pictures() {
    if missing(Tool::Pdfium) {
        return;
    }
    let dir = scratch("many-png");
    make_pdf(&dir.join("report.pdf"), 3);
    let out = convert_file(&dir.join("report.pdf"), "png", 2).unwrap();
    assert_eq!(out, dir.join("report pages"));
    assert!(out.is_dir());
    assert_eq!(
        files_in(&out),
        vec!["report pages - page 01.png", "report pages - page 02.png", "report pages - page 03.png"]
    );
    // The empty file reserved for a single result must not be left beside the folder.
    assert_eq!(files_in(&dir), vec!["report pages", "report.pdf"]);
}

#[test]
fn a_second_conversion_gets_its_own_folder() {
    if missing(Tool::Pdfium) {
        return;
    }
    let dir = scratch("twice");
    make_pdf(&dir.join("report.pdf"), 2);
    let first = convert_file(&dir.join("report.pdf"), "png", 3).unwrap();
    let second = convert_file(&dir.join("report.pdf"), "png", 4).unwrap();
    assert_ne!(first, second);
    assert!(first.is_dir() && second.is_dir());
}

#[test]
fn a_pdf_becomes_a_jpeg() {
    if missing(Tool::Pdfium) {
        return;
    }
    let dir = scratch("jpg");
    make_pdf(&dir.join("note.pdf"), 1);
    let out = convert_file(&dir.join("note.pdf"), "jpg", 5).unwrap();
    assert_eq!(&std::fs::read(&out).unwrap()[..3], &[0xFF, 0xD8, 0xFF]);
}

#[test]
fn pdf_text_comes_out_page_by_page() {
    if missing(Tool::Pdfium) {
        return;
    }
    let dir = scratch("txt");
    make_pdf(&dir.join("note.pdf"), 3);
    let out = convert_file(&dir.join("note.pdf"), "txt", 6).unwrap();
    let text = std::fs::read_to_string(&out).unwrap();
    assert!(text.contains("Page 1 of 3") && text.contains("Page 2 of 3") && text.contains("Page 3 of 3"), "{text:?}");
    assert!(text.find("Page 1").unwrap() < text.find("Page 2").unwrap());
    assert_eq!(text.matches('\u{c}').count(), 2, "a page break between each pair of pages");
}

#[test]
fn a_picture_becomes_a_pdf_with_its_own_proportions() {
    if missing(Tool::Pdfium) {
        return;
    }
    let dir = scratch("picture");
    image::RgbImage::from_pixel(400, 200, image::Rgb([200, 30, 30])).save(dir.join("wide.png")).unwrap();
    let out = convert_file(&dir.join("wide.png"), "pdf", 7).unwrap();
    assert_eq!(out, dir.join("wide.pdf"));
    let library = Library::load().unwrap();
    let document = library.open(&out, None).unwrap();
    assert_eq!(document.pages().len(), 1);
    let page = document.pages().get(0).unwrap();
    let ratio = page.width().value / page.height().value;
    assert!((ratio - 2.0).abs() < 0.01, "the page keeps the picture's 2:1 shape, got {ratio}");
}

#[test]
fn a_pdf_that_is_only_a_picture_explains_there_is_no_text() {
    if missing(Tool::Pdfium) {
        return;
    }
    let dir = scratch("scan");
    image::RgbImage::from_pixel(100, 100, image::Rgb([255, 255, 255])).save(dir.join("scan.png")).unwrap();
    let pdf = convert_file(&dir.join("scan.png"), "pdf", 8).unwrap();
    let error = convert_file(&pdf, "txt", 9).unwrap_err();
    assert!(error.contains("no text") && error.contains("scan"), "{error}");
    // A failed conversion leaves nothing behind.
    assert!(!dir.join("scan.txt").exists());
}

#[test]
fn a_damaged_pdf_is_reported_by_name() {
    if missing(Tool::Pdfium) {
        return;
    }
    let dir = scratch("damaged");
    std::fs::write(dir.join("broken.pdf"), "not really a pdf").unwrap();
    let error = convert_file(&dir.join("broken.pdf"), "png", 10).unwrap_err();
    assert!(error.contains("broken.pdf") && error.contains("damaged"), "{error}");
    assert!(!dir.join("broken.png").exists());
}

#[test]
fn a_pdf_becomes_a_word_document_through_libreoffice() {
    if missing(Tool::Pdfium) || missing(Tool::LibreOffice) {
        return;
    }
    let dir = scratch("docx");
    make_pdf(&dir.join("note.pdf"), 2);
    let out = convert_file(&dir.join("note.pdf"), "docx", 11).unwrap();
    assert_eq!(out, dir.join("note.docx"));
    let mut archive = zip::ZipArchive::new(std::fs::File::open(&out).unwrap()).unwrap();
    let mut xml = String::new();
    std::io::Read::read_to_string(&mut archive.by_name("word/document.xml").unwrap(), &mut xml).unwrap();
    assert!(xml.contains("Page 2 of 2"), "the page text should be in the document");
}
