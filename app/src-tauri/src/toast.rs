//! The small progress/result window in the top-right corner.

use crate::engine::jobs::JobUpdate;
use tauri::{AppHandle, Emitter, LogicalPosition, Manager};

pub const TOAST_LABEL: &str = "toast";
/// Must match the toast window width in tauri.conf.json.
const TOAST_WIDTH: f64 = 420.0;
const MARGIN: f64 = 16.0;
const MENU_BAR: f64 = 36.0;

pub fn publish(app: &AppHandle, update: &JobUpdate) {
    let Some(win) = app.get_webview_window(TOAST_LABEL) else {
        return;
    };
    if let Ok(Some(monitor)) = app.primary_monitor() {
        let width = monitor.size().width as f64 / monitor.scale_factor();
        let _ = win.set_position(LogicalPosition::new(width - TOAST_WIDTH - MARGIN, MENU_BAR));
    }
    let _ = win.show();
    let _ = win.emit("job-update", update);
}

pub fn hide(app: &AppHandle) {
    if let Some(win) = app.get_webview_window(TOAST_LABEL) {
        let _ = win.hide();
    }
}
