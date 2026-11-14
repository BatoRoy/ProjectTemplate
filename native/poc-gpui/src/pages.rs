//! Home (§4.2), Examples placeholder, Scroll lab (ScrollLabPage.tsx),
//! App Options (§4.4/4.5) and About (§4.7).

use bato_theme::{Rgb, ThemeId, ACCENT_PRESETS, SCALES};
use gpui::{canvas, div, img, prelude::*, AnyElement, Context, Div, Focusable, HighlightStyle, StyledText, Window};

use crate::app::{Root, Status, TestState, ToastKind};
use crate::theme::{c, ink_for, u, Theme, APP_NAME, BRAND_ACCENT, MONO, TAGLINE, VERSION};
use crate::tracked_text::tracked;
use crate::ui::{self, badge, button, card, icon, segmented, Size, Tone, Tween, Variant, W_MEDIUM, W_SEMIBOLD};

pub const LAB_ROWS: usize = 1000;
const WORDS: [&str; 16] = [
    "Aurora", "Basalt", "Cobalt", "Driftwood", "Ember", "Fjord", "Granite", "Harbor", "Indigo", "Juniper", "Kestrel", "Lagoon",
    "Meridian", "Nimbus", "Obsidian", "Prairie",
];
const SENTENCES: [&str; 7] = [
    "Synced 14 files from the archive without conflicts.",
    "Waiting for the backend to acknowledge the request.",
    "Thumbnail cache rebuilt; 3 entries were stale.",
    "Quick brown foxes jump over lazy dogs — 0123456789.",
    "Scheduled for tomorrow at 09:30, repeats weekly.",
    "Imported 1,284 rows, skipped 7 duplicates.",
    "Connection restored after 2 retries (412 ms).",
];
const LAB_P0: &str = "Typography check. The quick brown fox jumps over the lazy dog. Sphinx of black quartz, judge my vow. How vexingly quick daft zebras jump! Pack my box with five dozen liquor jugs.";
const LAB_P1: &str = "Numbers and punctuation: 1,234,567.89 — 3/4 · 42% · 09:41 · (parentheses) [brackets] {braces} “quotes” ‘single’ … Ligatures and kerning: fi fl ffi AV To Wa Yo LT.";

/// Text runs for `len` bytes with `mono` in JetBrains Mono, the rest in the
/// inherited font. (StyledText's family overrides only apply to whole runs.)
fn mono_runs(window: &Window, len: usize, mono: std::ops::Range<usize>, size: f32, color: gpui::Hsla) -> Vec<gpui::TextRun> {
    let mut st = window.text_style();
    st.color = color;
    st.font_family = crate::theme::SANS.into();
    st.font_weight = gpui::FontWeight::NORMAL;
    let mut runs = Vec::new();
    if mono.start > 0 {
        runs.push(st.to_run(mono.start));
    }
    let mut m = st.to_run(mono.len());
    // At 13px use the 12-in-13 face: the inline word is 12px in CSS.
    m.font.family = if size == 13.0 { "Bato JetBrains Mono 12in13".into() } else { MONO.into() };
    runs.push(m);
    if mono.end < len {
        runs.push(st.to_run(len - mono.end));
    }
    runs
}

/// `mx-auto px-8 py-12 max-w-[content-width]` page wrapper.
fn page(wide: bool) -> Div {
    div()
        .w_full()
        .max_w(if wide { u(100_000.) } else { u(896.) })
        .px(u(32.))
        .py(u(48.))
        .flex()
        .flex_col()
}

fn centered(child: Div) -> Div {
    div().w_full().flex_none().flex().justify_center().child(child)
}

impl Root {
    // ─── Home ───────────────────────────────────────────────────────────────

    pub fn render_home(&mut self, t: &Theme, window: &mut Window, cx: &mut Context<Self>) -> Div {
        let p = &t.p;
        let on_card = p.card;

        let cmd_bg = t.wash(0.04, p.bg);
        let header = div()
            .flex()
            .items_center()
            .gap(u(20.))
            .pb(u(40.))
            .child(ui::app_mark(t, 56.))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .child(div().text_size(u(30.)).line_height(u(36.)).font_weight(W_SEMIBOLD).child(tracked(APP_NAME, -0.025, c(p.text))))
                    .child(ui::text(TAGLINE, 14., 20., c(p.subtext)).mt(u(4.))),
            )
            .child(
                button(t, "commands", Variant::Secondary, Size::Sm, p.bg, false)
                    .child(icon("command", 13., c(p.text)))
                    .child("Commands")
                    .child(ui::kbd(t, "Ctrl K", cmd_bg)),
            );

        // Backend card
        let (tone, label) = match self.status {
            Status::Online => (Tone::Success, "Online"),
            Status::Offline => (Tone::Error, "Offline"),
            Status::Checking => (Tone::Warning, "Checking…"),
        };
        let ping_icon = icon("refresh-cw", 13., c(p.text));
        let ping_icon = if self.status == Status::Checking { ui::spinning(ping_icon, self.status_since) } else { ping_icon };
        // `self-center` 16px mono line in a 19.5px row ≡ a 19.5px mono line box
        // (same baseline); avoids an explicit 19.5px height, which GPUI would
        // snap to 20px.
        let dd = |s: String| {
            div().min_w_0().font_family(MONO).text_size(u(12.)).line_height(u(19.5)).text_color(c(p.text)).truncate().child(s)
        };
        let backend_card = card(t, Some("Backend"), Some("server"), Some(badge(t, tone, true, on_card, label).into_any_element()))
            .flex_1()
            .min_w_0()
            .child(
                div()
                    .flex()
                    .gap(u(16.))
                    .text_size(u(13.))
                    .line_height(u(19.5))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(u(6.))
                            .text_color(c(p.muted))
                            .child("Server")
                            .child("Version"),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .gap(u(6.))
                            .child(dd(self.backend.clone()))
                            .child(dd(self.backend_version.clone().unwrap_or_else(|| "—".into()))),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(u(8.))
                    .mt(u(20.))
                    .child(
                        div()
                            .text_size(u(12.))
                            .line_height(u(16.))
                            .text_color(c(p.muted))
                            .child(StyledText::new("Run make dev-server").with_runs(mono_runs(window, 19, 4..19, 12., c(p.muted)))),
                    )
                    .child(
                        button(t, "ping", Variant::Secondary, Size::Sm, on_card, false)
                            .child(ping_icon)
                            .child("Ping")
                            .on_click(cx.listener(|this, _, _, cx| this.ping(cx))),
                    ),
            );

        let para = "A native OS notification from the Electron main process. Falls back to the web Notification API in a plain browser.";
        let ns = para.find("Notification API").unwrap();
        let styled = StyledText::new(para).with_runs(mono_runs(window, para.len(), ns..ns + "Notification".len(), 13., c(p.subtext)));
        let notif_card = card(t, Some("Notifications"), Some("bell"), None)
            .flex_1()
            .min_w_0()
            .child(div().text_size(u(13.)).line_height(u(21.125)).text_color(c(p.subtext)).child(styled))
            .child(
                div().flex().justify_end().mt(u(20.)).child(
                    button(t, "notify", Variant::Secondary, Size::Sm, on_card, false)
                        .child(icon("bell", 13., c(p.text)))
                        .child("Notify")
                        .on_click(cx.listener(|this, _, _, cx| this.notify_os(cx))),
                ),
            );

        // Components card
        let mk = |id: &'static str, v: Variant, label: &'static str, msg: &'static str, kind: ToastKind| {
            button(t, id, v, Size::Md, on_card, false).child(label).on_click(cx.listener(move |this, _, _, cx| {
                let name = this.name_input.read(cx).content.to_string();
                let m = if id == "b-primary" {
                    if name.is_empty() { "Success toast".to_string() } else { format!("Hello, {name}!") }
                } else {
                    msg.to_string()
                };
                this.toast(m, kind, cx);
            }))
        };
        let au_pos = match &self.auto_update_tween {
            Some(tw) => tw.at(ui::ease_std).0,
            None => if self.auto_update { 1.0 } else { 0.0 },
        };
        let foot = "Open App Options in the sidebar to switch theme, accent and scale. The Examples page has the full component kit.";
        let hl = |word: &str| {
            let i = foot.find(word).unwrap();
            (i..i + word.len(), HighlightStyle { color: Some(c(p.subtext)), font_weight: Some(W_MEDIUM), ..Default::default() })
        };
        let foot_text = StyledText::new(foot).with_highlights([hl("App Options"), hl("Examples")]);
        let components = card(
            t,
            Some("Components"),
            Some("blocks"),
            Some(badge(t, Tone::Info, false, on_card, div().font_family(MONO).child(format!("v{VERSION}"))).into_any_element()),
        )
        .mt(u(16.))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(u(24.))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(u(6.))
                        .child(ui::text("Your name", 12., 16., c(p.subtext)).font_weight(W_MEDIUM))
                        .child(self.name_input.clone()),
                )
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap(u(8.))
                        .child(mk("b-primary", Variant::Primary, "Primary", "", ToastKind::Success))
                        .child(mk("b-secondary", Variant::Secondary, "Secondary", "Just so you know", ToastKind::Info))
                        .child(mk("b-ghost", Variant::Ghost, "Ghost", "Quiet action", ToastKind::Info))
                        .child(mk("b-success", Variant::Success, "Success", "All good", ToastKind::Success))
                        .child(mk("b-danger", Variant::Danger, "Danger", "Something went wrong", ToastKind::Error)),
                )
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap(u(8.))
                        .child(badge(t, Tone::Neutral, false, on_card, "Neutral"))
                        .child(badge(t, Tone::Info, false, on_card, "Info"))
                        .child(badge(t, Tone::Success, false, on_card, "Success"))
                        .child(badge(t, Tone::Warning, false, on_card, "Warning"))
                        .child(badge(t, Tone::Error, false, on_card, "Error")),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .gap(u(24.))
                        .pt(u(20.))
                        .border_t_1()
                        .border_color(c(t.line_over(on_card)))
                        .child(
                            div()
                                .child(ui::text("Update automatically", 13., 19.5, c(p.text)).font_weight(W_MEDIUM))
                                .child(ui::text("Download new versions in the background.", 12., 16., c(p.muted)).mt(u(2.))),
                        )
                        .child(ui::switch(t, "auto-update", au_pos).on_click(cx.listener(|this, _, _, cx| {
                            this.auto_update = !this.auto_update;
                            let (from, to) = if this.auto_update { (0.0, 1.0) } else { (1.0, 0.0) };
                            this.auto_update_tween = Some(Tween::new(from, to, 200));
                            cx.notify();
                        }))),
                )
                .child(div().text_size(u(12.)).line_height(u(16.)).text_color(c(p.muted)).child(foot_text)),
        );

        centered(
            page(self.settings.wide)
                .child(header)
                .child(div().flex().gap(u(16.)).child(backend_card).child(notif_card))
                .child(components),
        )
    }

    // ─── Examples ───────────────────────────────────────────────────────────

    pub fn render_examples(&mut self, t: &Theme, _cx: &mut Context<Self>) -> Div {
        let p = &t.p;
        centered(
            page(self.settings.wide)
                .py(u(24.))
                .px(u(24.))
                .child(ui::text("Component kit", 20., 28., c(p.text)).font_weight(W_SEMIBOLD))
                .child(ui::text("Coming in a later phase.", 14., 20., c(p.subtext)).mt(u(4.))),
        )
    }

    // ─── Scroll lab ─────────────────────────────────────────────────────────

    pub fn render_scroll_lab(&mut self, t: &Theme, _cx: &mut Context<Self>) -> Div {
        let p = &t.p;
        let s = t.s;
        let header = div()
            .pb(u(32.))
            .child(div().text_size(u(30.)).line_height(u(36.)).font_weight(W_SEMIBOLD).child(tracked("Scroll lab", -0.025, c(p.text))))
            .child(ui::text("1,000 rows · wheel, touchpad and fling · text at 16 / 14 / 13 / 12 px", 14., 20., c(p.subtext)).mt(u(4.)));
        let text_card = card(t, Some("Text rendering"), None, None).mb(u(24.)).child(
            div()
                .flex()
                .flex_col()
                .gap(u(12.))
                .child(ui::text(LAB_P0, 16., 24., c(p.text)))
                .child(ui::text(LAB_P0, 14., 20., c(p.text)))
                .child(ui::text(LAB_P1, 13., 19.5, c(p.subtext)))
                .child(ui::text(LAB_P1, 12., 16., c(p.muted)))
                .child(ui::text("0123456789 · const answer = 42; // JetBrains Mono 12px", 12., 16., c(p.subtext)).font_family(MONO)),
        );

        // Virtualized rows: only the rows intersecting the viewport (± 3) exist.
        let row_h = 56.0 * s;
        let offset = self.main.axis.offset();
        let viewport = self.main.viewport_h.max(800.0);
        let (first, last) = match self.lab_section_top.get() {
            Some(top) => {
                let rel = offset - top - 1.0;
                let first = ((rel / row_h).floor() as i64 - 3).clamp(0, LAB_ROWS as i64) as usize;
                let last = (((rel + viewport) / row_h).ceil() as i64 + 3).clamp(0, LAB_ROWS as i64) as usize;
                (first, last.max(first))
            }
            None => (0, 20),
        };
        let line = t.line_over(p.card);
        let hover = t.wash(0.04, p.card);
        let chip_bg = bato_theme::blend(p.accent, 0.15, p.card);
        let mut rows = div().relative().w_full().h(u(56. * LAB_ROWS as f32));
        for i in first..last {
            let n = i + 1;
            rows = rows.child(
                div()
                    .id(("lab-row", i))
                    .absolute()
                    .left_0()
                    .right_0()
                    .top(u(56. * i as f32))
                    .h(u(56.))
                    .flex()
                    .items_center()
                    .gap(u(12.))
                    .px(u(16.))
                    .when(i + 1 < LAB_ROWS, |d| d.border_b_1().border_color(c(line)))
                    .hover(move |s| s.bg(c(hover)))
                    .child(
                        div()
                            .size(u(32.))
                            .flex_none()
                            .rounded(u(8.))
                            .bg(c(chip_bg))
                            .flex()
                            .items_center()
                            .justify_center()
                            .font_family(MONO)
                            .text_size(u(11.))
                            .line_height(u(16.5))
                            .font_weight(W_MEDIUM)
                            .text_color(c(p.accent_bright))
                            .child(format!("{}", n % 100)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .child(ui::text(format!("Row {n} — {}", WORDS[i % WORDS.len()]), 14., 20., c(p.text)).font_weight(W_MEDIUM).truncate())
                            .child(ui::text(SENTENCES[i % SENTENCES.len()], 12., 16., c(p.muted)).truncate()),
                    )
                    .child(ui::text(format!("{} ms", (i * 37) % 1000), 12., 16., c(p.subtext)).font_family(MONO)),
            );
        }
        // Measure the section's content-space top for the visible-range math.
        let cell = self.lab_section_top.clone();
        let handle = self.main.handle.clone();
        let probe = canvas(
            move |bounds, _, _| {
                let vp_top = f32::from(handle.bounds().top());
                cell.set(Some(f32::from(bounds.top()) - vp_top + offset));
            },
            |_, _, _, _| {},
        )
        .absolute()
        .top_0()
        .left_0()
        .size_0();
        let section = div()
            .relative()
            .bg(c(p.card))
            .border_1()
            .border_color(c(line))
            .rounded(u(16.))
            .shadow(t.shadow_md())
            .overflow_hidden()
            .child(probe)
            .child(rows);

        centered(page(self.settings.wide).child(header).child(text_card).child(section))
    }

    // ─── App Options ────────────────────────────────────────────────────────

    pub fn render_options(&mut self, t: &Theme, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let p = &t.p;
        let line = t.line_over(p.card);
        let accent = t.accent_raw.clone();

        let mut swatches: Vec<(String, bool)> = vec![(BRAND_ACCENT.to_string(), true)];
        for (_, hex) in ACCENT_PRESETS.iter() {
            if !hex.eq_ignore_ascii_case(BRAND_ACCENT) {
                swatches.push((hex.to_string(), false));
            }
        }
        let active_ring = bato_theme::blend(p.text, 0.8, p.card);
        let mut accent_row = div().flex().flex_wrap().items_center().gap(u(10.));
        for (i, (hex, is_default)) in swatches.into_iter().enumerate() {
            let rgb = Rgb::hex(&hex).unwrap();
            let ink = ink_for(rgb);
            let active = hex.eq_ignore_ascii_case(&accent);
            let hex2 = hex.clone();
            let s = t.s;
            accent_row = accent_row.child(
                div()
                    .id(("swatch", i))
                    .size(u(28.))
                    .flex_none()
                    .rounded_full()
                    .bg(c(rgb))
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .when(active, |d| d.shadow(vec![ui::ring(c(active_ring), 4.0 * s), ui::ring(c(p.card), 2.0 * s)]))
                    .hover(|st| st.size(u(30.8)).m(u(-1.4)))
                    .when(active, |d| d.child(icon("check", 14., c(ink))))
                    .when(!active && is_default, |d| d.child(icon("star-filled", 11., c(ink)).opacity(0.9)))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        let h = hex2.clone();
                        this.update_settings(cx, |s| s.accent = Some(h));
                    })),
            );
        }
        let line_c = c(t.line_over(p.card));
        accent_row = accent_row
            .child(
                div()
                    .id("swatch-custom")
                    .relative()
                    .size(u(28.))
                    .flex_none()
                    .cursor_pointer()
                    .hover(|st| st.size(u(30.8)).m(u(-1.4)))
                    .child(img("icons/rainbow.svg").size_full())
                    .child(div().absolute().inset_0().rounded_full().shadow(vec![t.ring_inset(line_c)]))
                    .on_click(cx.listener(|this, _, w, cx| {
                        this.hex_open = !this.hex_open;
                        if this.hex_open {
                            let cur = cx.global::<Theme>().accent_raw.clone();
                            this.hex_input.update(cx, |i, cx| i.set_text(&cur, cx));
                            let fh = this.hex_input.read(cx).focus_handle(cx);
                            w.focus(&fh, cx);
                        }
                        cx.notify();
                    })),
            )
            .child(div().ml_auto().font_family(MONO).text_size(u(11.)).line_height(u(16.5)).text_color(c(p.muted)).child(accent.clone()));

        let accent_block = div()
            .pb(u(20.))
            .border_b_1()
            .border_color(c(line))
            .child(accent_row)
            .when(self.hex_open, |d| {
                d.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(u(8.))
                        .mt(u(12.))
                        .child(ui::text("Custom hex", 12., 16., c(p.subtext)).font_weight(W_MEDIUM))
                        .child(div().w(u(120.)).child(self.hex_input.clone())),
                )
            });

        let row = |label: &str, desc: Option<&str>, control: AnyElement, first: bool| {
            div()
                .flex()
                .items_center()
                .justify_between()
                .gap(u(24.))
                .py(u(14.))
                .when(!first, |d| d.border_t_1().border_color(c(line)))
                .child(
                    div()
                        .min_w_0()
                        .child(ui::text(label.to_string(), 13., 19.5, c(p.text)).font_weight(W_MEDIUM))
                        .when_some(desc, |d, desc| d.child(ui::text(desc.to_string(), 12., 16., c(p.muted)).mt(u(2.)))),
                )
                .child(control)
        };
        let ent = cx.entity();
        let theme_seg = {
            let e = ent.clone();
            segmented(
                t,
                "seg-theme",
                &[(ThemeId::Dark, "Dark"), (ThemeId::Dim, "Dim"), (ThemeId::Light, "Light")],
                &self.settings.theme,
                move |v, _, cx| {
                    let v = *v;
                    e.update(cx, |this, cx| this.update_settings(cx, |s| s.theme = v));
                },
            )
        };
        let scale_opts: Vec<(u32, String)> = SCALES.iter().map(|s| (*s, format!("{s}%"))).collect();
        let scale_opts2: Vec<(u32, &str)> = scale_opts.iter().map(|(v, l)| (*v, l.as_str())).collect();
        let scale_seg = {
            let e = ent.clone();
            segmented(t, "seg-scale", &scale_opts2, &self.settings.scale, move |v, _, cx| {
                let v = *v;
                e.update(cx, |this, cx| this.update_settings(cx, |s| s.scale = v));
            })
        };
        let width_seg = {
            let e = ent.clone();
            segmented(t, "seg-width", &[(false, "Comfortable"), (true, "Full")], &self.settings.wide, move |v, _, cx| {
                let v = *v;
                e.update(cx, |this, cx| this.update_settings(cx, |s| s.wide = v));
            })
        };
        let ts_pos = match &self.text_select_tween {
            Some(tw) => tw.at(ui::ease_std).0,
            None => if self.settings.text_select { 1.0 } else { 0.0 },
        };
        let ts_switch = ui::switch(t, "text-select", ts_pos).on_click(cx.listener(|this, _, _, cx| {
            let on = !this.settings.text_select;
            this.text_select_tween = Some(Tween::new(if on { 0.0 } else { 1.0 }, if on { 1.0 } else { 0.0 }, 200));
            this.update_settings(cx, |s| s.text_select = on);
        }));

        // ServerUrlCard
        let dirty = self.server_dirty(cx);
        let saved = self.saved_flash.is_some();
        let testing = self.test == TestState::Testing;
        let server = div()
            .py(u(16.))
            .border_t_1()
            .border_color(c(line))
            .flex()
            .flex_col()
            .gap(u(8.))
            .child(ui::text("Server address", 13., 19.5, c(p.text)).font_weight(W_MEDIUM))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(u(8.))
                    .child(div().flex_1().min_w_0().child(self.server_input.clone()))
                    .child(
                        button(t, "srv-test", Variant::Secondary, Size::Icon(36.), p.card, testing)
                            .child(icon("plug", 14., c(p.text)))
                            .when(!testing, |b| b.on_click(cx.listener(|this, _, _, cx| this.test_connection(cx)))),
                    )
                    .child(
                        button(t, "srv-save", Variant::Primary, Size::Icon(36.), p.card, !dirty && !saved)
                            .child(icon(if saved { "check" } else { "save" }, 14., c(p.accent_ink)))
                            .when(dirty || saved, |b| b.on_click(cx.listener(|this, _, _, cx| this.save_server(cx)))),
                    ),
            )
            .when(self.test == TestState::Ok, |d| {
                d.child(ui::text(if dirty { "Server reached — remember to save" } else { "Server reached" }, 12., 16., c(p.green)))
            })
            .when(self.test == TestState::Fail, |d| d.child(ui::text("No server answered at this address", 12., 16., c(p.red))))
            .child(ui::text(
                "Backend address including port. Saved on this machine — point it at whichever machine runs the server.",
                12.,
                16.,
                c(p.muted),
            ));

        div()
            .flex()
            .flex_col()
            .child(accent_block)
            .child(row("Theme", None, theme_seg.into_any_element(), true))
            .child(row("Scale", Some("Scales every element."), scale_seg.into_any_element(), false))
            .child(row("Content width", Some("Full uses the whole window."), width_seg.into_any_element(), false))
            .child(row("Text selection", Some("Off feels more native."), ts_switch.into_any_element(), false))
            .child(server)
            .into_any_element()
    }

    // ─── About ──────────────────────────────────────────────────────────────

    pub fn render_about(&mut self, t: &Theme, cx: &mut Context<Self>) -> AnyElement {
        let p = &t.p;
        div()
            .flex()
            .flex_col()
            .items_center()
            .py(u(8.))
            .child(div().mb(u(12.)).child(ui::app_mark(t, 56.)))
            .child(ui::text(APP_NAME, 16., 24., c(p.text)).font_weight(W_SEMIBOLD))
            .child(ui::text(format!("v{VERSION}"), 12., 19.2, c(p.muted)).font_family(MONO).mt(u(2.)))
            .child(ui::text(TAGLINE, 14., 20., c(p.subtext)).mt(u(8.)).max_w(u(320.)).text_center())
            .child(
                div()
                    .mt(u(20.))
                    .w_full()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(u(8.))
                    .child(
                        button(t, "check-updates", Variant::Secondary, Size::Md, p.card, false)
                            .child(icon("refresh-cw", 14., c(p.text)))
                            .child("Check for updates")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.about_unsupported = true;
                                cx.notify();
                            })),
                    )
                    .child(
                        div().min_h(u(16.)).child(
                            ui::text(if self.about_unsupported { "Updates apply to packaged builds only." } else { "" }, 12., 16., c(p.muted)),
                        ),
                    ),
            )
            .into_any_element()
    }
}
