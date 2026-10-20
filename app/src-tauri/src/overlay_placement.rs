//! Where the ring window goes: centered on the cursor, but never off the screen.
//! Pure arithmetic in physical pixels, so it is tested on every platform.
#![cfg_attr(not(windows), allow(dead_code))]

/// A monitor's rectangle in physical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Area {
    pub left: i32,
    pub top: i32,
    pub width: i32,
    pub height: i32,
}

/// Top-left corner for a `size` x `size` window centered on `(cursor_x, cursor_y)`,
/// moved just far enough to sit fully inside `area`. A window larger than the
/// area is pinned to the area's top-left.
pub fn ring_origin(cursor_x: i32, cursor_y: i32, size: i32, area: Area) -> (i32, i32) {
    let wanted_left = cursor_x - size / 2;
    let wanted_top = cursor_y - size / 2;
    (
        wanted_left.min(area.left + area.width - size).max(area.left),
        wanted_top.min(area.top + area.height - size).max(area.top),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCREEN: Area = Area { left: 0, top: 0, width: 1920, height: 1080 };

    #[test]
    fn centered_when_there_is_room() {
        assert_eq!(ring_origin(960, 540, 330, SCREEN), (795, 375));
    }

    #[test]
    fn pushed_in_from_each_edge() {
        assert_eq!(ring_origin(10, 540, 330, SCREEN).0, 0, "left edge");
        assert_eq!(ring_origin(1915, 540, 330, SCREEN).0, 1590, "right edge");
        assert_eq!(ring_origin(960, 5, 330, SCREEN).1, 0, "top edge");
        assert_eq!(ring_origin(960, 1075, 330, SCREEN).1, 750, "bottom edge");
    }

    #[test]
    fn a_corner_is_pushed_in_on_both_axes() {
        assert_eq!(ring_origin(1919, 1079, 330, SCREEN), (1590, 750));
        assert_eq!(ring_origin(0, 0, 330, SCREEN), (0, 0));
    }

    #[test]
    fn a_second_monitor_to_the_left_uses_its_own_edges() {
        let left_monitor = Area { left: -1920, top: 0, width: 1920, height: 1080 };
        assert_eq!(ring_origin(-1915, 540, 330, left_monitor).0, -1920);
        assert_eq!(ring_origin(-5, 540, 330, left_monitor).0, -330);
    }

    #[test]
    fn a_monitor_above_with_a_negative_top() {
        let above = Area { left: 0, top: -1080, width: 1920, height: 1080 };
        assert_eq!(ring_origin(960, -1078, 330, above).1, -1080);
    }

    #[test]
    fn a_window_bigger_than_the_screen_is_pinned_to_its_top_left() {
        let tiny = Area { left: 100, top: 50, width: 200, height: 200 };
        assert_eq!(ring_origin(150, 100, 330, tiny), (100, 50));
    }
}
