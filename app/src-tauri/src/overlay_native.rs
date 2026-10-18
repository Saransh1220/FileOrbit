//! Windows-only: show the overlay without stealing focus.
//!
//! Activating a window mid-drag would cancel the drag, so the overlay is a
//! non-activating tool window that is shown with SW_SHOWNOACTIVATE.

use tauri::WebviewWindow;
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowLongPtrW, SetWindowLongPtrW, SetWindowPos, GWL_EXSTYLE, GWL_STYLE,
    HWND_TOPMOST, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,     WS_CAPTION, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_SYSMENU, WS_THICKFRAME,
};

pub fn show_without_activating(win: &WebviewWindow) {
    let Ok(hwnd) = win.hwnd() else {
        let _ = win.show();
        return;
    };
    let _ = win.set_decorations(false);
    // SAFETY: hwnd belongs to our own live window.
    unsafe {
        let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        let wanted = style | (WS_EX_NOACTIVATE.0 | WS_EX_TOOLWINDOW.0) as isize;
        if wanted != style {
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, wanted);
        }
    }
    // Tauri's own show also makes the embedded webview visible, which it needs
    // to receive drops. WS_EX_NOACTIVATE (set above) keeps it from taking focus.
    let _ = win.show();
    // SAFETY: as above.
    unsafe {
        let frame = (WS_CAPTION.0 | WS_THICKFRAME.0 | WS_SYSMENU.0) as isize;
        let base = GetWindowLongPtrW(hwnd, GWL_STYLE);
        if base & frame != 0 {
            SetWindowLongPtrW(hwnd, GWL_STYLE, base & !frame);
        }
        let _ = SetWindowPos(
            hwnd,
            Some(HWND_TOPMOST),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_FRAMECHANGED,
        );
    }
}
