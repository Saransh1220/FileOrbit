//! Merging PDFs and combining pictures, with the real PDF engine.
//! Skips (passes) when the "PDF tools" pack is not installed.

use app_lib::engine::binaries::{self, Tool};
use app_lib::engine::jobs::merge_files;
use app_lib::engine::pdf::Library;
use pdfium_render::prelude::*;
use std::path::{Path, PathBuf};

/// PDFium is not safe to drive from several threads at once, so these tests take turns.
fn turn() -> std::sync::MutexGuard<'static, ()> {
    static TURN: std::sync::Mutex<()> = std::sync::Mutex::new(());
    TURN.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("radial-merge-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn missing() -> bool {
    let gone = binaries::find(Tool::Pdfium).is_err();
    if gone {
        eprintln!("skipping: the PDF tools pack is not installed");
    }
    gone
}

/// A PDF whose pages say "<label>1", "<label>2", ...
fn make_pdf(path: &Path, label: &str, pages: usize) {
    let library = Library::load().unwrap();
    let mut document = library.create().unwrap();
    for number in 1..=pages {
        let mut page = document.pages_mut().create_page_at_end(PdfPagePaperSize::a4()).unwrap();
        let font = document.fonts_mut().helvetica();
        page.objects_mut()
            .create_text_object(PdfPoints::new(72.0), PdfPoints::new(700.0), format!("{label}{number}"), font, PdfPoints::new(36.0))
            .unwrap();
    }
    document.save_to_file(path).unwrap();
}

/// The text of each page, in page order.
fn page_texts(path: &Path) -> Vec<String> {
    let library = Library::load().unwrap();
    let document = library.open(path, None).unwrap();
    document.pages().iter().map(|page| page.text().unwrap().all().trim().to_string()).collect()
}

fn picture(path: &Path, width: u32, height: u32) {
    image::RgbImage::from_pixel(width, height, image::Rgb([30, 90, 200])).save(path).unwrap();
}

#[test]
fn three_pdfs_merge_with_every_page_in_order() {
    let _turn = turn();
    if missing() {
        return;
    }
    let dir = scratch("three");
    make_pdf(&dir.join("a.pdf"), "A", 2);
    make_pdf(&dir.join("b.pdf"), "B", 1);
    make_pdf(&dir.join("c.pdf"), "C", 3);
    let inputs = vec![dir.join("a.pdf"), dir.join("b.pdf"), dir.join("c.pdf")];
    let (out, pages) = merge_files(&inputs).unwrap();
    assert_eq!(out, dir.join("a (merged).pdf"), "named after the first file, next to it");
    assert_eq!(pages, 6);
    assert_eq!(page_texts(&out), vec!["A1", "A2", "B1", "C1", "C2", "C3"]);
}

#[test]
fn the_order_given_is_the_order_used_not_alphabetical() {
    let _turn = turn();
    if missing() {
        return;
    }
    let dir = scratch("order");
    make_pdf(&dir.join("a.pdf"), "A", 1);
    make_pdf(&dir.join("b.pdf"), "B", 1);
    let (out, _) = merge_files(&[dir.join("b.pdf"), dir.join("a.pdf")]).unwrap();
    assert_eq!(page_texts(&out), vec!["B1", "A1"]);
}

#[test]
fn the_originals_are_not_touched_and_a_second_merge_gets_its_own_name() {
    let _turn = turn();
    if missing() {
        return;
    }
    let dir = scratch("originals");
    make_pdf(&dir.join("a.pdf"), "A", 2);
    make_pdf(&dir.join("b.pdf"), "B", 2);
    let before = (std::fs::read(dir.join("a.pdf")).unwrap(), std::fs::read(dir.join("b.pdf")).unwrap());
    let inputs = vec![dir.join("a.pdf"), dir.join("b.pdf")];
    let (first, _) = merge_files(&inputs).unwrap();
    let (second, _) = merge_files(&inputs).unwrap();
    assert_eq!(first, dir.join("a (merged).pdf"));
    assert_eq!(second, dir.join("a (merged) (1).pdf"));
    assert_eq!((std::fs::read(dir.join("a.pdf")).unwrap(), std::fs::read(dir.join("b.pdf")).unwrap()), before);
}

#[test]
fn a_picture_becomes_a_page_between_pdfs() {
    let _turn = turn();
    if missing() {
        return;
    }
    let dir = scratch("mixed");
    make_pdf(&dir.join("a.pdf"), "A", 1);
    picture(&dir.join("scan.png"), 400, 200);
    make_pdf(&dir.join("z.pdf"), "Z", 1);
    let (out, pages) = merge_files(&[dir.join("a.pdf"), dir.join("scan.png"), dir.join("z.pdf")]).unwrap();
    assert_eq!(pages, 3);
    let library = Library::load().unwrap();
    let document = library.open(&out, None).unwrap();
    let middle = document.pages().get(1).unwrap();
    let ratio = middle.width().value / middle.height().value;
    assert!((ratio - 2.0).abs() < 0.01, "the picture's page keeps its 2:1 shape, got {ratio}");
    assert_eq!(page_texts(&out)[0], "A1");
    assert_eq!(page_texts(&out)[2], "Z1");
}

#[test]
fn pictures_alone_combine_into_one_pdf_with_a_page_each() {
    let _turn = turn();
    if missing() {
        return;
    }
    let dir = scratch("combine");
    picture(&dir.join("one.png"), 300, 300);
    picture(&dir.join("two.png"), 600, 300);
    picture(&dir.join("three.png"), 300, 600);
    let (out, pages) = merge_files(&[dir.join("one.png"), dir.join("two.png"), dir.join("three.png")]).unwrap();
    assert_eq!(out, dir.join("one (merged).pdf"));
    assert_eq!(pages, 3);
}

#[test]
fn a_damaged_file_stops_the_merge_by_name_and_leaves_nothing_behind() {
    let _turn = turn();
    if missing() {
        return;
    }
    let dir = scratch("damaged");
    make_pdf(&dir.join("a.pdf"), "A", 1);
    std::fs::write(dir.join("broken.pdf"), "not a pdf").unwrap();
    let error = merge_files(&[dir.join("a.pdf"), dir.join("broken.pdf")]).unwrap_err();
    assert!(error.contains("broken.pdf") && error.contains("damaged"), "{error}");
    assert!(!dir.join("a (merged).pdf").exists(), "no half-written result");
}

#[test]
fn a_file_that_is_not_a_pdf_or_picture_is_refused_by_name() {
    let _turn = turn();
    if missing() {
        return;
    }
    let dir = scratch("refused");
    make_pdf(&dir.join("a.pdf"), "A", 1);
    std::fs::write(dir.join("song.mp3"), "x").unwrap();
    let error = merge_files(&[dir.join("a.pdf"), dir.join("song.mp3")]).unwrap_err();
    assert!(error.contains("song.mp3"), "{error}");
    assert!(!dir.join("a (merged).pdf").exists());
}

#[test]
fn merging_needs_at_least_two_files() {
    let _turn = turn();
    if missing() {
        return;
    }
    let dir = scratch("one");
    make_pdf(&dir.join("a.pdf"), "A", 1);
    assert!(merge_files(&[dir.join("a.pdf")]).unwrap_err().contains("at least two"));
    assert!(merge_files(&[]).is_err());
}

#[test]
fn a_file_that_disappeared_is_named() {
    let _turn = turn();
    if missing() {
        return;
    }
    let dir = scratch("gone");
    make_pdf(&dir.join("a.pdf"), "A", 1);
    let error = merge_files(&[dir.join("a.pdf"), dir.join("gone.pdf")]).unwrap_err();
    assert!(error.contains("gone.pdf"), "{error}");
}
