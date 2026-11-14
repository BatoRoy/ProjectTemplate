//! Window-level input routing for the shell: scrollbar thumb drag, the
//! touchscreen (claimed touches → `touch::TouchRouter`, taps → synthetic
//! clicks) and BatoScroll autoscroll (capture-phase listeners that win over
//! every element underneath).

use std::time::Instant;

use gpui::{
    canvas, div, img, point, prelude::*, px, AnyElement, Context, CursorStyle, DispatchPhase, Modifiers, MouseButton,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, PlatformInput, ScrollEvent, TouchDragEvent, TouchPhase, Window,
};
use scroll_physics::TouchOutcome;

use crate::app::{Root, View};
use crate::autoscroll_ui::{eligible, Verdict};
use crate::theme::{ca, Theme};
use crate::ui;

/// A left click at `position`, as Chromium emits it at the end of a tap:
/// move, press, release.
pub fn synthetic_click(window: &mut Window, cx: &mut gpui::App, position: gpui::Point<gpui::Pixels>) {
    let modifiers = Modifiers::default();
    window.dispatch_event(PlatformInput::MouseMove(MouseMoveEvent { position, pressed_button: None, modifiers }), cx);
    window.dispatch_event(
        PlatformInput::MouseDown(MouseDownEvent { button: MouseButton::Left, position, modifiers, click_count: 1, first_mouse: false }),
        cx,
    );
    window.dispatch_event(PlatformInput::MouseUp(MouseUpEvent { button: MouseButton::Left, position, modifiers, click_count: 1 }), cx);
}

impl Root {
    fn autoscroll_eligible(&self, pos: gpui::Point<gpui::Pixels>, cx: &gpui::App) -> bool {
        let mut inputs = Vec::new();
        if self.view == View::Home {
            inputs.push(self.name_input.read(cx).control_bounds.get());
        }
        eligible(pos, self.main.handle.bounds(), self.main.axis.max(), self.modal.is_some(), &inputs)
    }

    fn on_capture_down(&mut self, ev: &MouseDownEvent, cx: &mut Context<Self>) -> Verdict {
        let eligible = self.autoscroll_eligible(ev.position, cx);
        let (v, engaged) = self.auto.press(ev.button, ev.position, eligible, Instant::now());
        if engaged {
            // Engaging stops any wheel glide or fling on the target.
            self.main.axis.stop();
        }
        if v == Verdict::Consume {
            cx.notify();
        }
        v
    }

    /// Touchscreen: GPUI's recognizer offers every touch at touch-down; we
    /// claim it in `input_capture` and drive `TouchDrag` with the stream.
    /// Returns the outcome (a `Tap` is turned into a click by the caller).
    fn on_touch(&mut self, ev: &TouchDragEvent, cx: &mut Context<Self>) -> Option<TouchOutcome> {
        if ev.phase == TouchPhase::Started && self.auto.cancel("touch-down") == Verdict::Consume {
            self.touch_ignore = true;
            cx.notify();
        }
        if self.touch_ignore {
            if matches!(ev.phase, TouchPhase::Ended | TouchPhase::Cancelled) {
                self.touch_ignore = false;
            }
            return None;
        }
        let area = (self.modal.is_none()).then(|| self.main.handle.bounds());
        let outcome = self.touch.handle(ev.phase, ev.position, Instant::now(), &mut self.main.axis, area);
        if self.scroll_debug {
            eprintln!(
                "[touch] phase={:?} id=first pos=({:.1},{:.1}) start=({:.1},{:.1}) main={} -> {:?} offset={:.1}",
                ev.phase,
                f32::from(ev.position.x),
                f32::from(ev.position.y),
                f32::from(ev.start_position.x),
                f32::from(ev.start_position.y),
                self.touch.on_main(),
                outcome,
                self.main.axis.offset()
            );
        }
        cx.notify();
        Some(outcome)
    }

    /// Window-level listeners, registered every frame from a zero-size canvas.
    pub fn input_capture(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let dragging = self.main.drag.is_some();
        canvas(|_, _, _| {}, move |_, _, window, _| {
            // Scrollbar thumb drag: follow the pointer anywhere while held.
            if dragging {
                let e = entity.clone();
                window.on_mouse_event(move |ev: &MouseMoveEvent, _, _, cx| {
                    e.update(cx, |this, cx| {
                        let s = cx.global::<Theme>().s;
                        this.main.drag_to(f32::from(ev.position.y), this.main.viewport_h, 10.0 * s);
                        cx.notify();
                    });
                });
                let e = entity.clone();
                window.on_mouse_event(move |_: &MouseUpEvent, _, _, cx| {
                    e.update(cx, |this, cx| {
                        this.main.drag = None;
                        cx.notify();
                    });
                });
            }

            // Touch: claim the gesture (keeps GPUI's own pan/tap recognizer
            // out) and run TouchDrag; a tap becomes a click after this dispatch.
            let e = entity.clone();
            window.on_mouse_event(move |ev: &TouchDragEvent, phase, window, cx| {
                if phase != DispatchPhase::Capture {
                    return;
                }
                if ev.phase == TouchPhase::Started {
                    window.prevent_default();
                }
                let outcome = e.update(cx, |this, cx| this.on_touch(ev, cx));
                if outcome == Some(TouchOutcome::Tap) {
                    let at = ev.position;
                    window.defer(cx, move |window, cx| synthetic_click(window, cx, at));
                }
            });

            // BatoScroll: capture phase, so it wins over the element underneath.
            let e = entity.clone();
            window.on_mouse_event(move |ev: &MouseDownEvent, phase, _, cx| {
                if phase == DispatchPhase::Capture && e.update(cx, |this, cx| this.on_capture_down(ev, cx)) == Verdict::Consume {
                    cx.stop_propagation();
                }
            });
            let e = entity.clone();
            window.on_mouse_event(move |ev: &MouseUpEvent, phase, _, cx| {
                if phase == DispatchPhase::Capture
                    && e.update(cx, |this, cx| {
                        let v = this.auto.release(ev.button);
                        if v == Verdict::Consume {
                            cx.notify();
                        }
                        v
                    }) == Verdict::Consume
                {
                    cx.stop_propagation();
                }
            });
            // Registered every frame (not only while engaged): the press that
            // engages and the next move can arrive before a frame is drawn.
            let e = entity.clone();
            window.on_mouse_event(move |ev: &MouseMoveEvent, phase, _, cx| {
                if phase == DispatchPhase::Capture
                    && e.update(cx, |this, cx| {
                        let v = this.auto.moved(ev.position);
                        if v == Verdict::Consume {
                            cx.notify();
                        }
                        v
                    }) == Verdict::Consume
                {
                    cx.stop_propagation();
                }
            });
            let e = entity.clone();
            window.on_mouse_event(move |_: &ScrollEvent, phase, _, cx| {
                if phase == DispatchPhase::Capture
                    && e.update(cx, |this, cx| {
                        let v = this.auto.cancel("wheel");
                        if v == Verdict::Consume {
                            cx.notify();
                        }
                        v
                    }) == Verdict::Consume
                {
                    cx.stop_propagation();
                }
            });
        })
        .absolute()
        .size_0()
    }

    /// Full-window overlay while autoscroll is engaged: occludes the content
    /// (no hover, no clicks), shows the all-scroll-like cursor and the compass.
    pub fn render_autoscroll(&self, t: &Theme) -> Option<AnyElement> {
        if !self.auto.engaged() {
            return None;
        }
        let a = &self.auto.auto;
        let (ox, oy) = a.origin();
        let d = a.direction();
        let bits: String = [d.up, d.down, d.left, d.right].iter().map(|b| if *b { '1' } else { '0' }).collect();
        // Pop-in: scale .6 → 1 and fade, 120 ms ease-out.
        let f = ui::ease_out((self.auto.since.elapsed().as_secs_f32() / 0.12).min(1.0));
        let size = a.config().indicator_size * t.s * (0.6 + 0.4 * f);
        let s = t.s;
        Some(
            div()
                .id("autoscroll-overlay")
                .absolute()
                .inset_0()
                .occlude()
                // GPUI has no `all-scroll` cursor; Crosshair is the closest shape.
                .cursor(CursorStyle::Crosshair)
                .child(
                    div()
                        .absolute()
                        .left(px(ox - size / 2.0))
                        .top(px(oy - size / 2.0))
                        .size(px(size))
                        .opacity(f)
                        .rounded_full()
                        // drop-shadow(0 1px 3px rgba(0,0,0,.45))
                        .shadow(vec![gpui::BoxShadow {
                            color: ca(bato_theme::Rgb::BLACK, 0.45),
                            offset: point(px(0.), px(s)),
                            blur_radius: px(1.5 * s),
                            spread_radius: px(0.),
                            inset: false,
                        }])
                        .child(img(format!("icons/compass-{bits}.svg")).size_full()),
                )
                .into_any_element(),
        )
    }
}
