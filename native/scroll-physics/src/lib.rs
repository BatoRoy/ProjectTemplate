//! Chromium-style scroll physics, independent of any UI toolkit.
//!
//! The previous native attempts (Qt/QML, Tauri/WebKitGTK) failed on *feel*:
//! wheel notches jumped instead of gliding, and touchpads had no fling. This
//! crate ports the behaviour of Chromium on Linux, which is the reference the
//! Electron apps set, so every native client scrolls the same way regardless of
//! the toolkit drawing it.
//!
//! One [`ScrollAxis`] per scrollable axis. The toolkit feeds it input and asks
//! for the offset each frame:
//!
//! | Input                                   | Call                                 |
//! |-----------------------------------------|--------------------------------------|
//! | Mouse wheel (notched or hi-res value120) | [`ScrollAxis::wheel`] (animated)     |
//! | Touchpad / pixel deltas                 | [`ScrollAxis::precise`] (1:1)         |
//! | Touchpad fingers lifted (`axis_stop`)   | [`ScrollAxis::precise_end`] (fling)  |
//! | Scrollbar drag, restore, jump           | [`ScrollAxis::set_offset`]           |
//! | Smooth programmatic scroll              | [`ScrollAxis::animate_to`]           |
//! | Every frame while animating             | [`ScrollAxis::tick`] → keep asking for frames while it returns `true` |
//!
//! Sign convention: a positive delta moves the content *down* (offset grows),
//! i.e. a wheel turned toward the user. Offsets are logical pixels in
//! `0..=max`.
//!
//! References (Chromium `main`):
//! - `cc/animation/scroll_offset_animation_curve.cc` (ease-in-out, inverse-delta
//!   duration, retargeting with velocity continuity)
//! - `ui/events/gestures/fling_curve.cc` (fling curve constants)
//! - `ui/events/event.cc` (`kWheelDelta = 120` px per notch on Linux)

use std::time::{Duration, Instant};

/// Logical pixels per wheel notch. Chromium on Linux uses 120 since M109.
pub const PX_PER_NOTCH: f32 = 120.0;

/// Tunables. Defaults are Chromium's.
#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub px_per_notch: f32,
    /// Set false to make wheel scrolling jump (accessibility / reduced motion).
    pub smooth_wheel: bool,
    /// Flings slower than this (px/s) are dropped.
    pub min_fling_velocity: f32,
    /// If the fingers rested this long before lifting, there is no fling.
    pub fling_stopped_after: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            px_per_notch: PX_PER_NOTCH,
            smooth_wheel: true,
            min_fling_velocity: 50.0,
            fling_stopped_after: Duration::from_millis(40),
        }
    }
}

// ─── Cubic bezier timing function ───────────────────────────────────────────

/// `cubic-bezier(x1, y1, x2, y2)` with endpoints (0,0) and (1,1).
#[derive(Debug, Clone, Copy)]
struct Bezier {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
}

impl Bezier {
    /// Chromium's `EaseInOutWithInitialSlope`: ease-in-out (0.42, 0, 0.58, 1)
    /// with the first control point scaled so the curve starts at `slope`.
    fn ease_in_out_with_slope(slope: f64) -> Self {
        let slope = slope.clamp(-1000.0, 1000.0);
        Self { x1: 0.42, y1: 0.42 * slope, x2: 0.58, y2: 1.0 }
    }

    fn coord(p1: f64, p2: f64, s: f64) -> f64 {
        let inv = 1.0 - s;
        3.0 * inv * inv * s * p1 + 3.0 * inv * s * s * p2 + s * s * s
    }

    fn deriv(p1: f64, p2: f64, s: f64) -> f64 {
        let inv = 1.0 - s;
        3.0 * inv * inv * p1 + 6.0 * inv * s * (p2 - p1) + 3.0 * s * s * (1.0 - p2)
    }

    /// Curve parameter for a given x (time fraction). x(s) is monotonic for
    /// x1, x2 in [0, 1], so Newton with a bisection fallback converges.
    fn solve_s(&self, x: f64) -> f64 {
        let x = x.clamp(0.0, 1.0);
        let mut s = x;
        for _ in 0..8 {
            let err = Self::coord(self.x1, self.x2, s) - x;
            if err.abs() < 1e-7 {
                return s;
            }
            let d = Self::deriv(self.x1, self.x2, s);
            if d.abs() < 1e-6 {
                break;
            }
            s -= err / d;
        }
        let (mut lo, mut hi) = (0.0, 1.0);
        s = x;
        for _ in 0..40 {
            let v = Self::coord(self.x1, self.x2, s);
            if (v - x).abs() < 1e-7 {
                break;
            }
            if v < x { lo = s } else { hi = s }
            s = (lo + hi) / 2.0;
        }
        s
    }

    /// Progress (0..1, may overshoot with steep initial slopes) at time fraction x.
    fn value(&self, x: f64) -> f64 {
        Self::coord(self.y1, self.y2, self.solve_s(x))
    }

    /// d(progress)/d(time fraction) at x.
    fn slope(&self, x: f64) -> f64 {
        let s = self.solve_s(x);
        let dx = Self::deriv(self.x1, self.x2, s);
        if dx.abs() < 1e-9 {
            return 0.0;
        }
        Self::deriv(self.y1, self.y2, s) / dx
    }
}

// ─── Wheel animation (ScrollOffsetAnimationCurve, kMouseWheel) ──────────────

/// Chromium's `kInverseDelta` duration: 12 frames (200 ms) up to 120 px,
/// ramping down to 6 frames (100 ms) at 480 px.
fn segment_duration(delta: f64) -> f64 {
    (14.0 - delta.abs() / 60.0).clamp(6.0, 12.0) / 60.0
}

#[derive(Debug, Clone, Copy)]
struct WheelAnim {
    start: Instant,
    from: f64,
    to: f64,
    duration: f64,
    curve: Bezier,
}

impl WheelAnim {
    fn new(now: Instant, from: f64, to: f64) -> Self {
        Self {
            start: now,
            from,
            to,
            duration: segment_duration(to - from),
            curve: Bezier::ease_in_out_with_slope(0.0),
        }
    }

    fn fraction(&self, now: Instant) -> f64 {
        if self.duration <= 0.0 {
            return 1.0;
        }
        (now.saturating_duration_since(self.start).as_secs_f64() / self.duration).min(1.0)
    }

    fn position(&self, now: Instant) -> f64 {
        let f = self.fraction(now);
        if f >= 1.0 {
            return self.to;
        }
        self.from + (self.to - self.from) * self.curve.value(f)
    }

    /// px/s
    fn velocity(&self, now: Instant) -> f64 {
        let f = self.fraction(now);
        if f >= 1.0 || self.duration <= 0.0 {
            return 0.0;
        }
        (self.to - self.from) * self.curve.slope(f) / self.duration
    }

    fn done(&self, now: Instant) -> bool {
        self.fraction(now) >= 1.0
    }

    /// Chromium `UpdateTarget`: restart from the current position toward the
    /// new target, keeping velocity continuous, and never taking longer than
    /// 2.5x what the current velocity would need.
    fn retarget(&mut self, now: Instant, new_to: f64) {
        let pos = self.position(now);
        let vel = self.velocity(now);
        let rem = new_to - pos;
        if rem.abs() < 0.01 {
            *self = Self { start: now, from: pos, to: new_to, duration: 0.0, curve: self.curve };
            return;
        }
        let mut duration = segment_duration(rem);
        if vel.abs() > 1e-6 {
            let bound = rem / vel * 2.5;
            if bound > 0.0 {
                duration = duration.min(bound);
            }
        }
        let slope = vel * duration / rem;
        *self = Self {
            start: now,
            from: pos,
            to: new_to,
            duration,
            curve: Bezier::ease_in_out_with_slope(slope),
        };
    }
}

// ─── Fling (ui/events/gestures/fling_curve.cc) ──────────────────────────────

const FLING_ALPHA: f64 = -5707.62;
const FLING_BETA: f64 = 172.0;
const FLING_GAMMA: f64 = 3.7;

fn fling_pos(t: f64) -> f64 {
    FLING_ALPHA * (-FLING_GAMMA * t).exp() - FLING_BETA * t - FLING_ALPHA
}

fn fling_time_at_velocity(v: f64) -> f64 {
    -((v + FLING_BETA) / (-FLING_ALPHA * FLING_GAMMA)).ln() / FLING_GAMMA
}

/// Highest start speed the curve can represent (~20 946 px/s).
fn fling_max_velocity() -> f64 {
    -FLING_ALPHA * FLING_GAMMA - FLING_BETA
}

#[derive(Debug, Clone, Copy)]
struct Fling {
    start: Instant,
    origin: f64,
    dir: f64,
    t0: f64,
    p0: f64,
    end: f64,
}

impl Fling {
    fn new(now: Instant, origin: f64, velocity: f64) -> Self {
        let speed = velocity.abs().min(fling_max_velocity());
        let t0 = fling_time_at_velocity(speed);
        Self {
            start: now,
            origin,
            dir: velocity.signum(),
            t0,
            p0: fling_pos(t0),
            end: fling_time_at_velocity(0.0),
        }
    }

    fn position(&self, now: Instant) -> f64 {
        let t = (self.t0 + now.saturating_duration_since(self.start).as_secs_f64()).min(self.end);
        self.origin + self.dir * (fling_pos(t) - self.p0)
    }

    fn done(&self, now: Instant) -> bool {
        self.t0 + now.saturating_duration_since(self.start).as_secs_f64() >= self.end
    }
}

// ─── Velocity tracking for touchpad ─────────────────────────────────────────

/// Least-squares velocity over the last 100 ms of precise deltas (the
/// horizon Chromium's/Android's VelocityTracker uses).
#[derive(Debug, Default, Clone)]
struct VelocityTracker {
    samples: Vec<(Instant, f64)>, // (time, cumulative position)
    pos: f64,
}

const HORIZON: Duration = Duration::from_millis(100);

impl VelocityTracker {
    fn reset(&mut self) {
        self.samples.clear();
        self.pos = 0.0;
    }

    fn add(&mut self, now: Instant, delta: f64) {
        self.pos += delta;
        self.samples.push((now, self.pos));
        self.samples.retain(|(t, _)| now.saturating_duration_since(*t) <= HORIZON);
        if self.samples.len() > 20 {
            let excess = self.samples.len() - 20;
            self.samples.drain(..excess);
        }
    }

    /// px/s; 0 when the pointer was resting before release.
    fn velocity(&self, now: Instant, stopped_after: Duration) -> f64 {
        let Some(&(last_t, _)) = self.samples.last() else { return 0.0 };
        if now.saturating_duration_since(last_t) > stopped_after || self.samples.len() < 2 {
            return 0.0;
        }
        let n = self.samples.len() as f64;
        let ts: Vec<f64> = self
            .samples
            .iter()
            .map(|(t, _)| t.saturating_duration_since(self.samples[0].0).as_secs_f64())
            .collect();
        let mean_t = ts.iter().sum::<f64>() / n;
        let mean_p = self.samples.iter().map(|s| s.1).sum::<f64>() / n;
        let mut num = 0.0;
        let mut den = 0.0;
        for (t, (_, p)) in ts.iter().zip(&self.samples) {
            num += (t - mean_t) * (p - mean_p);
            den += (t - mean_t) * (t - mean_t);
        }
        if den < 1e-9 { 0.0 } else { num / den }
    }
}

// ─── The axis ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
enum Motion {
    Idle,
    Wheel(WheelAnim),
    Fling(Fling),
}

/// Scroll state for one axis. Cheap to clone; holds no toolkit types.
#[derive(Debug, Clone)]
pub struct ScrollAxis {
    config: Config,
    offset: f64,
    max: f64,
    motion: Motion,
    tracker: VelocityTracker,
}

impl Default for ScrollAxis {
    fn default() -> Self {
        Self::new(Config::default())
    }
}

impl ScrollAxis {
    pub fn new(config: Config) -> Self {
        Self { config, offset: 0.0, max: 0.0, motion: Motion::Idle, tracker: VelocityTracker::default() }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn set_config(&mut self, config: Config) {
        self.config = config;
    }

    /// Current offset in logical px, already clamped to `0..=max`.
    pub fn offset(&self) -> f32 {
        self.offset as f32
    }

    pub fn max(&self) -> f32 {
        self.max as f32
    }

    /// Call whenever content or viewport size changes:
    /// `max = (content_len - viewport_len).max(0)`.
    pub fn set_max(&mut self, max: f32) {
        self.max = (max as f64).max(0.0);
        if self.offset > self.max {
            self.offset = self.max;
        }
        if let Motion::Wheel(a) = &mut self.motion {
            a.to = a.to.clamp(0.0, self.max);
        }
    }

    pub fn is_animating(&self) -> bool {
        !matches!(self.motion, Motion::Idle)
    }

    /// Mouse wheel, in notches (value120 / 120; fractional for hi-res wheels).
    pub fn wheel_notches(&mut self, notches: f32, now: Instant) {
        self.wheel(notches * self.config.px_per_notch, now);
    }

    /// Mouse wheel, in px. Animated: each event moves the target and the
    /// running animation is retargeted with continuous velocity.
    pub fn wheel(&mut self, delta_px: f32, now: Instant) {
        let delta = delta_px as f64;
        if !self.config.smooth_wheel {
            self.motion = Motion::Idle;
            self.offset = (self.offset + delta).clamp(0.0, self.max);
            return;
        }
        match &mut self.motion {
            Motion::Wheel(anim) if !anim.done(now) => {
                let to = (anim.to + delta).clamp(0.0, self.max);
                anim.retarget(now, to);
            }
            _ => {
                let to = (self.offset + delta).clamp(0.0, self.max);
                if (to - self.offset).abs() < 0.01 {
                    self.motion = Motion::Idle;
                    return;
                }
                self.motion = Motion::Wheel(WheelAnim::new(now, self.offset, to));
            }
        }
    }

    /// Touchpad / precise pixel deltas: applied immediately, 1:1. Cancels any
    /// running animation or fling (fingers down stops a fling, like Chromium).
    pub fn precise(&mut self, delta_px: f32, now: Instant) {
        if !matches!(self.motion, Motion::Idle) {
            self.motion = Motion::Idle;
            self.tracker.reset();
        }
        self.tracker.add(now, delta_px as f64);
        self.offset = (self.offset + delta_px as f64).clamp(0.0, self.max);
    }

    /// Fingers lifted (Wayland `axis_stop`, winit `TouchPhase::Ended`). Starts
    /// a kinetic fling from the recent velocity. Returns true if a fling started.
    pub fn precise_end(&mut self, now: Instant) -> bool {
        let v = self.tracker.velocity(now, self.config.fling_stopped_after);
        self.tracker.reset();
        if v.abs() < self.config.min_fling_velocity as f64 {
            return false;
        }
        self.motion = Motion::Fling(Fling::new(now, self.offset, v));
        true
    }

    /// Fingers touched the pad without moving (a hold gesture). Stops a fling.
    pub fn stop(&mut self) {
        self.motion = Motion::Idle;
        self.tracker.reset();
    }

    /// Jump (scrollbar drag, state restore). Cancels animation.
    pub fn set_offset(&mut self, offset: f32) {
        self.motion = Motion::Idle;
        self.offset = (offset as f64).clamp(0.0, self.max);
    }

    /// Smooth scroll to an absolute offset (scroll-into-view, Home/End keys).
    pub fn animate_to(&mut self, target: f32, now: Instant) {
        let to = (target as f64).clamp(0.0, self.max);
        match &mut self.motion {
            Motion::Wheel(anim) if !anim.done(now) => anim.retarget(now, to),
            _ => {
                if (to - self.offset).abs() >= 0.01 {
                    self.motion = Motion::Wheel(WheelAnim::new(now, self.offset, to));
                }
            }
        }
    }

    /// The target the content is heading to (equals `offset()` when idle).
    pub fn target(&self) -> f32 {
        match &self.motion {
            Motion::Wheel(a) => a.to as f32,
            _ => self.offset as f32,
        }
    }

    /// Advance to `now`. Returns true while still moving: the caller should
    /// request another frame (vsync-driven) and call `tick` again.
    pub fn tick(&mut self, now: Instant) -> bool {
        match self.motion {
            Motion::Idle => false,
            Motion::Wheel(anim) => {
                self.offset = anim.position(now).clamp(0.0, self.max);
                if anim.done(now) {
                    self.motion = Motion::Idle;
                    false
                } else {
                    true
                }
            }
            Motion::Fling(f) => {
                let p = f.position(now);
                self.offset = p.clamp(0.0, self.max);
                // Hitting an edge ends the fling (no elastic overscroll on Linux).
                if f.done(now) || p <= 0.0 || p >= self.max {
                    self.motion = Motion::Idle;
                    false
                } else {
                    true
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    fn axis(max: f32) -> ScrollAxis {
        let mut a = ScrollAxis::default();
        a.set_max(max);
        a
    }

    #[test]
    fn one_notch_glides_120px_in_200ms() {
        let t = Instant::now();
        let mut a = axis(10_000.0);
        a.wheel_notches(1.0, t);
        assert!(a.tick(t + ms(50)));
        let mid = a.offset();
        assert!(mid > 0.0 && mid < 120.0, "mid {mid}");
        assert!(a.tick(t + ms(100)));
        assert!((a.offset() - 60.0).abs() < 1.0, "ease-in-out is symmetric: {}", a.offset());
        assert!(!a.tick(t + ms(200)));
        assert_eq!(a.offset(), 120.0);
    }

    #[test]
    fn big_deltas_are_faster() {
        assert!((segment_duration(120.0) - 0.2).abs() < 1e-9);
        assert!((segment_duration(480.0) - 0.1).abs() < 1e-9);
        assert!((segment_duration(300.0) - (14.0 - 5.0) / 60.0).abs() < 1e-9);
    }

    #[test]
    fn retarget_keeps_velocity_continuous_and_accumulates() {
        let t = Instant::now();
        let mut a = axis(10_000.0);
        a.wheel_notches(1.0, t);
        a.tick(t + ms(80));
        let before = a.offset();
        let mut probe = a.clone();
        probe.tick(t + ms(81));
        let v_before = (probe.offset() - before) / 0.001;

        a.wheel_notches(1.0, t + ms(80));
        assert_eq!(a.target(), 240.0);
        a.tick(t + ms(81));
        let v_after = (a.offset() - before) / 0.001;
        assert!((v_after - v_before).abs() / v_before < 0.1, "{v_before} vs {v_after}");
        while a.tick(t + ms(1000)) {}
        assert_eq!(a.offset(), 240.0);
    }

    #[test]
    fn target_is_clamped_to_bounds() {
        let t = Instant::now();
        let mut a = axis(100.0);
        a.wheel_notches(5.0, t);
        assert_eq!(a.target(), 100.0);
        a.tick(t + ms(500));
        assert_eq!(a.offset(), 100.0);
        a.wheel_notches(1.0, t + ms(600));
        assert!(!a.is_animating(), "already at the end");
    }

    #[test]
    fn no_smooth_wheel_jumps() {
        let t = Instant::now();
        let mut a = axis(1000.0);
        a.set_config(Config { smooth_wheel: false, ..Config::default() });
        a.wheel_notches(1.0, t);
        assert_eq!(a.offset(), 120.0);
        assert!(!a.is_animating());
    }

    #[test]
    fn touchpad_is_one_to_one_then_flings() {
        let t = Instant::now();
        let mut a = axis(100_000.0);
        // 1000 px/s: 10 px every 10 ms for 100 ms.
        for i in 0..10 {
            a.precise(10.0, t + ms(i * 10));
        }
        assert_eq!(a.offset(), 100.0);
        assert!(a.precise_end(t + ms(95)));
        let start = a.offset();
        a.tick(t + ms(105));
        let v = (a.offset() - start) / 0.010;
        assert!((v - 1000.0).abs() < 150.0, "fling starts at release velocity: {v}");
        let mut now = t + ms(105);
        while a.tick(now) {
            now += ms(16);
        }
        assert!(a.offset() > start + 100.0, "glided on: {}", a.offset());
        assert!(now - t < ms(1500), "fling ends within ~1.3 s");
    }

    #[test]
    fn resting_fingers_do_not_fling() {
        let t = Instant::now();
        let mut a = axis(10_000.0);
        for i in 0..5 {
            a.precise(10.0, t + ms(i * 10));
        }
        assert!(!a.precise_end(t + ms(200)));
        assert!(!a.is_animating());
    }

    #[test]
    fn fingers_down_cancel_fling() {
        let t = Instant::now();
        let mut a = axis(100_000.0);
        for i in 0..10 {
            a.precise(30.0, t + ms(i * 10));
        }
        assert!(a.precise_end(t + ms(95)));
        a.tick(t + ms(150));
        let at = a.offset();
        a.precise(0.0, t + ms(151));
        assert!(!a.is_animating());
        assert!(!a.tick(t + ms(300)));
        assert_eq!(a.offset(), at);
    }

    #[test]
    fn fling_curve_constants_match_chromium() {
        assert!((fling_max_velocity() - 20_946.194).abs() < 0.01);
        let end = fling_time_at_velocity(0.0);
        assert!((end - 1.2999).abs() < 0.001, "{end}");
    }

    #[test]
    fn shrinking_content_clamps() {
        let t = Instant::now();
        let mut a = axis(1000.0);
        a.set_offset(900.0);
        a.set_max(300.0);
        assert_eq!(a.offset(), 300.0);
        a.wheel_notches(-1.0, t);
        assert_eq!(a.target(), 180.0);
    }

    #[test]
    fn hi_res_wheel_stream_moves_smoothly() {
        // A free-spinning wheel: 1/8 notch every 8 ms.
        let t = Instant::now();
        let mut a = axis(100_000.0);
        let mut last = 0.0;
        let mut max_step: f32 = 0.0;
        for i in 0..60 {
            let now = t + ms(i * 8);
            a.wheel_notches(0.125, now);
            a.tick(now);
            max_step = max_step.max(a.offset() - last);
            assert!(a.offset() >= last, "monotonic");
            last = a.offset();
        }
        // 15 px per 8 ms is ~1875 px/s; no single frame should leap far past that.
        assert!(max_step < 40.0, "max step {max_step}");
    }
}
