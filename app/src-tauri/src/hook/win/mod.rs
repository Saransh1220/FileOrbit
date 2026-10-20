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

use super::selection_memory::choose;
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
    // Explorer shrinks the selection when Shift is held during a click, so the selection from
    // just before the press is kept (see `selection_memory`).
    let mut shift_held = false;
    let mut button_held = false;
    let mut remembered: Option<explorer::Selection> = None;

    for event in rx {
        let action = match event {
            MouseEvent::ShiftDown => {
                shift_held = true;
                if !button_held {
                    remembered = explorer::selection_at_cursor();
                    eprintln!("[hook] Shift down: remembered {} selected file(s)", remembered.as_ref().map_or(0, |s| s.files.len()));
                }
                TriggerAction::None
            }
            MouseEvent::ShiftUp => {
                shift_held = false;
                if !button_held {
                    remembered = None;
                }
                TriggerAction::None
            }
            MouseEvent::Down { x, y } => {
                button_held = true;
                press = (x, y);
                probed = false;
                files.clear();
                trigger.button_down(x as f64, y as f64);
                TriggerAction::None
            }
            MouseEvent::Move { x, y, shift } => trigger.button_dragged(x as f64, y as f64, shift, || {
                // Probe once per press: a negative answer is final for this drag.
                if probed {
                    return overlay::wants_ring(&files);
                }
                probed = true;
                let current = explorer::selection_at(press.0, press.1);
                files = match (remembered.as_ref(), current) {
                    // The same window: Explorer may have shrunk the selection when Shift was clicked.
                    (Some(before), Some(now)) if before.window == now.window => choose(&before.files, now.files),
                    (_, Some(now)) => now.files,
                    (_, None) => Vec::new(),
                };
                eprintln!("[hook] shift-drag, files selected in Explorer: {}", files.len());
                // No ring at all when nothing can be done with these files.
                overlay::wants_ring(&files)
            }),
            MouseEvent::Cancel => {
                if trigger.cancel() {
                    eprintln!("[hook] Esc -> close the ring");
                    overlay::hide(&app);
                }
                TriggerAction::None
            }
            MouseEvent::Up => {
                button_held = false;
                let a = trigger.button_up();
                if a != TriggerAction::None {
                    eprintln!("[hook] mouse up -> {a:?}");
                }
                // With Shift still down, the selection after this click is the one the next
                // press will start from.
                remembered = if shift_held { explorer::selection_at_cursor() } else { None };
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
