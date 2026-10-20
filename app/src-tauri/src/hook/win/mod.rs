//! Windows listener: a low-level mouse hook feeding the shared [`DragTrigger`].
//!
//! Windows cannot show what is inside an in-flight drag, so "is this a file
//! drag?" is answered differently from macOS: the press must start on an
//! Explorer window or the desktop, and the files are the Explorer selection
//! at that moment (see `explorer`).

mod explorer;
mod mouse_hook;

use std::path::PathBuf;
use std::sync::mpsc;
use tauri::AppHandle;
use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};

use super::{DragTrigger, TriggerAction};
use crate::overlay;
use mouse_hook::MouseEvent;

pub fn start(app: AppHandle) {
    let (tx, rx) = mpsc::channel();
    let hook_app = app.clone();
    std::thread::spawn(move || mouse_hook::run(hook_app, tx));
    std::thread::spawn(move || worker(app, rx));
}

/// Runs the trigger rules off the hook thread: the hook callback must return
/// fast, while reading the Explorer selection (COM) can take a few ms.
fn worker(app: AppHandle, rx: mpsc::Receiver<MouseEvent>) {
    // SAFETY: first COM call on this thread; the worker owns it for its lifetime.
    let _ = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
    let mut trigger = DragTrigger::default();
    let mut press: (i32, i32) = (0, 0);
    let mut probed = false;
    let mut files: Vec<PathBuf> = Vec::new();

    for event in rx {
        let action = match event {
            MouseEvent::Down { x, y } => {
                press = (x, y);
                probed = false;
                files.clear();
                trigger.button_down(x as f64, y as f64);
                TriggerAction::None
            }
            MouseEvent::Move { x, y, shift } => trigger.button_dragged(x as f64, y as f64, shift, || {
                // Probe once per press: a negative answer is final for this drag.
                if probed {
                    return !files.is_empty();
                }
                probed = true;
                files = explorer::selected_files_at(press.0, press.1);
                eprintln!("[hook] shift-drag, files selected in Explorer: {}", files.len());
                !files.is_empty()
            }),
            MouseEvent::Cancel => {
                if trigger.cancel() {
                    eprintln!("[hook] Esc -> close the ring");
                    overlay::hide(&app);
                }
                TriggerAction::None
            }
            MouseEvent::Up => {
                let a = trigger.button_up();
                if a != TriggerAction::None {
                    eprintln!("[hook] mouse up -> {a:?}");
                }
                a
            }
        };
        match action {
            TriggerAction::Show { x, y } => overlay::show_at_physical(&app, x, y, &files),
            TriggerAction::Ended => overlay::notify_drag_ended(&app),
            TriggerAction::None => {}
        }
    }
}
