pub mod blur;
pub mod grid;
pub mod icon;
pub mod label;
pub mod lablist;
pub mod modal;
pub mod scroller;
pub mod surface;
pub mod switch;
pub mod input;
pub mod timer;

use std::sync::atomic::{AtomicU32, Ordering};

static SCALE: AtomicU32 = AtomicU32::new(0x3f80_0000); // 1.0

/// Logical → device px factor (window scale factor × UI scale). Widgets need
/// it to snap text baselines and icons to device pixels the way Chromium does.
pub fn device_scale() -> f32 {
    f32::from_bits(SCALE.load(Ordering::Relaxed))
}

pub fn set_device_scale(s: f32) {
    SCALE.store(s.to_bits(), Ordering::Relaxed);
}

pub fn snap_rect(r: iced::Rectangle) -> iced::Rectangle {
    let s = device_scale();
    let x0 = (r.x * s).round();
    let y0 = (r.y * s).round();
    let x1 = ((r.x + r.width) * s).round();
    let y1 = ((r.y + r.height) * s).round();
    iced::Rectangle { x: x0 / s, y: y0 / s, width: (x1 - x0) / s, height: (y1 - y0) / s }
}
