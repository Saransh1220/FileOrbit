//! Finds the files selected in the Explorer window (or desktop) under a point.

use std::path::PathBuf;
use windows::core::{Interface, PWSTR};
use windows::Win32::Foundation::{HWND, POINT};
use windows::Win32::System::Com::{
    CoCreateInstance, CoTaskMemFree, IDispatch, IServiceProvider, CLSCTX_ALL,
};
use windows::Win32::System::Variant::VARIANT;
use windows::Win32::UI::Shell::{
    IFolderView2, IShellBrowser, IShellItemArray, IShellView, IShellWindows,
    IWebBrowserApp, ShellWindows, SID_STopLevelBrowser, SIGDN_FILESYSPATH, SVGIO_SELECTION, SWC_DESKTOP,
    SWFO_NEEDDISPATCH,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetAncestor, GetClassNameW, GetCursorPos, IsWindowVisible, WindowFromPoint, GA_ROOT,
};

/// `CSIDL_DESKTOP`, the location code `FindWindowSW` uses for the desktop.
const CSIDL_DESKTOP: i32 = 0;

/// Top-level window classes whose file drags we handle: Explorer folders,
/// and the two windows that can host the desktop.
const FILE_SURFACE_CLASSES: [&str; 3] = ["CabinetWClass", "Progman", "WorkerW"];

/// The files selected in one Explorer window (or the desktop), and which window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    /// The window's handle, so a selection remembered earlier is only reused for the same window.
    pub window: isize,
    pub files: Vec<PathBuf>,
}

/// The selection of the Explorer window or desktop under `(x, y)`, or `None` if that point
/// is not on a file surface (or the selection cannot be read).
pub fn selection_at(x: i32, y: i32) -> Option<Selection> {
    // SAFETY: Win32/COM calls on valid values; the calling thread is a COM apartment.
    unsafe { selection(x, y) }
}

/// The same, for the window under the mouse pointer right now.
pub fn selection_at_cursor() -> Option<Selection> {
    let mut point = POINT::default();
    // SAFETY: `point` is a valid out parameter.
    unsafe { GetCursorPos(&mut point) }.ok()?;
    selection_at(point.x, point.y)
}

unsafe fn selection(x: i32, y: i32) -> Option<Selection> {
    let root = unsafe { GetAncestor(WindowFromPoint(POINT { x, y }), GA_ROOT) };
    if root.0.is_null() {
        return None;
    }
    let class = class_name(root);
    if !FILE_SURFACE_CLASSES.contains(&class.as_str()) {
        return None;
    }
    let windows: IShellWindows = unsafe { CoCreateInstance(&ShellWindows, None, CLSCTX_ALL) }.ok()?;
    let view = if class == "CabinetWClass" {
        unsafe { explorer_view(&windows, root) }
    } else {
        unsafe { desktop_view(&windows) }
    }?;
    let files = unsafe { selected_paths(&view) }?;
    Some(Selection { window: root.0 as isize, files })
}

fn class_name(hwnd: HWND) -> String {
    let mut buf = [0u16; 256];
    // SAFETY: `buf` is a valid writable buffer.
    let n = unsafe { GetClassNameW(hwnd, &mut buf) };
    String::from_utf16_lossy(&buf[..n.max(0) as usize])
}

/// The visible tab of an Explorer window. Explorer tabs share one top-level
/// window, so several shell windows match; only the active tab's view is shown.
unsafe fn explorer_view(windows: &IShellWindows, root: HWND) -> Option<IShellView> {
    let count = unsafe { windows.Count() }.ok()?;
    for i in 0..count {
        let Ok(dispatch) = (unsafe { windows.Item(&VARIANT::from(i)) }) else { continue };
        let Ok(app) = dispatch.cast::<IWebBrowserApp>() else { continue };
        let Ok(handle) = (unsafe { app.HWND() }) else { continue };
        if handle.0 as isize != root.0 as isize {
            continue;
        }
        let Some(view) = (unsafe { active_view(&dispatch) }) else { continue };
        let visible = unsafe { view.GetWindow() }.map(|w| unsafe { IsWindowVisible(w) }.as_bool());
        if visible.unwrap_or(false) {
            return Some(view);
        }
    }
    None
}

unsafe fn desktop_view(windows: &IShellWindows) -> Option<IShellView> {
    let mut hwnd = 0i32;
    let dispatch = unsafe {
        windows.FindWindowSW(
            &VARIANT::from(CSIDL_DESKTOP),
            &VARIANT::default(),
            SWC_DESKTOP,
            &mut hwnd,
            SWFO_NEEDDISPATCH,
        )
    }
    .ok()?;
    unsafe { active_view(&dispatch) }
}

unsafe fn active_view(dispatch: &IDispatch) -> Option<IShellView> {
    let provider = dispatch.cast::<IServiceProvider>().ok()?;
    let browser: IShellBrowser = unsafe { provider.QueryService(&SID_STopLevelBrowser) }.ok()?;
    unsafe { browser.QueryActiveShellView() }.ok()
}

unsafe fn selected_paths(view: &IShellView) -> Option<Vec<PathBuf>> {
    let folder = view.cast::<IFolderView2>().ok()?;
    let items: IShellItemArray = unsafe { folder.Items(SVGIO_SELECTION) }.ok()?;
    let count = unsafe { items.GetCount() }.ok()?;
    let mut paths = Vec::new();
    for i in 0..count {
        let Ok(item) = (unsafe { items.GetItemAt(i) }) else { continue };
        let Ok(name) = (unsafe { item.GetDisplayName(SIGDN_FILESYSPATH) }) else { continue };
        paths.push(PathBuf::from(pwstr_to_string(name)));
        // SAFETY: GetDisplayName allocates with the COM task allocator.
        unsafe { CoTaskMemFree(Some(name.0 as *const _)) };
    }
    Some(paths)
}

fn pwstr_to_string(p: PWSTR) -> String {
    // SAFETY: p is a valid NUL-terminated string returned by the shell.
    unsafe { p.to_string() }.unwrap_or_default()
}
