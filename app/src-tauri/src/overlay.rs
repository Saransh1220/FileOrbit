//! The transparent ring window that follows the cursor during a drag.

use crate::engine::{capabilities, registry};
use crate::engine::target::Target;
use serde::Serialize;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager};

pub const OVERLAY_LABEL: &str = "overlay";
/// Must match the overlay window's width/height in tauri.conf.json.
const OVERLAY_SIZE: f64 = 264.0;

#[derive(Serialize, Clone)]
struct RingOpened {
    name: String,
    targets: Vec<Target>,
    /// The dragged files, so the page can start conversions on a drop.
    paths: Vec<String>,
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
    let paths = paths.iter().map(|p| p.to_string_lossy().into_owned()).collect();
    RingOpened { name, targets, paths }
}

#[cfg(target_os = "macos")]
pub fn show_at(app: &AppHandle, x: f64, y: f64, paths: &[PathBuf]) {
    use tauri::LogicalPosition;
    let Some(win) = app.get_webview_window(OVERLAY_LABEL) else {
        return;
    };
    let half = OVERLAY_SIZE / 2.0;
    let _ = win.set_position(LogicalPosition::new(x - half, y - half));
    let _ = win.emit("ring-opened", ring_payload(paths));
    let _ = win.show();
}

/// Windows: `(x, y)` are physical screen pixels, and each monitor can have
/// its own scale, so the window is sized and placed in physical pixels too.
#[cfg(target_os = "windows")]
pub fn show_at_physical(app: &AppHandle, x: f64, y: f64, paths: &[PathBuf]) {
    use tauri::{PhysicalPosition, PhysicalSize};
    let Some(win) = app.get_webview_window(OVERLAY_LABEL) else {
        return;
    };
    let scale = app.monitor_from_point(x, y).ok().flatten().map_or(1.0, |m| m.scale_factor());
    let size = (OVERLAY_SIZE * scale).round();
    let _ = win.set_size(PhysicalSize::new(size as u32, size as u32));
    let _ = win.set_position(PhysicalPosition::new((x - size / 2.0).round() as i32, (y - size / 2.0).round() as i32));
    let _ = win.emit("ring-opened", ring_payload(paths));
    // Window and OLE calls belong on the UI thread, and the window is only
    // really visible once that thread has shown it.
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        crate::overlay_native::show_without_activating(&win);
        crate::overlay_catcher::cover(&handle, &win);
    });
}

pub fn notify_drag_ended(app: &AppHandle) {
    eprintln!("[overlay] drag ended -> notify ring");
    let _ = app.emit_to(OVERLAY_LABEL, "drag-ended", ());
}

pub fn hide(app: &AppHandle) {
    eprintln!("[overlay] hide");
    #[cfg(target_os = "windows")]
    let _ = app.run_on_main_thread(crate::overlay_catcher::uncover);
    if let Some(win) = app.get_webview_window(OVERLAY_LABEL) {
        let _ = win.hide();
    }
}
