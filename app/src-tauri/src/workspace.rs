//! The page editor window ("workspace"): opening it, and the commands it uses to
//! read pages and save the result. The editing itself happens in the window, on a
//! list of page references; see `engine::pdf::plan`.

use crate::engine::pdf::plan::{self, PageRef};
use crate::engine::pdf::thumbs::{self, PageShape};
use crate::engine::pdf::Library;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::ipc::Response;
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

const LABEL: &str = "workspace";

/// PDFium can garble results when several threads read pages at once (seen in tests),
/// and the editor asks for many thumbnails together, so its requests take turns.
static PDFIUM_TURN: Mutex<()> = Mutex::new(());

fn take_turn() -> std::sync::MutexGuard<'static, ()> {
    PDFIUM_TURN.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// The files the editor should show, set before the window loads so it can ask for them.
#[derive(Default)]
pub struct WorkspaceFiles(Mutex<Vec<String>>);

/// Opens the editor on these files, or hands them to the window that is already open.
pub fn open(app: &AppHandle, files: Vec<PathBuf>) {
    let paths: Vec<String> = files.iter().map(|f| f.to_string_lossy().into_owned()).collect();
    *app.state::<WorkspaceFiles>().0.lock().unwrap() = paths.clone();

    if let Some(window) = app.get_webview_window(LABEL) {
        let _ = app.emit("workspace-open", paths);
        let _ = window.show();
        let _ = window.set_focus();
        return;
    }
    let built = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("index.html#workspace".into()))
        .title("Organize pages")
        .inner_size(1120.0, 760.0)
        .min_inner_size(760.0, 480.0)
        .center()
        .build();
    if let Err(error) = built {
        eprintln!("[workspace] could not open the page editor: {error}");
    }
}

/// The files the window was opened for.
#[tauri::command]
pub fn workspace_files(app: AppHandle) -> Vec<String> {
    app.state::<WorkspaceFiles>().0.lock().unwrap().clone()
}

/// One opened file: its name and the shape of each page.
#[derive(Serialize)]
pub struct OpenedFile {
    path: String,
    name: String,
    pages: Vec<PageShape>,
}

#[tauri::command(async)]
pub fn workspace_open(path: String) -> Result<OpenedFile, String> {
    let _turn = take_turn();
    let file = PathBuf::from(&path);
    let library = Library::load()?;
    let pages = thumbs::shapes(library, &file, None)?;
    if pages.is_empty() {
        return Err("this PDF has no pages".into());
    }
    let name = file.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    Ok(OpenedFile { path, name, pages })
}

/// A page as a JPEG, sent as raw bytes (no text encoding of the picture).
#[tauri::command(async)]
pub fn workspace_thumbnail(path: String, page: u32, width: u32) -> Result<Response, String> {
    let _turn = take_turn();
    let library = Library::load()?;
    thumbs::thumbnail(library, &PathBuf::from(path), page, width).map(Response::new)
}

#[derive(Serialize)]
pub struct Saved {
    path: String,
    pages: u32,
}

/// Writes the planned pages to a new PDF next to the first file.
#[tauri::command(async)]
pub fn workspace_save(files: Vec<String>, pages: Vec<PageRef>) -> Result<Saved, String> {
    let _turn = take_turn();
    let inputs: Vec<PathBuf> = files.into_iter().map(PathBuf::from).collect();
    let library = Library::load()?;
    let (path, count) = plan::save_next_to_first(library, &inputs, &pages)?;
    Ok(Saved { path: path.to_string_lossy().into_owned(), pages: count })
}
