//! BatoScroll middle-click autoscroll, toolkit-neutral.
//!
//! A port of `~/Code/BatoScroll/src/content.js` (vendored into Electron clients
//! as `electron/batoscroll.js`), so native clients behave exactly like the
//! extension and the Electron apps. Spec: `native/HYBRID-SPEC.md` §6.1.
//!
//! The toolkit owns events, hit-testing and drawing; this owns the decisions:
//!
//! | Toolkit event                                   | Call |
//! |-------------------------------------------------|------|
//! | Middle press on an eligible spot (see below)     | [`Autoscroll::press_middle`] → consume the press |
//! | Pointer moved (anywhere in the window)           | [`Autoscroll::pointer_moved`] |
//! | Middle released                                  | [`Autoscroll::release_middle`] |
//! | Any press while engaged                          | [`Autoscroll::press_other`] → `true`: consume it **and its release** |
//! | Esc, any wheel/touchpad scroll, focus lost       | [`Autoscroll::cancel`] (consume the Esc/wheel) |
//! | Every frame while [`Autoscroll::is_engaged`]     | [`Autoscroll::tick`] → scroll by the returned whole-device-pixel step, instantly |
//!
//! Eligible press: not on a text input/textarea/select/editable, and not inside
//! anything marked "batoscroll-ignore" (a component that uses the middle
//! button itself); and the scroll target under the pointer can actually scroll.
//! Buttons and links *are* eligible (embedded mode). If nothing under the
//! pointer can scroll, don't call `press_middle` — the click passes through.
//!
//! While engaged: draw the compass ([`Autoscroll::origin`],
//! [`Autoscroll::direction`], [`AutoscrollConfig::indicator_size`]), show the
//! `all-scroll` cursor, swallow all pointer input to the content, and suspend
//! hover effects and text selection. Stop any wheel animation/fling on the
//! target axis when it engages (`ScrollAxis::stop`) and move it with
//! `ScrollAxis::set_offset(offset + step)`.

use std::path::PathBuf;
use std::time::Instant;

/// `~/.config/bato/batoscroll.json`, shared by every Bato app and the extension
/// options. Missing keys use defaults; values are clamped like settings.js.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AutoscrollConfig {
    /// px radius per axis around the origin with zero speed.
    pub dead_zone: f32,
    /// px/s per px beyond the dead zone.
    pub sensitivity: f32,
    /// px/s cap per axis.
    pub max_speed: f32,
    /// Compass diameter, px.
    pub indicator_size: f32,
    /// Travel (px) while held that makes it hold-to-scroll instead of toggle.
    pub move_threshold: f32,
    /// Velocity easing time constant, ms (0 = instant).
    pub smoothing_ms: f32,
    pub invert: bool,
}

impl Default for AutoscrollConfig {
    fn default() -> Self {
        Self {
            dead_zone: 10.0,
            sensitivity: 15.0,
            max_speed: 15000.0,
            indicator_size: 44.0,
            move_threshold: 12.0,
            smoothing_ms: 45.0,
            invert: false,
        }
    }
}

impl AutoscrollConfig {
    pub fn path() -> PathBuf {
        let base = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .unwrap_or_else(|| PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".config"));
        base.join("bato").join("batoscroll.json")
    }

    /// Never fails: a missing or corrupt file yields defaults.
    pub fn load() -> Self {
        std::fs::read_to_string(Self::path()).map(|s| Self::from_json(&s)).unwrap_or_default()
    }

    pub fn from_json(s: &str) -> Self {
        let d = Self::default();
        let Ok(serde_json::Value::Object(v)) = serde_json::from_str::<serde_json::Value>(s) else {
            return d;
        };
        // settings.js clampNum: non-finite → fallback, else clamp.
        let num = |key: &str, min: f32, max: f32, fallback: f32| {
            v.get(key)
                .and_then(|x| x.as_f64())
                .filter(|x| x.is_finite())
                .map(|x| (x as f32).clamp(min, max))
                .unwrap_or(fallback)
        };
        Self {
            dead_zone: num("deadZone", 0.0, 300.0, d.dead_zone),
            sensitivity: num("sensitivity", 1.0, 200.0, d.sensitivity),
            max_speed: num("maxSpeed", 100.0, 40000.0, d.max_speed),
            indicator_size: num("indicatorSize", 20.0, 160.0, d.indicator_size),
            move_threshold: num("moveThreshold", 2.0, 100.0, d.move_threshold),
            smoothing_ms: num("smoothing", 0.0, 200.0, d.smoothing_ms),
            invert: v.get("invert").and_then(|x| x.as_bool()).unwrap_or(d.invert),
        }
    }

    /// Target velocity (px/s) for a cursor offset from the origin on one axis.
    pub fn axis_velocity(&self, delta: f32) -> f32 {
        let mag = delta.abs();
        if mag <= self.dead_zone {
            return 0.0;
        }
        let v = ((mag - self.dead_zone) * self.sensitivity).min(self.max_speed);
        delta.signum() * v * if self.invert { -1.0 } else { 1.0 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoscrollState {
    Idle,
    /// Middle button held; scrolling; compass shown.
    Armed,
    /// Toggle mode: button released after a clean click; keeps scrolling
    /// until any click, Esc, wheel or focus loss.
    Active,
}

/// Which compass arrows are lit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Direction {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

const VEL_EPSILON: f64 = 0.5;
const MAX_DT: f64 = 0.05;

#[derive(Debug, Clone)]
pub struct Autoscroll {
    config: AutoscrollConfig,
    state: AutoscrollState,
    origin: (f32, f32),
    cursor: (f32, f32),
    max_travel: f32,
    vel: (f64, f64),
    acc: (f64, f64),
    last: Option<Instant>,
}

impl Default for Autoscroll {
    fn default() -> Self {
        Self::new(AutoscrollConfig::default())
    }
}

impl Autoscroll {
    pub fn new(config: AutoscrollConfig) -> Self {
        Self {
            config,
            state: AutoscrollState::Idle,
            origin: (0.0, 0.0),
            cursor: (0.0, 0.0),
            max_travel: 0.0,
            vel: (0.0, 0.0),
            acc: (0.0, 0.0),
            last: None,
        }
    }

    pub fn config(&self) -> &AutoscrollConfig {
        &self.config
    }

    /// The extension re-reads settings on every page load; call this when a
    /// gesture starts if you want live changes (cheap: one small file).
    pub fn set_config(&mut self, config: AutoscrollConfig) {
        self.config = config;
    }

    pub fn state(&self) -> AutoscrollState {
        self.state
    }

    /// Armed or Active: overlay up, input captured, frames needed.
    pub fn is_engaged(&self) -> bool {
        self.state != AutoscrollState::Idle
    }

    /// Compass centre, window coordinates (logical px).
    pub fn origin(&self) -> (f32, f32) {
        self.origin
    }

    /// Lit arrows, from the *target* velocity (like indicator.js).
    pub fn direction(&self) -> Direction {
        let vx = self.config.axis_velocity(self.cursor.0 - self.origin.0);
        let vy = self.config.axis_velocity(self.cursor.1 - self.origin.1);
        Direction { up: vy < 0.0, down: vy > 0.0, left: vx < 0.0, right: vx > 0.0 }
    }

    /// Middle button pressed on an eligible spot. Returns true if it engaged
    /// (consume the press). A middle press while already engaged stops it
    /// (also consumed) — same as any other press.
    pub fn press_middle(&mut self, x: f32, y: f32, now: Instant) -> bool {
        if self.is_engaged() {
            self.stop();
            return true;
        }
        self.state = AutoscrollState::Armed;
        self.origin = (x, y);
        self.cursor = (x, y);
        self.max_travel = 0.0;
        self.vel = (0.0, 0.0);
        self.acc = (0.0, 0.0);
        self.last = Some(now);
        true
    }

    pub fn pointer_moved(&mut self, x: f32, y: f32) {
        if !self.is_engaged() {
            return;
        }
        self.cursor = (x, y);
        if self.state == AutoscrollState::Armed {
            let travel = (x - self.origin.0).hypot(y - self.origin.1);
            self.max_travel = self.max_travel.max(travel);
        }
    }

    /// Middle button released. Returns true if the release should be consumed
    /// (it always is while engaged).
    pub fn release_middle(&mut self) -> bool {
        match self.state {
            AutoscrollState::Armed => {
                if self.max_travel > self.config.move_threshold {
                    self.stop(); // hold-to-scroll: releasing ends it
                } else {
                    self.state = AutoscrollState::Active; // clean click: toggle mode
                }
                true
            }
            AutoscrollState::Active => true,
            AutoscrollState::Idle => false,
        }
    }

    /// Any non-middle press. While engaged it stops autoscroll; return value
    /// true means: consume this press **and its matching release**, so the
    /// click never reaches the content (BatoScroll swallows the trailing click).
    pub fn press_other(&mut self) -> bool {
        if self.is_engaged() {
            self.stop();
            true
        } else {
            false
        }
    }

    /// Esc, wheel/touchpad scroll, window blur: stop immediately.
    /// Returns true if it was engaged (consume the Esc / wheel event).
    pub fn cancel(&mut self) -> bool {
        let was = self.is_engaged();
        self.stop();
        was
    }

    fn stop(&mut self) {
        self.state = AutoscrollState::Idle;
        self.vel = (0.0, 0.0);
        self.acc = (0.0, 0.0);
        self.last = None;
    }

    /// Advance one frame. Returns the (dx, dy) to scroll by *now*, in logical
    /// px, quantized to whole device pixels (`device_scale` = output scale ×
    /// UI scale, whatever maps logical px to physical). Positive dy = content
    /// moves up (offset grows). Keep requesting frames while engaged.
    pub fn tick(&mut self, now: Instant, device_scale: f32) -> (f32, f32) {
        if !self.is_engaged() {
            return (0.0, 0.0);
        }
        let last = self.last.replace(now).unwrap_or(now);
        let dt = now.saturating_duration_since(last).as_secs_f64().min(MAX_DT);

        let tx = self.config.axis_velocity(self.cursor.0 - self.origin.0) as f64;
        let ty = self.config.axis_velocity(self.cursor.1 - self.origin.1) as f64;
        let tau = self.config.smoothing_ms as f64 / 1000.0;
        let alpha = if tau > 0.0 { 1.0 - (-dt / tau).exp() } else { 1.0 };
        self.vel.0 += (tx - self.vel.0) * alpha;
        self.vel.1 += (ty - self.vel.1) * alpha;
        if self.vel.0.abs() < VEL_EPSILON {
            self.vel.0 = 0.0;
        }
        if self.vel.1.abs() < VEL_EPSILON {
            self.vel.1 = 0.0;
        }

        self.acc.0 += self.vel.0 * dt;
        self.acc.1 += self.vel.1 * dt;
        let dpr = device_scale.max(0.01) as f64;
        let sx = (self.acc.0 * dpr).trunc() / dpr;
        let sy = (self.acc.1 * dpr).trunc() / dpr;
        self.acc.0 -= sx;
        self.acc.1 -= sy;
        (sx as f32, sy as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    #[test]
    fn config_defaults_and_clamping() {
        assert_eq!(AutoscrollConfig::from_json("not json"), AutoscrollConfig::default());
        let c = AutoscrollConfig::from_json(r#"{"deadZone": 999, "sensitivity": 0, "smoothing": "x", "invert": true}"#);
        assert_eq!(c.dead_zone, 300.0);
        assert_eq!(c.sensitivity, 1.0);
        assert_eq!(c.smoothing_ms, 45.0);
        assert!(c.invert);
        assert_eq!(c.max_speed, 15000.0);
    }

    #[test]
    fn speed_curve() {
        let c = AutoscrollConfig::default();
        assert_eq!(c.axis_velocity(10.0), 0.0, "inside dead zone");
        assert_eq!(c.axis_velocity(20.0), 150.0, "(20-10)*15");
        assert_eq!(c.axis_velocity(-20.0), -150.0);
        assert_eq!(c.axis_velocity(5000.0), 15000.0, "capped");
        let inv = AutoscrollConfig { invert: true, ..c };
        assert_eq!(inv.axis_velocity(20.0), -150.0);
    }

    #[test]
    fn clean_click_enters_toggle_mode_and_next_click_stops() {
        let t = Instant::now();
        let mut a = Autoscroll::default();
        assert!(a.press_middle(100.0, 100.0, t));
        a.pointer_moved(104.0, 103.0); // 5 px < 12 threshold
        assert!(a.release_middle());
        assert_eq!(a.state(), AutoscrollState::Active);
        assert!(a.press_other(), "click while active is swallowed");
        assert_eq!(a.state(), AutoscrollState::Idle);
        assert!(!a.press_other(), "idle clicks pass through");
    }

    #[test]
    fn moving_while_held_is_hold_mode() {
        let t = Instant::now();
        let mut a = Autoscroll::default();
        a.press_middle(100.0, 100.0, t);
        a.pointer_moved(100.0, 140.0);
        a.pointer_moved(100.0, 105.0); // max travel counts, not final position
        a.release_middle();
        assert_eq!(a.state(), AutoscrollState::Idle);
    }

    #[test]
    fn scrolls_with_eased_velocity_in_whole_device_pixels() {
        let t = Instant::now();
        let mut a = Autoscroll::default();
        a.press_middle(0.0, 0.0, t);
        a.pointer_moved(0.0, 30.0); // target 300 px/s down
        assert_eq!(a.direction(), Direction { down: true, ..Default::default() });
        let mut total = 0.0;
        let mut now = t;
        for _ in 0..120 {
            now += ms(16);
            let (dx, dy) = a.tick(now, 1.0);
            assert_eq!(dx, 0.0);
            assert_eq!(dy, dy.trunc(), "whole px at scale 1");
            total += dy;
        }
        // ~1.92 s at 300 px/s minus the ease-in (~45 ms) ≈ 560 px.
        assert!((total - 562.0).abs() < 10.0, "{total}");
        // At scale 1.5 steps are multiples of 1/1.5 px.
        let (_, dy) = a.tick(now + ms(16), 1.5);
        assert!(((dy * 1.5) - (dy * 1.5).round()).abs() < 1e-4);
    }

    #[test]
    fn smoothing_eases_in_and_dead_zone_stops() {
        let t = Instant::now();
        let mut a = Autoscroll::default();
        a.press_middle(0.0, 0.0, t);
        a.pointer_moved(0.0, 1010.0); // way past max → 15000 px/s target
        let (_, first) = a.tick(t + ms(16), 1.0);
        assert!(first < 15000.0 * 0.016 * 0.5, "eased, not instant: {first}");
        a.pointer_moved(0.0, 5.0); // back in the dead zone
        let mut now = t + ms(16);
        for _ in 0..60 {
            now += ms(16);
            a.tick(now, 1.0);
        }
        assert_eq!(a.tick(now + ms(16), 1.0), (0.0, 0.0), "decays to rest");
        assert!(a.is_engaged(), "still engaged until a click/Esc");
    }

    #[test]
    fn long_frame_gaps_are_clamped() {
        let t = Instant::now();
        let mut a = Autoscroll::new(AutoscrollConfig { smoothing_ms: 0.0, ..Default::default() });
        a.press_middle(0.0, 0.0, t);
        a.pointer_moved(0.0, 110.0); // 1500 px/s
        let (_, dy) = a.tick(t + Duration::from_secs(2), 1.0);
        assert_eq!(dy, 75.0, "dt clamped to 50 ms");
    }

    #[test]
    fn cancel_and_middle_press_while_engaged_stop() {
        let t = Instant::now();
        let mut a = Autoscroll::default();
        a.press_middle(0.0, 0.0, t);
        a.release_middle();
        assert!(a.press_middle(50.0, 50.0, t + ms(500)), "consumed");
        assert!(!a.is_engaged());
        a.press_middle(0.0, 0.0, t);
        assert!(a.cancel());
        assert!(!a.cancel());
    }
}
