//! Windows listener: placeholder so the crate compiles on Windows.
//! Real implementation (WH_MOUSE_LL hook) lands in the Windows spike.

use tauri::AppHandle;

pub fn start(app: AppHandle) {
    crate::status::set(&app, "unsupported");
}
