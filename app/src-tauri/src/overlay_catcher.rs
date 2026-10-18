//! Windows-only: an invisible native window laid over the ring that receives
//! the drag.
//!
//! The webview's inner windows belong to the WebView2 browser process, so
//! Windows does not reliably hand them an Explorer drag that began before the
//! overlay existed. A plain window of our own, registered with our drop
//! target, behaves like any normal app window.

use std::sync::OnceLock;
use tauri::{AppHandle, WebviewWindow};
use windows::core::w;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, GetWindowRect, RegisterClassW, SetLayeredWindowAttributes,
    SetWindowPos, ShowWindow, HWND_TOPMOST, LWA_ALPHA, SWP_NOACTIVATE, SWP_SHOWWINDOW, SW_HIDE,
    WNDCLASSW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
};

/// The catcher's handle, stored as an integer so the static is `Sync`.
static CATCHER: OnceLock<isize> = OnceLock::new();

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    // SAFETY: forwarding the unmodified arguments we were given.
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

fn create(app: &AppHandle) -> Option<HWND> {
    // SAFETY: standard window-class registration and creation on the UI thread.
    unsafe {
        let instance = GetModuleHandleW(None).ok()?;
        let class = WNDCLASSW {
            lpfnWndProc: Some(wnd_proc),
            hInstance: instance.into(),
            lpszClassName: w!("RadialRingDropCatcher"),
            ..Default::default()
        };
        RegisterClassW(&class);
        let hwnd = CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TOPMOST,
            w!("RadialRingDropCatcher"),
            w!(""),
            WS_POPUP,
            0,
            0,
            1,
            1,
            None,
            None,
            Some(instance.into()),
            None,
        )
        .ok()?;
        // Nearly invisible but still hit-testable: a fully transparent
        // layered window would let the drag fall through to Explorer.
        let _ = SetLayeredWindowAttributes(hwnd, Default::default(), 1, LWA_ALPHA);
        crate::overlay_drop::register(app, hwnd, hwnd);
        Some(hwnd)
    }
}

/// Covers the overlay window with the catcher. Must run on the UI thread.
pub fn cover(app: &AppHandle, overlay: &WebviewWindow) {
    let Ok(overlay_hwnd) = overlay.hwnd() else {
        return;
    };
    let hwnd = match CATCHER.get() {
        Some(&h) => HWND(h as *mut _),
        None => {
            let Some(h) = create(app) else {
                eprintln!("[drop] could not create the catcher window");
                return;
            };
            let _ = CATCHER.set(h.0 as isize);
            h
        }
    };
    let mut r = RECT::default();
    // SAFETY: both windows are live and owned by this thread.
    unsafe {
        let _ = GetWindowRect(overlay_hwnd, &mut r);
        let _ = SetWindowPos(
            hwnd,
            Some(HWND_TOPMOST),
            r.left,
            r.top,
            r.right - r.left,
            r.bottom - r.top,
            SWP_NOACTIVATE | SWP_SHOWWINDOW,
        );
    }
    eprintln!("[drop] catcher over ({},{} {}x{})", r.left, r.top, r.right - r.left, r.bottom - r.top);
}

/// Hides the catcher. Must run on the UI thread.
pub fn uncover() {
    if let Some(&h) = CATCHER.get() {
        // SAFETY: the catcher lives for the whole run.
        unsafe {
            let _ = ShowWindow(HWND(h as *mut _), SW_HIDE);
        }
    }
}
