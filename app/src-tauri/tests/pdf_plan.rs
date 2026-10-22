//! Saving a page plan and drawing thumbnails, with the real PDF engine.
//! Skips (passes) when the "PDF tools" pack is not installed.

use app_lib::engine::binaries::{self, Tool};
use app_lib::engine::pdf::plan::{save_next_to_first as save, save_part, PageRef, SaveKind};
use app_lib::engine::pdf::thumbs;
use app_lib::engine::pdf::Library;
use pdfium_render::prelude::*;
use std::path::{Path, PathBuf};

/// PDFium is not safe to drive from several threads at once, so these tests take turns.
fn turn() -> std::sync::MutexGuard<'static, ()> {
    static TURN: std::sync::Mutex<()> = std::sync::Mutex::new(());
    TURN.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("radial-plan-{}-{name}", std::process::id()));
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

fn page_texts(path: &Path) -> Vec<String> {
    let library = Library::load().unwrap();
    let document = library.open(path, None).unwrap();
    document.pages().iter().map(|page| page.text().unwrap().all().trim().to_string()).collect()
}

fn page(source: usize, page: u32, turn: u16) -> PageRef {
    PageRef { source, page, turn, blank: None }
}

#[test]
fn pages_can_be_reordered_dropped_and_repeated() {
    let _turn = turn();
    if missing() {
        return;
    }
    let dir = scratch("reorder");
    make_pdf(&dir.join("a.pdf"), "A", 4);
    let library = Library::load().unwrap();
    let plan = [page(0, 3, 0), page(0, 0, 0), page(0, 0, 0), page(0, 2, 0)];
    let (out, count) = save(&library, &[dir.join("a.pdf")], &plan, SaveKind::Edited).unwrap();
    assert_eq!(out, dir.join("a (edited).pdf"));
    assert_eq!(count, 4);
    assert_eq!(page_texts(&out), vec!["A4", "A1", "A1", "A3"]);
}

#[test]
fn rotation_is_added_to_what_the_page_already_has() {
    let _turn = turn();
    if missing() {
        return;
    }
    let dir = scratch("rotate");
    make_pdf(&dir.join("a.pdf"), "A", 2);
    let library = Library::load().unwrap();
    let (once, _) = save(&library, &[dir.join("a.pdf")], &[page(0, 0, 90), page(0, 1, 0)], SaveKind::Edited).unwrap();
    // Turn the already turned page by another quarter: 180 in all.
    let (twice, _) = save(&library, &[once.clone()], &[page(0, 0, 90), page(0, 1, 270)], SaveKind::Edited).unwrap();

    let rotations = |path: &Path| -> Vec<f32> {
        let document = library.open(path, None).unwrap();
        document.pages().iter().map(|p| p.rotation().unwrap().as_degrees()).collect()
    };
    assert_eq!(rotations(&once), vec![90.0, 0.0]);
    assert_eq!(rotations(&twice), vec![180.0, 270.0]);
    // A turned A4 page is wider than it is tall.
    let document = library.open(&once, None).unwrap();
    let turned = document.pages().get(0).unwrap();
    assert!(turned.width().value > turned.height().value);
}

#[test]
fn a_bad_plan_writes_nothing() {
    let _turn = turn();
    if missing() {
        return;
    }
    let dir = scratch("bad");
    make_pdf(&dir.join("a.pdf"), "A", 2);
    let library = Library::load().unwrap();
    let error = save(&library, &[dir.join("a.pdf")], &[page(0, 5, 0)], SaveKind::Edited).unwrap_err();
    assert!(error.contains("page 1"), "{error}");
    assert!(!dir.join("a (edited).pdf").exists(), "no half-written result");
    assert!(save(&library, &[dir.join("a.pdf")], &[], SaveKind::Edited).is_err());
}

#[test]
fn pages_from_two_files_can_be_woven_together() {
    let _turn = turn();
    if missing() {
        return;
    }
    let dir = scratch("weave");
    make_pdf(&dir.join("a.pdf"), "A", 2);
    make_pdf(&dir.join("b.pdf"), "B", 2);
    let library = Library::load().unwrap();
    let plan = [page(0, 0, 0), page(1, 0, 0), page(0, 1, 0), page(1, 1, 0)];
    let (out, _) = save(&library, &[dir.join("a.pdf"), dir.join("b.pdf")], &plan, SaveKind::Edited).unwrap();
    assert_eq!(page_texts(&out), vec!["A1", "B1", "A2", "B2"]);
}

#[test]
fn page_shapes_and_thumbnails_come_back() {
    let _turn = turn();
    if missing() {
        return;
    }
    let dir = scratch("thumbs");
    make_pdf(&dir.join("a.pdf"), "A", 3);
    let library = Library::load().unwrap();
    let shapes = thumbs::shapes(&library, &dir.join("a.pdf"), None).unwrap();
    assert_eq!(shapes.len(), 3);
    assert!(shapes[0].height > shapes[0].width, "A4 is taller than wide");

    let bytes = thumbs::thumbnail(&library, &dir.join("a.pdf"), 1, 200).unwrap();
    assert_eq!(&bytes[..2], &[0xFF, 0xD8], "a JPEG starts with FF D8");
    let picture = image::load_from_memory(&bytes).unwrap();
    assert_eq!(picture.width(), 200);
    assert!(picture.height() > 250, "keeps the page's shape");
    assert!(thumbs::thumbnail(&library, &dir.join("a.pdf"), 9, 200).is_err());
}

#[test]
fn a_blank_page_is_inserted_with_the_asked_size_and_a_selection_can_be_extracted() {
    let _turn = turn();
    if missing() {
        return;
    }
    let dir = scratch("blank");
    make_pdf(&dir.join("a.pdf"), "A", 3);
    let library = Library::load().unwrap();
    let blank = PageRef { source: 0, page: 0, turn: 0, blank: Some([300.0, 200.0]) };
    let plan = [page(0, 0, 0), blank, page(0, 2, 0)];
    let (out, count) = save(&library, &[dir.join("a.pdf")], &plan, SaveKind::Edited).unwrap();
    assert_eq!(count, 3);
    let document = library.open(&out, None).unwrap();
    let middle = document.pages().get(1).unwrap();
    assert!((middle.width().value - 300.0).abs() < 0.5 && (middle.height().value - 200.0).abs() < 0.5);
    assert_eq!(page_texts(&out), vec!["A1", "", "A3"]);

    let (picked, _) = save(&library, &[dir.join("a.pdf")], &[page(0, 1, 0)], SaveKind::Extract).unwrap();
    assert_eq!(picked, dir.join("a (extract).pdf"));
    assert_eq!(page_texts(&picked), vec!["A2"]);
}

#[test]
fn split_parts_are_numbered_next_to_the_original() {
    let _turn = turn();
    if missing() {
        return;
    }
    let dir = scratch("parts");
    make_pdf(&dir.join("a.pdf"), "A", 4);
    let library = Library::load().unwrap();
    let inputs = [dir.join("a.pdf")];
    let (one, _) = save_part(&library, &inputs, &[page(0, 0, 0), page(0, 1, 0)], 1).unwrap();
    let (two, _) = save_part(&library, &inputs, &[page(0, 2, 0), page(0, 3, 0)], 2).unwrap();
    assert_eq!((one.file_name().unwrap().to_str().unwrap(), two.file_name().unwrap().to_str().unwrap()), ("a (part 1).pdf", "a (part 2).pdf"));
    assert_eq!(page_texts(&two), vec!["A3", "A4"]);
}
