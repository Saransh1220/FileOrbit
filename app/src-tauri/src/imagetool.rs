//! The picture tool window ("imagetool"): opening it, and the commands it uses to list
//! pictures, preview a result and save. The work itself is in `engine::imaging`.

use crate::engine::imaging::info::{self, Info};
use crate::engine::imaging::pipeline::{self, Outcome};
use crate::engine::imaging::spec::Job;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::ipc::Response;
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

const LABEL: &str = "imagetool";

/// The pictures the window was opened for, set before it loads so it can ask for them.
#[derive(Default)]
pub struct ImageToolFiles(Mutex<Vec<String>>);

/// Opens the window on these pictures, or hands them to the window that is already open.
pub fn open(app: &AppHandle, files: Vec<PathBuf>) {
    let paths: Vec<String> = files.iter().map(|f| f.to_string_lossy().into_owned()).collect();
    *app.state::<ImageToolFiles>().0.lock().unwrap() = paths.clone();

    if let Some(window) = app.get_webview_window(LABEL) {
        let _ = app.emit("imagetool-open", paths);
        crate::workspace::bring_to_front(&window);
        return;
    }
    let built = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("index.html#imagetool".into()))
        .title("Resize and compress")
        .inner_size(1100.0, 720.0)
        .min_inner_size(900.0, 560.0)
        .center()
        .focused(true)
        .build();
    match built {
        Ok(window) => crate::workspace::bring_to_front(&window),
        Err(error) => eprintln!("[imagetool] could not open the window: {error}"),
    }
}

#[tauri::command]
pub fn imagetool_files(app: AppHandle) -> Vec<String> {
    app.state::<ImageToolFiles>().0.lock().unwrap().clone()
}

/// Each path's size and name; a picture that cannot be read is reported by name instead of failing the rest.
#[derive(Serialize)]
pub struct Listed {
    pictures: Vec<Info>,
    errors: Vec<String>,
}

#[tauri::command(async)]
pub fn imagetool_info(paths: Vec<String>) -> Listed {
    let mut listed = Listed { pictures: Vec::new(), errors: Vec::new() };
    for path in paths {
        match info::read(&PathBuf::from(path)) {
            Ok(picture) => listed.pictures.push(picture),
            Err(message) => listed.errors.push(message),
        }
    }
    listed
}

/// A small JPEG of the original, as raw bytes.
#[tauri::command(async)]
pub fn imagetool_thumbnail(path: String, longest: u32) -> Result<Response, String> {
    info::thumbnail(&PathBuf::from(path), longest).map(Response::new)
}

#[derive(Serialize)]
struct PreviewHeader {
    width: u32,
    height: u32,
    bytes: u64,
    notes: Vec<String>,
}

/// Does the job in memory and returns what it would save: a 4-byte big-endian length,
/// that many bytes of JSON (`PreviewHeader`), then the picture's own bytes (JPG or PNG).
#[tauri::command(async)]
pub fn imagetool_preview(path: String, job: Job) -> Result<Response, String> {
    let rendered = pipeline::render(&PathBuf::from(path), &job)?;
    let header = serde_json::to_vec(&PreviewHeader { width: rendered.width, height: rendered.height, bytes: rendered.bytes.len() as u64, notes: rendered.notes })
        .map_err(|e| e.to_string())?;
    let mut out = Vec::with_capacity(4 + header.len() + rendered.bytes.len());
    out.extend((header.len() as u32).to_be_bytes());
    out.extend(header);
    out.extend(rendered.bytes);
    Ok(Response::new(out))
}

#[derive(Serialize)]
pub struct Saved {
    saved: Vec<Outcome>,
    errors: Vec<String>,
}

/// Saves every picture next to its original; one failing does not stop the others.
#[tauri::command(async)]
pub fn imagetool_save(app: AppHandle, paths: Vec<String>, job: Job) -> Saved {
    let total = paths.len();
    let mut result = Saved { saved: Vec::new(), errors: Vec::new() };
    for (done, path) in paths.into_iter().enumerate() {
        let _ = app.emit("imagetool-progress", (done, total));
        match pipeline::run(&PathBuf::from(&path), &job) {
            Ok(outcome) => result.saved.push(outcome),
            Err(message) => {
                eprintln!("[imagetool] {path}: {message}");
                result.errors.push(message);
            }
        }
    }
    result
}
