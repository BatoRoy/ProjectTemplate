//! Scrolling: GPUI scroll events → `scroll_physics::ScrollAxis`, plus the
//! 5px Hybrid scrollbar.
//!
//! What GPUI (gpui_linux, Wayland) hands us, per `wl_pointer.frame`
//! (crates/gpui_linux/src/linux/wayland/client.rs):
//!
//! | Wayland                          | GPUI `ScrollEvent`                                          |
//! |----------------------------------|-------------------------------------------------------------|
//! | source=wheel, `axis_value120` v  | `ScrollDelta::Lines(-v/120 × 3)` (`Axis` events are dropped) |
//! | source=wheel, `axis_discrete` n  | `ScrollDelta::Lines(-n × 3)` (wl_pointer < v8)               |
//! | source=finger/continuous, `axis` | `ScrollDelta::Pixels(-value × 3)`                            |
//! | `axis_stop`                      | upstream: dropped. Patched: `Pixels(0,0)` + `TouchPhase::Ended` |
//!
//! So `Lines` are wheel notches ×3 (fractions survive for hi-res wheels) and
//! `Pixels` are touchpad/continuous deltas ×3. We undo both factors. Chromium
//! (Electron's engine) then scales a touchpad's raw `axis` value by
//! 120/10 = 12 px per unit (`wayland_pointer.cc`, `kWaylandUnscaledTouchpadScrolling`
//! is disabled by default), so we do the same to feel identical.

use std::time::Instant;

use gpui::{ScrollDelta, ScrollEvent, TouchPhase};
use scroll_physics::ScrollAxis;

/// GPUI multiplies both notches and finger deltas by this (`SCROLL_LINES` and
/// the hard-coded `modifier = 3.0` in the Wayland client).
pub const GPUI_FACTOR: f32 = 3.0;
/// Chromium's touchpad gain on Wayland: px per `wl_pointer.axis` unit.
pub const CHROMIUM_TOUCHPAD_GAIN: f32 = 12.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScrollInput {
    /// Wheel notches, positive = content moves up (offset grows).
    Notches(f32),
    /// Touchpad px, positive = offset grows.
    Precise(f32),
    /// Fingers lifted.
    End,
}

pub fn touchpad_gain() -> f32 {
    std::env::var("BATO_TOUCHPAD_GAIN").ok().and_then(|v| v.parse().ok()).unwrap_or(CHROMIUM_TOUCHPAD_GAIN)
}

/// Maps one GPUI scroll event to the vertical-axis inputs for `ScrollAxis`.
pub fn map_event(delta: ScrollDelta, phase: TouchPhase, gain: f32) -> Vec<ScrollInput> {
    let mut out = Vec::with_capacity(2);
    match delta {
        ScrollDelta::Lines(l) => {
            if l.y != 0.0 {
                out.push(ScrollInput::Notches(-l.y / GPUI_FACTOR));
            }
        }
        ScrollDelta::Pixels(p) => {
            let y = f32::from(p.y);
            if y != 0.0 {
                out.push(ScrollInput::Precise(-y / GPUI_FACTOR * gain));
            }
        }
    }
    if matches!(phase, TouchPhase::Ended) {
        out.push(ScrollInput::End);
    }
    out
}

pub fn debug_enabled() -> bool {
    std::env::var("BATO_SCROLL_DEBUG").map(|v| v == "1").unwrap_or(false)
}

/// One vertical scroller: physics + the measured viewport/content.
pub struct Scroller {
    pub name: &'static str,
    pub axis: ScrollAxis,
    pub handle: gpui::ScrollHandle,
    pub viewport_h: f32,
    pub content_h: f32,
    /// Thumb drag in progress: (pointer y at grab, offset at grab).
    pub drag: Option<(f32, f32)>,
    pub thumb_hover: bool,
    /// Offset to apply once the content has been measured (BATO_VIEW scroll=).
    pub pending_offset: Option<f32>,
    gain: f32,
    debug: bool,
}

impl Scroller {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            axis: ScrollAxis::default(),
            handle: gpui::ScrollHandle::new(),
            viewport_h: 0.0,
            content_h: 0.0,
            drag: None,
            thumb_hover: false,
            pending_offset: None,
            gain: touchpad_gain(),
            debug: debug_enabled(),
        }
    }

    /// Feed a GPUI scroll event. Returns true if a frame is needed.
    pub fn on_scroll(&mut self, ev: &ScrollEvent) -> bool {
        let now = Instant::now();
        let inputs = map_event(ev.delta, ev.touch_phase, self.gain);
        if self.debug {
            let (kind, dx, dy) = match ev.delta {
                ScrollDelta::Lines(l) => ("lines", l.x, l.y),
                ScrollDelta::Pixels(p) => ("pixels", f32::from(p.x), f32::from(p.y)),
            };
            eprintln!(
                "[scroll {}] t={:?} kind={kind} dx={dx:.3} dy={dy:.3} phase={:?} source={:?} -> {:?} offset={:.1}/{:.1}",
                self.name, now, ev.touch_phase, ev.source, inputs, self.axis.offset(), self.axis.max()
            );
        }
        for i in &inputs {
            match *i {
                ScrollInput::Notches(n) => self.axis.wheel_notches(n, now),
                ScrollInput::Precise(px) => self.axis.precise(px, now),
                ScrollInput::End => {
                    let flung = self.axis.precise_end(now);
                    if self.debug {
                        eprintln!("[scroll {}] gesture end, fling={flung}", self.name);
                    }
                }
            }
        }
        !inputs.is_empty()
    }

    /// Per frame: sync measured sizes, advance physics, push the offset into
    /// the GPUI scroll handle. Returns true while animating.
    pub fn frame(&mut self) -> bool {
        let max = self.handle.max_offset().y;
        let bounds = self.handle.bounds();
        self.viewport_h = f32::from(bounds.size.height);
        self.content_h = self.viewport_h + f32::from(max);
        self.axis.set_max(f32::from(max));
        if let Some(p) = self.pending_offset {
            if self.axis.max() > 0.0 {
                self.axis.set_offset(p);
                self.pending_offset = None;
            }
        }
        let moving = self.axis.tick(Instant::now());
        self.handle.set_offset(gpui::point(gpui::px(0.), gpui::px(-self.axis.offset())));
        moving || self.pending_offset.is_some()
    }

    pub fn overflows(&self) -> bool {
        self.axis.max() > 0.5
    }

    /// Thumb (top, length) in px along a track of `track` px, Chromium style
    /// (min length 2 × thickness = 10px at 100%).
    pub fn thumb(&self, track: f32, min_len: f32) -> (f32, f32) {
        if self.content_h <= 0.0 || !self.overflows() {
            return (0.0, track);
        }
        let len = (track * self.viewport_h / self.content_h).clamp(min_len.min(track), track);
        let top = (track - len) * (self.axis.offset() / self.axis.max().max(1.0));
        (top, len)
    }

    /// Thumb drag: pointer moved `dy` px from the grab point.
    pub fn drag_to(&mut self, pointer_y: f32, track: f32, min_len: f32) {
        if let Some((y0, off0)) = self.drag {
            let (_, len) = self.thumb(track, min_len);
            let room = (track - len).max(1.0);
            let off = off0 + (pointer_y - y0) * self.axis.max() / room;
            self.axis.set_offset(off);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{point, px};

    #[test]
    fn wheel_notch_maps_to_one_notch() {
        // value120 = 120 (one notch down) arrives as Lines(y = -3).
        let v = map_event(ScrollDelta::Lines(point(0.0, -3.0)), TouchPhase::Moved, 12.0);
        assert_eq!(v, vec![ScrollInput::Notches(1.0)]);
    }

    #[test]
    fn hi_res_wheel_fraction_survives() {
        // value120 = -15 (1/8 notch up) → Lines(y = 0.375).
        let v = map_event(ScrollDelta::Lines(point(0.0, 15.0 / 120.0 * 3.0)), TouchPhase::Moved, 12.0);
        assert_eq!(v, vec![ScrollInput::Notches(-0.125)]);
    }

    #[test]
    fn touchpad_pixels_use_chromium_gain() {
        // wl axis value 2.0 (finger, downwards) → GPUI Pixels(y = -6).
        let v = map_event(ScrollDelta::Pixels(point(px(0.), px(-6.0))), TouchPhase::Moved, 12.0);
        assert_eq!(v, vec![ScrollInput::Precise(24.0)]);
    }

    #[test]
    fn axis_stop_ends_the_gesture() {
        let v = map_event(ScrollDelta::Pixels(point(px(0.), px(0.))), TouchPhase::Ended, 12.0);
        assert_eq!(v, vec![ScrollInput::End]);
        let v = map_event(ScrollDelta::Pixels(point(px(0.), px(-3.0))), TouchPhase::Ended, 12.0);
        assert_eq!(v, vec![ScrollInput::Precise(12.0), ScrollInput::End]);
    }

    #[test]
    fn horizontal_only_is_ignored() {
        let v = map_event(ScrollDelta::Lines(point(-3.0, 0.0)), TouchPhase::Moved, 12.0);
        assert!(v.is_empty());
    }

    #[test]
    fn fling_starts_after_gesture_end() {
        let mut s = Scroller::new("t");
        s.axis.set_max(100_000.0);
        let t = Instant::now();
        for i in 0..10 {
            for inp in map_event(ScrollDelta::Pixels(point(px(0.), px(-3.0))), TouchPhase::Moved, 12.0) {
                if let ScrollInput::Precise(p) = inp {
                    s.axis.precise(p, t + std::time::Duration::from_millis(i * 10));
                }
            }
        }
        assert!(s.axis.precise_end(t + std::time::Duration::from_millis(95)));
    }
}
