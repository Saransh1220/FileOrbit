//! macOS listener built on a listen-only CGEventTap.
//!
//! Needs the "Input Monitoring" permission. We ask for it up front so macOS
//! shows its prompt instead of silently delivering nothing.

use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
use core_graphics::event::{
    CallbackResult, CGEventFlags, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
    CGEventType,
};
use objc2_app_kit::{NSPasteboard, NSPasteboardNameDrag};
use objc2_foundation::{NSString, NSURL};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::AppHandle;

use super::{DragTrigger, TriggerAction};
use crate::overlay;

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGPreflightListenEventAccess() -> bool;
    fn CGRequestListenEventAccess() -> bool;
}

const FILE_URL_TYPE: &str = "public.file-url";

pub fn start(app: AppHandle) {
    std::thread::spawn(move || run(app));
}

fn run(app: AppHandle) {
    // SAFETY: plain C calls with no arguments.
    let granted = unsafe { CGPreflightListenEventAccess() || CGRequestListenEventAccess() };
    eprintln!("[hook] listen access granted: {granted}");
    if !granted {
        // The tap can be created without permission yet never receive events,
        // so stop here; macOS needs an app restart after the user grants it.
        crate::status::set(&app, "permission-denied");
        return;
    }

    let state = Mutex::new((DragTrigger::default(), Vec::<PathBuf>::new()));
    let cb_app = app.clone();

    let tap = CGEventTap::new(
        CGEventTapLocation::HID,
        CGEventTapPlacement::HeadInsertEventTap,
        CGEventTapOptions::ListenOnly,
        vec![
            CGEventType::LeftMouseDown,
            CGEventType::LeftMouseDragged,
            CGEventType::LeftMouseUp,
        ],
        move |_proxy, event_type, event| {
            let loc = event.location();
            let shift = event.get_flags().contains(CGEventFlags::CGEventFlagShift);
            let mut guard = state.lock().unwrap();
            let (t, files) = &mut *guard;
            let action = match event_type {
                CGEventType::LeftMouseDown => {
                    t.button_down(loc.x, loc.y);
                    TriggerAction::None
                }
                CGEventType::LeftMouseDragged => {
                    t.button_dragged(loc.x, loc.y, shift, || {
                        *files = drag_pasteboard_files();
                        eprintln!("[hook] shift-drag, files on pasteboard: {}", files.len());
                        !files.is_empty()
                    })
                }
                CGEventType::LeftMouseUp => {
                    let a = t.button_up();
                    if a != TriggerAction::None {
                        eprintln!("[hook] mouse up -> {a:?}");
                    }
                    a
                }
                _ => TriggerAction::None,
            };
            let dragged = files.clone();
            drop(guard);
            match action {
                TriggerAction::Show { x, y } => overlay::show_at(&cb_app, x, y, &dragged),
                TriggerAction::Ended => overlay::notify_drag_ended(&cb_app),
                TriggerAction::None => {}
            }
            CallbackResult::Keep
        },
    );

    let Ok(tap) = tap else {
        crate::status::set(&app, "permission-denied");
        return;
    };
    let Ok(source) = tap.mach_port().create_runloop_source(0) else {
        crate::status::set(&app, "error");
        return;
    };
    // SAFETY: kCFRunLoopCommonModes is a valid static CFString constant.
    CFRunLoop::get_current().add_source(&source, unsafe { kCFRunLoopCommonModes });
    tap.enable();
    crate::status::set(&app, "listening");
    CFRunLoop::run_current();
}

/// File paths on the system drag pasteboard (a Finder drag), or empty for a
/// text selection or in-app drag.
fn drag_pasteboard_files() -> Vec<PathBuf> {
    // SAFETY: NSPasteboardNameDrag is a valid static NSString constant, and
    // NSPasteboard is documented as safe to read from background threads.
    let board = unsafe { NSPasteboard::pasteboardWithName(NSPasteboardNameDrag) };
    let Some(items) = board.pasteboardItems() else {
        return Vec::new();
    };
    let file_url = NSString::from_str(FILE_URL_TYPE);
    items
        .iter()
        .filter_map(|item| {
            let text = item.stringForType(&file_url)?;
            // Finder hands out file *reference* URLs (file:///.file/id=...);
            // filePathURL resolves those to the real path, and passes plain
            // file URLs through unchanged.
            let resolved = NSURL::URLWithString(&text)?.filePathURL()?;
            Some(PathBuf::from(resolved.path()?.to_string()))
        })
        .collect()
}
