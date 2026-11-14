//! Touchscreen: routes one finger to `scroll_physics::TouchDrag`.
//!
//! The patched gpui_linux delivers `wl_touch` as GPUI's raw `TouchEvent`s.
//! GPUI's window would run its own recognizer (8 px slop, iOS fling) on them;
//! instead the app *claims* every touch at touch-down (a window-level
//! `TouchDragEvent` listener calling `prevent_default`), which hands it the
//! first finger's down/move/up/cancel stream and keeps GPUI's recognizer out
//! (no GPUI pans, momentum, taps or long presses). GPUI only tracks the first
//! finger; extra fingers never reach us (TouchDrag would ignore them too).
//!
//! The slop, tap/scroll arbitration and fling all come from `TouchDrag`:
//! - a touch that starts in the main scroll area drives the main `ScrollAxis`;
//! - anywhere else (sidebar, dialogs) it drives a dummy axis that cannot move,
//!   so it can only ever be a tap;
//! - a `Tap` becomes a synthetic left click at the lift position (mouse-move,
//!   down, up — Chromium also emits the mouse events only at tap end).

use std::time::Instant;

use gpui::{Bounds, Pixels, Point, TouchPhase};
use scroll_physics::{ScrollAxis, TouchDrag, TouchOutcome};

/// GPUI hands us only the claimed (first) finger, without its id.
pub const FINGER: u64 = 0;

pub struct TouchRouter {
    drag: TouchDrag,
    /// Axis for touches outside the scroll area: max 0, never moves.
    dummy: ScrollAxis,
    on_main: bool,
}

impl Default for TouchRouter {
    fn default() -> Self {
        Self { drag: TouchDrag::new(), dummy: ScrollAxis::default(), on_main: false }
    }
}

impl TouchRouter {
    /// `main_area`: window bounds of the main scroll viewport, or `None` when
    /// it must not scroll (a dialog is open).
    pub fn handle(
        &mut self,
        phase: TouchPhase,
        pos: Point<Pixels>,
        now: Instant,
        main: &mut ScrollAxis,
        main_area: Option<Bounds<Pixels>>,
    ) -> TouchOutcome {
        let (x, y) = (f32::from(pos.x), f32::from(pos.y));
        if phase == TouchPhase::Started {
            self.on_main = main_area.is_some_and(|b| b.contains(&pos));
        }
        let axis = if self.on_main { main } else { &mut self.dummy };
        match phase {
            TouchPhase::Started => self.drag.down(FINGER, x, y, axis),
            TouchPhase::Moved => self.drag.moved(FINGER, x, y, now, axis),
            TouchPhase::Ended => self.drag.up(FINGER, now, axis),
            TouchPhase::Cancelled => {
                self.drag.cancel(axis);
                TouchOutcome::Ended
            }
        }
    }

    /// Whether the current touch drives the main scroll area.
    pub fn on_main(&self) -> bool {
        self.on_main
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{point, px, size};
    use std::time::Duration;

    fn main_axis() -> ScrollAxis {
        let mut a = ScrollAxis::default();
        a.set_max(10_000.0);
        a
    }
    fn area() -> Option<Bounds<Pixels>> {
        Some(Bounds::new(point(px(224.), px(0.)), size(px(1051.), px(800.))))
    }
    fn p(x: f32, y: f32) -> Point<Pixels> {
        point(px(x), px(y))
    }

    #[test]
    fn tap_in_main_clicks() {
        let (mut r, mut a, t) = (TouchRouter::default(), main_axis(), Instant::now());
        r.handle(TouchPhase::Started, p(600., 400.), t, &mut a, area());
        assert_eq!(r.handle(TouchPhase::Moved, p(605., 408.), t, &mut a, area()), TouchOutcome::Pending);
        assert_eq!(r.handle(TouchPhase::Ended, p(605., 408.), t, &mut a, area()), TouchOutcome::Tap);
        assert_eq!(a.offset(), 0.0);
    }

    #[test]
    fn drag_follows_finger_minus_slop_then_flings() {
        let (mut r, mut a, t) = (TouchRouter::default(), main_axis(), Instant::now());
        r.handle(TouchPhase::Started, p(600., 600.), t, &mut a, area());
        let mut out = vec![];
        for i in 1..=10 {
            let y = 600. - 30. * i as f32;
            out.push(r.handle(TouchPhase::Moved, p(600., y), t + Duration::from_millis(i * 10), &mut a, area()));
        }
        assert_eq!(out[0], TouchOutcome::ScrollStarted);
        assert!(out[1..].iter().all(|o| *o == TouchOutcome::Scrolling));
        // 300 px of finger travel, 15 px of it inside the slop.
        assert!((a.offset() - 285.0).abs() < 0.01, "{}", a.offset());
        assert_eq!(r.handle(TouchPhase::Ended, p(600., 300.), t + Duration::from_millis(105), &mut a, area()), TouchOutcome::Ended);
        assert!(a.is_animating(), "fast lift flings");
    }

    #[test]
    fn touch_in_sidebar_never_scrolls_main_and_taps() {
        let (mut r, mut a, t) = (TouchRouter::default(), main_axis(), Instant::now());
        r.handle(TouchPhase::Started, p(100., 600.), t, &mut a, area());
        r.handle(TouchPhase::Moved, p(100., 300.), t, &mut a, area());
        r.handle(TouchPhase::Ended, p(100., 300.), t, &mut a, area());
        assert_eq!(a.offset(), 0.0);
        r.handle(TouchPhase::Started, p(100., 165.), t, &mut a, area());
        assert_eq!(r.handle(TouchPhase::Ended, p(100., 165.), t, &mut a, area()), TouchOutcome::Tap);
    }

    #[test]
    fn dialog_open_blocks_scrolling_but_not_taps() {
        let (mut r, mut a, t) = (TouchRouter::default(), main_axis(), Instant::now());
        r.handle(TouchPhase::Started, p(600., 600.), t, &mut a, None);
        r.handle(TouchPhase::Moved, p(600., 200.), t, &mut a, None);
        assert_eq!(a.offset(), 0.0);
        r.handle(TouchPhase::Ended, p(600., 200.), t, &mut a, None);
        r.handle(TouchPhase::Started, p(600., 600.), t, &mut a, None);
        assert_eq!(r.handle(TouchPhase::Ended, p(600., 600.), t, &mut a, None), TouchOutcome::Tap);
    }

    #[test]
    fn touch_down_stops_a_fling_and_does_not_click() {
        let (mut r, mut a, t) = (TouchRouter::default(), main_axis(), Instant::now());
        a.wheel_notches(3.0, t);
        assert!(a.is_animating());
        r.handle(TouchPhase::Started, p(600., 400.), t, &mut a, area());
        assert!(!a.is_animating());
        assert_eq!(r.handle(TouchPhase::Ended, p(600., 400.), t, &mut a, area()), TouchOutcome::Ended);
    }

    #[test]
    fn cancel_never_flings_or_taps() {
        let (mut r, mut a, t) = (TouchRouter::default(), main_axis(), Instant::now());
        r.handle(TouchPhase::Started, p(600., 600.), t, &mut a, area());
        for i in 1..=5 {
            r.handle(TouchPhase::Moved, p(600., 600. - 40. * i as f32), t + Duration::from_millis(i * 10), &mut a, area());
        }
        let off = a.offset();
        assert_eq!(r.handle(TouchPhase::Cancelled, p(600., 400.), t + Duration::from_millis(55), &mut a, area()), TouchOutcome::Ended);
        assert!(!a.is_animating());
        assert_eq!(a.offset(), off);
        assert!(!r.drag.is_active());
    }
}
