/// Pixels the pointer must travel with the button held before we call it a drag.
pub const DRAG_THRESHOLD_PX: f64 = 6.0;

#[derive(Debug, PartialEq)]
pub enum TriggerAction {
    None,
    /// Open the ring centered on this point (logical screen coordinates).
    Show { x: f64, y: f64 },
    /// The button was released; the ring decides whether a drop is in flight.
    Ended,
}

/// Pure state machine: feed it mouse events, it says when to open the ring.
/// Kept free of OS types so it can be unit-tested on any platform.
#[derive(Default)]
pub struct DragTrigger {
    pressed_at: Option<(f64, f64)>,
    shown: bool,
}

impl DragTrigger {
    pub fn button_down(&mut self, x: f64, y: f64) {
        self.pressed_at = Some((x, y));
        self.shown = false;
    }

    /// `shift` is whether Shift is held now; `has_files` is evaluated lazily
    /// because reading the drag pasteboard is comparatively expensive.
    pub fn button_dragged(
        &mut self,
        x: f64,
        y: f64,
        shift: bool,
        has_files: impl FnOnce() -> bool,
    ) -> TriggerAction {
        let Some((px, py)) = self.pressed_at else {
            return TriggerAction::None;
        };
        if self.shown || !shift {
            return TriggerAction::None;
        }
        if (x - px).hypot(y - py) < DRAG_THRESHOLD_PX || !has_files() {
            return TriggerAction::None;
        }
        self.shown = true;
        TriggerAction::Show { x, y }
    }

    /// The user abandoned the drag (Esc). Returns whether the ring was open, so the caller
    /// knows to close it. The release that follows then reports nothing.
    pub fn cancel(&mut self) -> bool {
        let was_shown = self.shown;
        self.pressed_at = None;
        self.shown = false;
        was_shown
    }

    pub fn button_up(&mut self) -> TriggerAction {
        let was_shown = self.shown;
        self.pressed_at = None;
        self.shown = false;
        if was_shown {
            TriggerAction::Ended
        } else {
            TriggerAction::None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn needs_shift_distance_and_files() {
        let mut t = DragTrigger::default();
        t.button_down(0.0, 0.0);
        assert_eq!(t.button_dragged(2.0, 0.0, true, || true), TriggerAction::None);
        assert_eq!(t.button_dragged(20.0, 0.0, false, || true), TriggerAction::None);
        assert_eq!(t.button_dragged(20.0, 0.0, true, || false), TriggerAction::None);
        assert_eq!(
            t.button_dragged(20.0, 0.0, true, || true),
            TriggerAction::Show { x: 20.0, y: 0.0 }
        );
    }

    #[test]
    fn cancelling_closes_the_ring_and_silences_the_release() {
        let mut t = DragTrigger::default();
        t.button_down(0.0, 0.0);
        t.button_dragged(20.0, 0.0, true, || true);
        assert!(t.cancel(), "the ring was open");
        assert_eq!(t.button_up(), TriggerAction::None, "the release after Esc reports nothing");
        // Cancelling when nothing is open is harmless and says so.
        assert!(!t.cancel());
        // A later drag works normally.
        t.button_down(0.0, 0.0);
        assert_eq!(t.button_dragged(20.0, 0.0, true, || true), TriggerAction::Show { x: 20.0, y: 0.0 });
    }

    #[test]
    fn shows_once_and_reports_end() {
        let mut t = DragTrigger::default();
        t.button_down(0.0, 0.0);
        t.button_dragged(20.0, 0.0, true, || true);
        assert_eq!(t.button_dragged(30.0, 0.0, true, || true), TriggerAction::None);
        assert_eq!(t.button_up(), TriggerAction::Ended);
        assert_eq!(t.button_up(), TriggerAction::None);
    }
}
