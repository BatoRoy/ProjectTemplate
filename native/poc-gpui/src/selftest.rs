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

fn touch_ev(id: u64, phase: TouchPhase, x: f32, y: f32) -> PlatformInput {
    PlatformInput::Touch(gpui::TouchEvent { id: gpui::TouchId(id), phase, position: point(px(x), px(y)), ..Default::default() })
}

fn check(name: &str, ok: bool, detail: String) -> bool {
    eprintln!("selftest {} {name} ({detail})", if ok { "PASS" } else { "FAIL" });
    ok
}

/// `BATO_SELFTEST=touch`: raw touch events through GPUI's dispatch (the same
/// `PlatformInput::Touch` the patched Wayland client emits): taps on the
/// switch and a nav row, a drag + fling on Scroll lab, a touch that catches the
/// fling, a sidebar drag, a cancel, and a second finger. Prints PASS/FAIL.
pub async fn run_touch(root: WeakEntity<Root>, cx: &mut AsyncWindowContext) {
    let ex = cx.background_executor().clone();
    let sleep = |ms| ex.timer(Duration::from_millis(ms));
    let send = |cx: &mut AsyncWindowContext, ev: PlatformInput| {
        cx.update(|window, cx| {
            window.dispatch_event(ev, cx);
        })
        .ok();
    };
    let get = |cx: &mut AsyncWindowContext| {
        root.update(cx, |r, _| (r.auto_update, r.view, r.main.axis.offset(), r.main.axis.is_animating())).unwrap()
    };
    let mut all = true;
    sleep(800).await;

    // 1. Tap the "Update automatically" switch (Home).
    let before = get(cx).0;
    send(cx, touch_ev(0, TouchPhase::Started, 1129., 644.));
    send(cx, touch_ev(0, TouchPhase::Moved, 1131., 646.));
    send(cx, touch_ev(0, TouchPhase::Ended, 1131., 646.));
    sleep(100).await;
    let after = get(cx).0;
    all &= check("tap toggles switch", after != before, format!("{before} -> {after}"));

    // 2. A drag inside the slop on a nav row is still a tap.
    send(cx, touch_ev(0, TouchPhase::Started, 78., 205.));
    send(cx, touch_ev(0, TouchPhase::Moved, 86., 214.));
    send(cx, touch_ev(0, TouchPhase::Ended, 86., 214.));
    sleep(300).await;
    let v = get(cx).1;
    all &= check("tap on nav row navigates", v == crate::app::View::ScrollLab, format!("{v:?}"));

    // 3. Drag 300 px up over 100 ms: content follows 1:1 minus the 15 px slop, then flings.
    send(cx, touch_ev(0, TouchPhase::Started, 700., 650.));
    for i in 1..=10 {
        sleep(10).await;
        send(cx, touch_ev(0, TouchPhase::Moved, 700., 650. - 30. * i as f32));
    }
    let off = get(cx).2;
    all &= check("drag tracks finger (300 - 15 slop)", (off - 285.).abs() < 0.5, format!("offset {off:.2}"));
    // Second finger while the first is down: ignored.
    send(cx, touch_ev(1, TouchPhase::Started, 900., 300.));
    send(cx, touch_ev(1, TouchPhase::Moved, 900., 100.));
    send(cx, touch_ev(1, TouchPhase::Ended, 900., 100.));
    let off2 = get(cx).2;
    all &= check("second finger ignored", (off2 - off).abs() < 0.01, format!("offset {off2:.2}"));
    send(cx, touch_ev(0, TouchPhase::Ended, 700., 350.));
    sleep(300).await;
    let (_, _, off3, anim) = get(cx);
    all &= check("lift flings", anim && off3 > off + 50., format!("offset {off3:.1}, animating {anim}"));

    // 4. Touch-down catches the fling; the lift does not click.
    let view = get(cx).1;
    send(cx, touch_ev(0, TouchPhase::Started, 700., 400.));
    let (_, _, caught, anim) = get(cx);
    send(cx, touch_ev(0, TouchPhase::Ended, 700., 400.));
    sleep(100).await;
    let (_, view2, off4, _) = get(cx);
    all &= check("touch-down stops fling, no click", !anim && (off4 - caught).abs() < 0.01 && view2 == view, format!("offset {caught:.1} -> {off4:.1}"));

    // 5. Dragging the sidebar never scrolls the content.
    send(cx, touch_ev(0, TouchPhase::Started, 100., 600.));
    send(cx, touch_ev(0, TouchPhase::Moved, 100., 400.));
    send(cx, touch_ev(0, TouchPhase::Ended, 100., 400.));
    sleep(50).await;
    let off5 = get(cx).2;
    all &= check("sidebar drag does not scroll", (off5 - off4).abs() < 0.01, format!("offset {off5:.1}"));

    // 6. wl_touch.cancel mid-drag: no fling.
    send(cx, touch_ev(0, TouchPhase::Started, 700., 600.));
    for i in 1..=5 {
        sleep(10).await;
        send(cx, touch_ev(0, TouchPhase::Moved, 700., 600. - 40. * i as f32));
    }
    send(cx, touch_ev(0, TouchPhase::Cancelled, 700., 400.));
    sleep(100).await;
    let (_, _, off6, anim) = get(cx);
    all &= check("cancel: no fling", !anim, format!("offset {off6:.1}"));

    eprintln!("selftest-touch summary: {}", if all { "ALL PASS" } else { "FAILURES" });
    cx.update(|_, cx| cx.quit()).ok();
}

/// `BATO_SELFTEST=autoscroll`: middle press/release/move through GPUI's
/// dispatch on Scroll lab; checks toggle mode, scrolling, the swallowed
/// stopping click, wheel cancel, sidebar ineligibility, then idleness.
pub async fn run_autoscroll(root: WeakEntity<Root>, cx: &mut AsyncWindowContext) {
    let ex = cx.background_executor().clone();
    let sleep = |ms| ex.timer(Duration::from_millis(ms));
    let m = Modifiers::default();
    let send = |cx: &mut AsyncWindowContext, ev: PlatformInput| {
        cx.update(|window, cx| {
            window.dispatch_event(ev, cx);
        })
        .ok();
    };
    let down = |b, x: f32, y: f32| PlatformInput::MouseDown(gpui::MouseDownEvent { button: b, position: point(px(x), px(y)), modifiers: m, click_count: 1, first_mouse: false });
    let up = |b, x: f32, y: f32| PlatformInput::MouseUp(gpui::MouseUpEvent { button: b, position: point(px(x), px(y)), modifiers: m, click_count: 1 });
    let mv = |x: f32, y: f32| PlatformInput::MouseMove(gpui::MouseMoveEvent { position: point(px(x), px(y)), pressed_button: None, modifiers: m });
    let get = |cx: &mut AsyncWindowContext| root.update(cx, |r, _| (r.auto.engaged(), r.view, r.main.axis.offset())).unwrap();
    let mut all = true;
    sleep(800).await;

    send(cx, down(gpui::MouseButton::Middle, 640., 400.));
    send(cx, up(gpui::MouseButton::Middle, 640., 400.));
    let (eng, _, o0) = get(cx);
    all &= check("middle click engages (toggle mode)", eng, String::new());
    send(cx, mv(640., 700.));
    sleep(500).await;
    let (eng, _, o1) = get(cx);
    all &= check("cursor below origin scrolls down", eng && o1 > o0 + 300., format!("{o0:.0} -> {o1:.0}"));
    // A left click on the sidebar "Home" row stops it and never lands.
    send(cx, down(gpui::MouseButton::Left, 78., 125.));
    send(cx, up(gpui::MouseButton::Left, 78., 125.));
    sleep(50).await;
    let (eng, view, _) = get(cx);
    all &= check("click stops it and is swallowed", !eng && view == crate::app::View::ScrollLab, format!("{view:?}"));

    // Wheel cancels and is consumed.
    send(cx, down(gpui::MouseButton::Middle, 640., 400.));
    send(cx, up(gpui::MouseButton::Middle, 640., 400.));
    let o2 = get(cx).2;
    send(cx, scroll(ScrollDelta::Lines(point(0., -3.)), TouchPhase::Moved));
    sleep(300).await;
    let (eng, _, o3) = get(cx);
    all &= check("wheel stops it and is consumed", !eng && (o3 - o2).abs() < 1.0, format!("{o2:.0} -> {o3:.0}"));

    // Hold mode: drag beyond the threshold, release ends it.
    send(cx, down(gpui::MouseButton::Middle, 640., 400.));
    send(cx, mv(640., 380.));
    send(cx, up(gpui::MouseButton::Middle, 640., 380.));
    all &= check("hold-to-scroll ends on release", !get(cx).0, String::new());

    // Sidebar: not scrollable, nothing happens.
    send(cx, down(gpui::MouseButton::Middle, 100., 400.));
    send(cx, up(gpui::MouseButton::Middle, 100., 400.));
    all &= check("middle on sidebar does nothing", !get(cx).0, String::new());

    sleep(300).await;
    root.update(cx, |r, _| r.selftest_frames = Some(Vec::new())).ok();
    sleep(1000).await;
    let after = root.update(cx, |r, _| r.selftest_frames.as_ref().map(|f| f.len()).unwrap_or(0)).unwrap_or(0);
    all &= check("idle afterwards", after == 0, format!("{after} renders in 1 s"));
    eprintln!("selftest-autoscroll summary: {}", if all { "ALL PASS" } else { "FAILURES" });
    cx.update(|_, cx| cx.quit()).ok();
}
