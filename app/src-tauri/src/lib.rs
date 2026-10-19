mod design;
pub mod engine;
mod hook;
mod overlay;
#[cfg(target_os = "windows")]
mod overlay_catcher;
#[cfg(target_os = "windows")]
mod overlay_drop;
#[cfg(target_os = "windows")]
mod overlay_native;
mod reveal;
mod settings;
mod status;
mod toast;
mod tray;

use std::path::PathBuf;
use tauri::{AppHandle, Manager, WindowEvent};

#[tauri::command]
fn hide_overlay(app: AppHandle) {
    overlay::hide(&app);
}

/// Lets the overlay's webview write to the app log (diagnostics).
#[tauri::command]
fn log_event(message: String) {
    eprintln!("[ui] {message}");
}

/// A file was dropped on a wedge: convert each dropped file to that format.
#[tauri::command]
fn start_conversion(app: AppHandle, paths: Vec<String>, target: String) {
    for path in paths {
        engine::jobs::start(app.clone(), PathBuf::from(path), target.clone());
    }
}

/// The user clicked Install on a "needs pack" toast.
#[tauri::command]
fn install_pack_for_job(app: AppHandle, job_id: u64) {
    engine::jobs::install_pack_for_job(app, job_id);
}

#[tauri::command]
fn hide_toast(app: AppHandle) {
    // Dismissing the prompt abandons any conversion that was waiting on it.
    app.state::<engine::pending::PendingJobs>().clear();
    toast::hide(&app);
}

#[tauri::command]
fn reveal_in_file_manager(path: String) {
    reveal::reveal(&PathBuf::from(path));
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(status::HookStatus::default())
        .manage(engine::pending::PendingJobs::default())
        .invoke_handler(tauri::generate_handler![
            hide_overlay,
            hide_toast,
            install_pack_for_job,
            design::get_design,
            log_event,
            reveal_in_file_manager,
            start_conversion,
            status::get_hook_status
        ])
        .on_window_event(|window, event| {
            // The overlay and toast are created once and reused; closing one
            // (Alt+F4) must hide it, never destroy it.
            if window.label() != "main" {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
            // The status window is secondary: closing it hides it, the app
            // keeps running from the menu bar.
            if window.label() == "main" {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            app.manage(design::CurrentDesign::new(design::load(app.handle())));
            tray::build(app)?;
            engine::capabilities::warm_up();
            hook::start(app.handle().clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub(crate) fn show_main_window(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.set_focus();
    }
}
