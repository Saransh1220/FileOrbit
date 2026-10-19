//! Windows-only: our own OLE drop target for the ring overlay.
//!
//! The ring must accept an Explorer drag that started before the window
//! existed, always as a copy. Owning the drop target (instead of relying on
//! the webview's) keeps that behavior in one place. It reports the pointer to
//! the page as `ring-drag` events; the page already knows the dragged paths
//! from `ring-opened`, so the data object is never read.

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use windows::core::{implement, Ref, Result};
use windows::Win32::Foundation::{HWND, POINT, POINTL};
use windows::Win32::Graphics::Gdi::ScreenToClient;
use windows::Win32::System::Com::IDataObject;
use windows::Win32::System::Ole::{
    IDropTarget, IDropTarget_Impl, RegisterDragDrop, DROPEFFECT, DROPEFFECT_COPY,
};
use windows::Win32::System::SystemServices::MODIFIERKEYS_FLAGS;

use crate::overlay::OVERLAY_LABEL;

/// Pointer position in physical pixels, relative to the overlay's client area.
#[derive(Serialize, Clone)]
struct RingDrag {
    kind: &'static str,
    x: f64,
    y: f64,
}

#[implement(IDropTarget)]
struct RingDropTarget {
    app: AppHandle,
    top: HWND,
}

impl RingDropTarget {
    fn report(&self, kind: &'static str, pt: Option<&POINTL>) {
        let (x, y) = match pt {
            Some(p) => {
                let mut point = POINT { x: p.x, y: p.y };
                // SAFETY: `top` is our live overlay window.
                let _ = unsafe { ScreenToClient(self.top, &mut point) };
                (point.x as f64, point.y as f64)
            }
            None => (0.0, 0.0),
        };
        if kind != "over" {
            eprintln!("[drop] {kind} at {x},{y}");
        }
        let _ = self.app.emit_to(OVERLAY_LABEL, "ring-drag", RingDrag { kind, x, y });
    }
}

/// Always a copy: the source file must never be moved or deleted.
fn accept_as_copy(effect: *mut DROPEFFECT) {
    if !effect.is_null() {
        // SAFETY: OLE passes a valid pointer for the effect.
        unsafe { *effect = DROPEFFECT_COPY };
    }
}

#[allow(non_snake_case)]
impl IDropTarget_Impl for RingDropTarget_Impl {
    fn DragEnter(
        &self,
        _data: Ref<'_, IDataObject>,
        _keys: MODIFIERKEYS_FLAGS,
        pt: &POINTL,
        effect: *mut DROPEFFECT,
    ) -> Result<()> {
        accept_as_copy(effect);
        self.report("enter", Some(pt));
        Ok(())
    }

    fn DragOver(&self, _keys: MODIFIERKEYS_FLAGS, pt: &POINTL, effect: *mut DROPEFFECT) -> Result<()> {
        accept_as_copy(effect);
        self.report("over", Some(pt));
        Ok(())
    }

    fn DragLeave(&self) -> Result<()> {
        self.report("leave", None);
        Ok(())
    }

    fn Drop(
        &self,
        _data: Ref<'_, IDataObject>,
        _keys: MODIFIERKEYS_FLAGS,
        pt: &POINTL,
        effect: *mut DROPEFFECT,
    ) -> Result<()> {
        accept_as_copy(effect);
        self.report("drop", Some(pt));
        Ok(())
    }
}

/// Registers our drop target on `hwnd`. `top` is the window whose client
/// area the reported positions are relative to. Must run on the UI thread.
pub fn register(app: &AppHandle, hwnd: HWND, top: HWND) {
    let target: IDropTarget = RingDropTarget { app: app.clone(), top }.into();
    // SAFETY: a live window on its own thread.
    let result = unsafe { RegisterDragDrop(hwnd, &target) };
    eprintln!("[drop] register on {:?}: {result:?}", hwnd.0);
}
