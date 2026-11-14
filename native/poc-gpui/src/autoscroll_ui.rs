//! BatoScroll middle-click autoscroll (HYBRID-SPEC §6.1) on top of
//! `scroll_physics::autoscroll::Autoscroll`, which owns every decision
//! (state machine, speed curve, smoothing, device-pixel stepping). This file
//! only routes GPUI events to it and decides what to swallow:
//!
//! | GPUI event (window-level listener, capture phase) | Call | Swallowed |
//! |---|---|---|
//! | middle down on an eligible spot | `press_middle` | yes |
//! | middle down elsewhere, idle | — | no (passes through) |
//! | any down while engaged | `press_middle`/`press_other` (stops) | yes, and its release |
//! | middle up while engaged | `release_middle` (toggle vs hold) | yes |
//! | mouse move while engaged | `pointer_moved` | yes (+ an occluding overlay: no hover) |
//! | wheel/touchpad while engaged | `cancel` | yes |
//! | Esc, window deactivated, touch-down | `cancel` | Esc yes; a touch that stopped it is ignored until lift |
//!
//! Eligible = inside the main scroll viewport, not on a text field, no dialog
//! open, and the content can scroll (max > 0). The sidebar is not scrollable,
//! so a middle press there does nothing.

use std::time::Instant;

use gpui::{Bounds, MouseButton, Pixels, Point};
use scroll_physics::autoscroll::{Autoscroll, AutoscrollConfig, AutoscrollState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Pass,
    Consume,
}

pub fn eligible(pos: Point<Pixels>, main: Bounds<Pixels>, max_offset: f32, dialog_open: bool, inputs: &[Bounds<Pixels>]) -> bool {
    !dialog_open && max_offset > 0.0 && main.contains(&pos) && !inputs.iter().any(|b| b.contains(&pos))
}

pub struct AutoRouter {
    pub auto: Autoscroll,
    /// A press that stopped autoscroll: its release is swallowed too.
    swallow_up: Option<MouseButton>,
    /// When the compass appeared (pop-in animation).
    pub since: Instant,
    debug: bool,
}

impl Default for AutoRouter {
    fn default() -> Self {
        Self { auto: Autoscroll::new(AutoscrollConfig::load()), swallow_up: None, since: Instant::now(), debug: crate::scroll::debug_enabled() }
    }
}

impl AutoRouter {
    #[cfg(test)]
    pub fn with_config(config: AutoscrollConfig) -> Self {
        Self { auto: Autoscroll::new(config), swallow_up: None, since: Instant::now(), debug: false }
    }

    pub fn engaged(&self) -> bool {
        self.auto.is_engaged()
    }

    fn log(&self, what: &str, before: AutoscrollState) {
        if self.debug && before != self.auto.state() {
            eprintln!("[autoscroll] {what}: {before:?} -> {:?} origin={:?}", self.auto.state(), self.auto.origin());
        }
    }

    /// Returns (verdict, engaged-just-now).
    pub fn press(&mut self, button: MouseButton, pos: Point<Pixels>, eligible: bool, now: Instant) -> (Verdict, bool) {
        let before = self.auto.state();
        let (x, y) = (f32::from(pos.x), f32::from(pos.y));
        let out = if self.auto.is_engaged() {
            // Any press stops it; the press and its release never reach the content.
            if button == MouseButton::Middle {
                self.auto.press_middle(x, y, now);
            } else {
                self.auto.press_other();
            }
            self.swallow_up = Some(button);
            (Verdict::Consume, false)
        } else if button == MouseButton::Middle && eligible {
            // Reload settings per gesture, like the extension does per page load.
            self.auto.set_config(AutoscrollConfig::load());
            self.auto.press_middle(x, y, now);
            self.since = now;
            (Verdict::Consume, true)
        } else {
            (Verdict::Pass, false)
        };
        self.log("press", before);
        out
    }

    pub fn release(&mut self, button: MouseButton) -> Verdict {
        if self.swallow_up == Some(button) {
            self.swallow_up = None;
            return Verdict::Consume;
        }
        let before = self.auto.state();
        let v = if button == MouseButton::Middle && self.auto.release_middle() { Verdict::Consume } else { Verdict::Pass };
        self.log("release", before);
        v
    }

    pub fn moved(&mut self, pos: Point<Pixels>) -> Verdict {
        if !self.auto.is_engaged() {
            return Verdict::Pass;
        }
        self.auto.pointer_moved(f32::from(pos.x), f32::from(pos.y));
        Verdict::Consume
    }

    /// Esc, wheel/touchpad, focus loss, touch-down.
    pub fn cancel(&mut self, why: &str) -> Verdict {
        let before = self.auto.state();
        let v = if self.auto.cancel() { Verdict::Consume } else { Verdict::Pass };
        self.log(why, before);
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{point, px, size};

    fn r() -> AutoRouter {
        AutoRouter::with_config(AutoscrollConfig::default())
    }
    fn p(x: f32, y: f32) -> Point<Pixels> {
        point(px(x), px(y))
    }

    #[test]
    fn eligibility() {
        let main = Bounds::new(p(224., 0.), size(px(1051.), px(800.)));
        let input = Bounds::new(p(357., 434.), size(px(790.), px(38.)));
        assert!(eligible(p(600., 300.), main, 1000., false, &[input]));
        assert!(!eligible(p(100., 300.), main, 1000., false, &[input]), "sidebar");
        assert!(!eligible(p(600., 450.), main, 1000., false, &[input]), "text field");
        assert!(!eligible(p(600., 300.), main, 0., false, &[input]), "nothing to scroll");
        assert!(!eligible(p(600., 300.), main, 1000., true, &[input]), "dialog open");
    }

    #[test]
    fn middle_click_toggles_and_any_press_stops_with_its_release() {
        let (mut a, t) = (r(), Instant::now());
        assert_eq!(a.press(MouseButton::Middle, p(600., 400.), true, t), (Verdict::Consume, true));
        assert_eq!(a.release(MouseButton::Middle), Verdict::Consume);
        assert_eq!(a.auto.state(), AutoscrollState::Active, "clean click → toggle mode");
        assert_eq!(a.moved(p(600., 600.)), Verdict::Consume);
        assert!(a.auto.direction().down);
        assert_eq!(a.press(MouseButton::Left, p(300., 300.), false, t), (Verdict::Consume, false));
        assert!(!a.engaged());
        assert_eq!(a.release(MouseButton::Left), Verdict::Consume, "the stopping click never lands");
        assert_eq!(a.release(MouseButton::Left), Verdict::Pass);
        assert_eq!(a.press(MouseButton::Left, p(300., 300.), false, t), (Verdict::Pass, false));
    }

    #[test]
    fn hold_to_scroll_ends_on_release() {
        let (mut a, t) = (r(), Instant::now());
        a.press(MouseButton::Middle, p(600., 400.), true, t);
        a.moved(p(600., 450.));
        assert_eq!(a.release(MouseButton::Middle), Verdict::Consume);
        assert!(!a.engaged());
    }

    #[test]
    fn ineligible_middle_passes_through() {
        let (mut a, t) = (r(), Instant::now());
        assert_eq!(a.press(MouseButton::Middle, p(100., 400.), false, t), (Verdict::Pass, false));
        assert!(!a.engaged());
        assert_eq!(a.release(MouseButton::Middle), Verdict::Pass);
        assert_eq!(a.moved(p(1., 1.)), Verdict::Pass);
    }

    #[test]
    fn wheel_and_escape_cancel_only_when_engaged() {
        let (mut a, t) = (r(), Instant::now());
        assert_eq!(a.cancel("wheel"), Verdict::Pass);
        a.press(MouseButton::Middle, p(600., 400.), true, t);
        a.release(MouseButton::Middle);
        assert_eq!(a.cancel("wheel"), Verdict::Consume);
        assert!(!a.engaged());
    }

    #[test]
    fn ticks_move_down_when_cursor_below() {
        let (mut a, t) = (r(), Instant::now());
        a.press(MouseButton::Middle, p(640., 400.), true, t);
        a.release(MouseButton::Middle);
        a.moved(p(640., 700.));
        let mut total = 0.0;
        for i in 1..=30 {
            total += a.auto.tick(t + std::time::Duration::from_millis(i * 16), 1.0).1;
        }
        assert!(total > 1000.0, "scrolls down fast: {total}");
    }
}
