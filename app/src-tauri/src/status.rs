//! Hook status shared with the UI. Stored as well as emitted, because the
//! listener can report before any window has registered for events.

use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};

#[derive(Default)]
pub struct HookStatus(Mutex<String>);

pub fn set(app: &AppHandle, status: &str) {
    eprintln!("[hook] status = {status}");
    *app.state::<HookStatus>().0.lock().unwrap() = status.to_string();
    let _ = app.emit("hook-status", status);
}

#[tauri::command]
pub fn get_hook_status(app: AppHandle) -> String {
    let current = app.state::<HookStatus>().0.lock().unwrap().clone();
    if current.is_empty() {
        "starting".into()
    } else {
        current
    }
}
