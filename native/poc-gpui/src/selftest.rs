//! `BATO_SELFTEST=1`: drives synthetic input through GPUI's real event dispatch
//! (`Window::dispatch_event`, i.e. hit-testing → our listeners → physics → the
//! frame loop), logs every frame, and checks that the app goes idle afterwards.
//! It never touches the desktop session: events are injected in-process.
//!
//! Sequence: 3 wheel notches 40 ms apart, a 100 ms touchpad swipe + lift
//! (fling), a click on the scrollbar track, the sidebar collapse toggle, then
//! 1.5 s of idle. Output: `selftest frame …` lines and a summary, then exit.

use std::time::{Duration, Instant};

use gpui::{point, px, AsyncWindowContext, Modifiers, PlatformInput, ScrollDelta, ScrollEvent, ScrollSource, TouchPhase, WeakEntity};

use crate::app::Root;

fn scroll(delta: ScrollDelta, phase: TouchPhase) -> PlatformInput {
    PlatformInput::Scroll(ScrollEvent {
        position: point(px(700.), px(400.)),
        delta,
        modifiers: Modifiers::default(),
        touch_phase: phase,
        source: ScrollSource::Pointer,
    })
}

pub async fn run(root: WeakEntity<Root>, cx: &mut AsyncWindowContext) {
    let t0 = Instant::now();
    let ex = cx.background_executor().clone();
    let sleep = |ms| ex.timer(Duration::from_millis(ms));
    sleep(800).await;
    let log = |cx: &mut AsyncWindowContext, what: &str| {
        let what = what.to_string();
        root.update(cx, |r, _| {
            eprintln!("selftest event t={:>5}ms {what} offset={:.1}", t0.elapsed().as_millis(), r.main.axis.offset());
        })
        .ok();
    };
    let dispatch = |cx: &mut AsyncWindowContext, ev: PlatformInput| {
        cx.update(|window, cx| {
            window.dispatch_event(ev, cx);
        })
        .ok();
    };
    root.update(cx, |r, _| r.selftest_frames = Some(Vec::new())).ok();

    // Wheel: 3 notches (Wayland value120=120 each → GPUI Lines(-3)).
    for _ in 0..3 {
        log(cx, "wheel notch");
        dispatch(cx, scroll(ScrollDelta::Lines(point(0., -3.)), TouchPhase::Moved));
        sleep(40).await;
    }
    sleep(600).await;

    // Touchpad: 10 × wl axis 2.0 every 10 ms (GPUI Pixels(-6)), then axis_stop.
    log(cx, "touchpad swipe start");
    for _ in 0..10 {
        dispatch(cx, scroll(ScrollDelta::Pixels(point(px(0.), px(-6.))), TouchPhase::Moved));
        sleep(10).await;
    }
    log(cx, "touchpad lift (axis_stop)");
    dispatch(cx, scroll(ScrollDelta::Pixels(point(px(0.), px(0.))), TouchPhase::Ended));
    sleep(1600).await;

    log(cx, "sidebar toggle");
    root.update(cx, |r, cx| r.toggle_sidebar(cx)).ok();
    sleep(400).await;
    root.update(cx, |r, cx| r.toggle_sidebar(cx)).ok();
    sleep(400).await;

    log(cx, "idle window start");
    let before = root.update(cx, |r, _| r.selftest_frames.as_ref().map(|f| f.len()).unwrap_or(0)).unwrap_or(0);
    sleep(1500).await;
    let after = root.update(cx, |r, _| r.selftest_frames.as_ref().map(|f| f.len()).unwrap_or(0)).unwrap_or(0);

    root.update(cx, |r, cx| {
        let frames = r.selftest_frames.take().unwrap_or_default();
        let mut prev: Option<(f64, f32)> = None;
        let mut gaps = Vec::new();
        for (t, off, w) in &frames {
            let line = match prev {
                Some((pt, _)) => {
                    gaps.push(t - pt);
                    format!("dt={:>6.2}ms", t - pt)
                }
                None => "dt=     -".into(),
            };
            eprintln!("selftest frame t={t:>8.2}ms {line} offset={off:>8.2} sidebar={w:.1}");
            prev = Some((*t, *off));
        }
        gaps.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let med = gaps.get(gaps.len() / 2).copied().unwrap_or(0.0);
        eprintln!(
            "selftest summary: frames={} median_frame_interval={med:.2}ms renders_during_1.5s_idle={} final_offset={:.1}",
            frames.len(),
            after - before,
            r.main.axis.offset()
        );
        cx.quit();
    })
    .ok();
}

fn click(cx: &mut AsyncWindowContext, x: f32, y: f32) {
    let position = point(px(x), px(y));
    cx.update(|window, cx| {
        window.dispatch_event(PlatformInput::MouseMove(gpui::MouseMoveEvent { position, pressed_button: None, modifiers: Modifiers::default() }), cx);
        window.dispatch_event(
            PlatformInput::MouseDown(gpui::MouseDownEvent {
                button: gpui::MouseButton::Left,
                position,
                modifiers: Modifiers::default(),
                click_count: 1,
                first_mouse: false,
            }),
            cx,
        );
        window.dispatch_event(
            PlatformInput::MouseUp(gpui::MouseUpEvent { button: gpui::MouseButton::Left, position, modifiers: Modifiers::default(), click_count: 1 }),
            cx,
        );
    })
    .ok();
}

/// `BATO_SELFTEST=ui`: synthetic clicks at 100% scale on Home: open App
/// Options, pick Light, close with Escape, put a name in the input, click
/// Primary (toast "Hello, Bato!"), click Ping. Stays open for a screenshot.
pub async fn run_ui(root: WeakEntity<Root>, cx: &mut AsyncWindowContext) {
    let ex = cx.background_executor().clone();
    let sleep = |ms| ex.timer(Duration::from_millis(ms));
    let state = |cx: &mut AsyncWindowContext, what: &str| {
        let what = what.to_string();
        root.update(cx, |r, _| eprintln!("selftest-ui {what}: modal={:?} theme={:?} toasts={}", r.modal, r.settings.theme, r.toasts.len())).ok();
    };
    sleep(800).await;
    click(cx, 88., 729.);
    sleep(300).await;
    state(cx, "after App Options click");
    click(cx, 843., 297.);
    sleep(300).await;
    state(cx, "after Light click");
    cx.update(|window, cx| {
        window.dispatch_event(
            PlatformInput::KeyDown(gpui::KeyDownEvent { keystroke: gpui::Keystroke::parse("escape").unwrap(), is_held: false, prefer_character_input: false }),
            cx,
        );
    })
    .ok();
    sleep(300).await;
    state(cx, "after Escape");
    root.update(cx, |r, cx| r.name_input.update(cx, |i, cx| i.set_text("Bato", cx))).ok();
    click(cx, 397., 513.);
    sleep(300).await;
    state(cx, "after Primary click");
}

/// `BATO_SELFTEST=visit`: go to Scroll lab, wheel through ~2,400 px of rows,
/// come back to Home and stay (for the "after visiting Scroll lab" memory
/// measurement).
pub async fn run_visit(root: WeakEntity<Root>, cx: &mut AsyncWindowContext) {
    let ex = cx.background_executor().clone();
    let sleep = |ms| ex.timer(Duration::from_millis(ms));
    sleep(500).await;
    root.update(cx, |r, cx| r.navigate(crate::app::View::ScrollLab, cx)).ok();
    sleep(300).await;
    for _ in 0..20 {
        cx.update(|window, cx| {
            window.dispatch_event(scroll(ScrollDelta::Lines(point(0., -3.)), TouchPhase::Moved), cx);
        })
        .ok();
        sleep(60).await;
    }
    sleep(800).await;
    root.update(cx, |r, cx| r.navigate(crate::app::View::Home, cx)).ok();
    eprintln!("selftest-visit done");
}

/// `BATO_SELFTEST=rail`: collapse the sidebar, hover the Scroll lab row and
/// stay, so a screenshot shows the RailTip.
pub async fn run_rail(root: WeakEntity<Root>, cx: &mut AsyncWindowContext) {
    let ex = cx.background_executor().clone();
    let sleep = |ms| ex.timer(Duration::from_millis(ms));
    sleep(500).await;
    root.update(cx, |r, cx| {
        if !r.collapsed {
            r.toggle_sidebar(cx)
        }
    })
    .ok();
    sleep(400).await;
    cx.update(|window, cx| {
        let position = point(px(32.), px(205.));
        window.dispatch_event(PlatformInput::MouseMove(gpui::MouseMoveEvent { position, pressed_button: None, modifiers: Modifiers::default() }), cx);
    })
    .ok();
}
