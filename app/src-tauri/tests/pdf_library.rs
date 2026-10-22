//! The PDF library loads from the managed pack and can create, save, open and
//! render a PDF. Skips (passes) when the "PDF tools" pack is not installed.

use app_lib::engine::binaries::{self, Tool};
use app_lib::engine::pdf::Library;
use pdfium_render::prelude::*;
use std::time::Instant;

fn installed() -> bool {
    let missing = binaries::find(Tool::Pdfium).is_err();
    if missing {
        eprintln!("skipping: the PDF tools pack is not installed");
    }
    !missing
}

#[test]
fn creates_saves_opens_and_renders_a_page() {
    if !installed() {
        return;
    }
    let started = Instant::now();
    let library = Library::load().expect("the library loads");
    eprintln!("[spike] library loaded in {:?}", started.elapsed());

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("hello.pdf");

    {
        let mut document = library.create().unwrap();
        let mut page = document.pages_mut().create_page_at_end(PdfPagePaperSize::a4()).unwrap();
        let font = document.fonts_mut().helvetica();
        page.objects_mut()
            .create_text_object(PdfPoints::new(72.0), PdfPoints::new(700.0), "Hello PDFium", font, PdfPoints::new(48.0))
            .unwrap();
        document.save_to_file(&path).unwrap();
    }
    assert!(std::fs::metadata(&path).unwrap().len() > 200, "a real file was written");

    let document = library.open(&path, None).unwrap();
    assert_eq!(document.pages().len(), 1);

    let rendered = Instant::now();
    let page = document.pages().get(0).unwrap();
    let bitmap = page.render_with_config(&PdfRenderConfig::new().set_target_width(800)).unwrap();
    let image = bitmap.as_image().expect("the bitmap converts to an image");
    eprintln!("[spike] rendered {}x{} in {:?}", image.width(), image.height(), rendered.elapsed());

    assert_eq!(image.width(), 800);
    // A4 is taller than wide.
    assert!(image.height() > image.width());
    // The page is not blank: the text left dark pixels on the white page.
    let pixels = image.to_luma8();
    let dark = pixels.pixels().filter(|p| p.0[0] < 100).count();
    let light = pixels.pixels().filter(|p| p.0[0] > 240).count();
    assert!(dark > 500, "expected the text to leave dark pixels, found {dark}");
    assert!(light > pixels.pixels().len() / 2, "most of the page should be white");
}

#[test]
fn a_file_that_is_not_a_pdf_gets_a_clear_message() {
    if !installed() {
        return;
    }
    let library = Library::load().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("fake.pdf");
    std::fs::write(&path, "this is not a pdf").unwrap();
    let error = library.open(&path, None).err().expect("opening fails");
    assert!(error.contains("fake.pdf") && error.contains("damaged"), "{error}");
}

/// The same thread loading the library again while it still holds it must carry on, and a
/// different thread must still wait until the first thread is completely done.
#[test]
fn loading_twice_on_one_thread_does_not_deadlock_but_other_threads_still_wait() {
    if binaries::find(Tool::Pdfium).is_err() {
        eprintln!("skipping: the PDF tools pack is not installed");
        return;
    }
    let (events, log) = std::sync::mpsc::channel::<&'static str>();
    let (inside, entered) = std::sync::mpsc::channel::<()>();

    let first = {
        let events = events.clone();
        std::thread::spawn(move || {
            let outer = Library::load().unwrap();
            inside.send(()).unwrap();
            let inner = Library::load().unwrap(); // would hang forever if it waited for itself
            events.send("first: nested load done").unwrap();
            // Give the other thread every chance to (wrongly) slip in while we still hold it.
            for _ in 0..50 {
                std::thread::yield_now();
            }
            drop(inner);
            drop(outer);
            events.send("first: released").unwrap();
        })
    };
    entered.recv().unwrap();
    let second = {
        let events = events.clone();
        std::thread::spawn(move || {
            let _held = Library::load().unwrap();
            events.send("second: got it").unwrap();
        })
    };
    first.join().unwrap();
    second.join().unwrap();
    drop(events);
    let order: Vec<_> = log.iter().collect();
    assert_eq!(order, ["first: nested load done", "first: released", "second: got it"]);
}
