//! The transparent ring window that follows the cursor during a drag.

use crate::engine::{capabilities, registry};
use crate::engine::target::Target;
use serde::Serialize;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, LogicalPosition, Manager};

pub const OVERLAY_LABEL: &str = "overlay";
/// Must match the overlay window's width/height in tauri.conf.json.
const OVERLAY_SIZE: f64 = 264.0;

#[derive(Serialize, Clone)]
struct RingOpened {
    name: String,
    targets: Vec<Target>,
}

/// Wedges for the first dragged file; the ring converts each dropped file
/// by its own type, so mixed drops only show the first file's options.
fn ring_payload(paths: &[PathBuf]) -> RingOpened {
    let first = paths.first();
    let targets = first
        .and_then(|p| registry::normalized_ext(p))
        .map(|ext| registry::targets_for_filtered(&ext, capabilities::is_supported))
        .unwrap_or_default();
    let name = first
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    eprintln!("[overlay] paths={paths:?} -> {} targets", targets.len());
    RingOpened { name, targets }
}

pub fn show_at(app: &AppHandle, x: f64, y: f64, paths: &[PathBuf]) {
    let Some(win) = app.get_webview_window(OVERLAY_LABEL) else {
        return;
    };
    let half = OVERLAY_SIZE / 2.0;
    let _ = win.set_position(LogicalPosition::new(x - half, y - half));
    let _ = win.emit("ring-opened", ring_payload(paths));
    let _ = win.show();
}

pub fn notify_drag_ended(app: &AppHandle) {
    eprintln!("[overlay] drag ended -> notify ring");
    let _ = app.emit_to(OVERLAY_LABEL, "drag-ended", ());
}

pub fn hide(app: &AppHandle) {
    eprintln!("[overlay] hide");
    if let Some(win) = app.get_webview_window(OVERLAY_LABEL) {
        let _ = win.hide();
    }
}
