//! Global "Shift + file drag" detection.
//!
//! Each OS has its own listener; both feed the same [`DragTrigger`] state
//! machine so the trigger rules live in exactly one place.

mod trigger;

#[cfg(target_os = "macos")]
mod mac;
#[cfg(target_os = "windows")]
mod win;

pub use trigger::{DragTrigger, TriggerAction};

use tauri::AppHandle;

/// Starts the platform listener on its own thread.
pub fn start(app: AppHandle) {
    #[cfg(target_os = "macos")]
    mac::start(app);
    #[cfg(target_os = "windows")]
    win::start(app);
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let _ = app;
}
