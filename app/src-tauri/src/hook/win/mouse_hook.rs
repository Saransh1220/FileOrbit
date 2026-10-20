//! The WH_MOUSE_LL hook and its message loop. Does no work itself: every
//! event is forwarded over a channel so the callback returns immediately.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::OnceLock;
use tauri::AppHandle;
use windows::Win32::Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_ESCAPE, VK_SHIFT};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, SetWindowsHookExW, KBDLLHOOKSTRUCT, MSG, MSLLHOOKSTRUCT, WH_KEYBOARD_LL,
    WH_MOUSE_LL, WM_KEYDOWN, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE,
};

/// Screen coordinates are physical pixels (the app is per-monitor DPI aware).
pub enum MouseEvent {
    Down { x: i32, y: i32 },
    Move { x: i32, y: i32, shift: bool },
    Up,
    /// Esc was pressed while a button was held.
    Cancel,
}

static EVENTS: OnceLock<Sender<MouseEvent>> = OnceLock::new();
static LEFT_DOWN: AtomicBool = AtomicBool::new(false);

pub fn run(app: AppHandle, tx: Sender<MouseEvent>) {
    let _ = EVENTS.set(tx);
    // SAFETY: plain Win32 calls; the hook procedure is a valid `extern "system"`
    // function and this thread pumps messages below, as low-level hooks require.
    unsafe {
        let module = GetModuleHandleW(None).map(|m| HINSTANCE(m.0)).ok();
        if SetWindowsHookExW(WH_MOUSE_LL, Some(hook_proc), module, 0).is_err() {
            crate::status::set(&app, "error");
            return;
        }
        // Esc during a drag: Windows cancels the drag itself, but only a keyboard hook tells us
        // to close the ring right away instead of when the button is finally released.
        if SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), module, 0).is_err() {
            eprintln!("[hook] keyboard hook unavailable: Esc will close the ring on release");
        }
        crate::status::set(&app, "listening");
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {}
    }
}

unsafe extern "system" fn keyboard_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 && wparam.0 as u32 == WM_KEYDOWN && LEFT_DOWN.load(Ordering::Relaxed) {
        // SAFETY: for WH_KEYBOARD_LL, lparam points to a KBDLLHOOKSTRUCT.
        let info = unsafe { &*(lparam.0 as *const KBDLLHOOKSTRUCT) };
        if info.vkCode == VK_ESCAPE.0 as u32 {
            if let Some(tx) = EVENTS.get() {
                let _ = tx.send(MouseEvent::Cancel);
            }
        }
    }
    // SAFETY: forwarding the unmodified arguments we were given.
    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

fn shift_held() -> bool {
    // SAFETY: no preconditions; the high bit means "currently down".
    unsafe { GetAsyncKeyState(VK_SHIFT.0 as i32) as u16 & 0x8000 != 0 }
}

unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        // SAFETY: for WH_MOUSE_LL, lparam points to a MSLLHOOKSTRUCT.
        let info = unsafe { &*(lparam.0 as *const MSLLHOOKSTRUCT) };
        let (x, y) = (info.pt.x, info.pt.y);
        let event = match wparam.0 as u32 {
            WM_LBUTTONDOWN => {
                LEFT_DOWN.store(true, Ordering::Relaxed);
                Some(MouseEvent::Down { x, y })
            }
            WM_MOUSEMOVE if LEFT_DOWN.load(Ordering::Relaxed) => {
                Some(MouseEvent::Move { x, y, shift: shift_held() })
            }
            WM_LBUTTONUP => {
                LEFT_DOWN.store(false, Ordering::Relaxed);
                Some(MouseEvent::Up)
            }
            _ => None,
        };
        if let (Some(event), Some(tx)) = (event, EVENTS.get()) {
            let _ = tx.send(event);
        }
    }
    // SAFETY: forwarding the unmodified arguments we were given.
    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}
