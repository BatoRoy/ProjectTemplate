//! Home: header, Backend card (real `/api/health` then `/api/info`),
//! Notifications card, Components card. The same page as the Electron
//! client's `HomePage.tsx`; replace it with your app's own.

use bato_kit::frame::Brand;
use bato_kit::iced::widget::{column, row};
use bato_kit::prelude::*;

use crate::{Msg, Status};

pub struct Home<'h> {
    pub brand: &'h Brand,
    pub backend: String,
    pub status: Status,
    pub version: Option<String>,
    pub name: &'h str,
    pub auto_update: bool,
}

fn toast(text: impl Into<String>, tone: Tone) -> Msg {
    Msg::Frame(bato_kit::frame::Msg::Toast(text.into(), tone))
}

pub fn view<'a>(cx: &Ctx, h: Home<'_>, wide: bool) -> El<'a, Msg> {
    let p = cx.p();
    let card_cx = cx.on_card();

    let header = row![
        app_mark(cx, h.brand.icon, 56.0),
        column![
            label(h.brand.name, 30.0, 36.0)
                .semibold()
                .tracking(-0.025)
                .color(c(p.text)),
            vgap(4.0),
            label(h.brand.tagline, 14.0, 20.0).color(c(p.subtext)),
        ]
        .width(Length::Fill),
        button(cx)
            .secondary()
            .sm()
            .icon(Ico::Command)
            .label("Commands")
            .push(kbd(&cx.on(cx.wash(0.04)), bato_kit::frame::MOD_K))
            .on_press(Msg::Frame(bato_kit::frame::Msg::OpenPalette)),
    ]
    .spacing(20)
    .align_y(Alignment::Center);

    let (tone, status_text) = match h.status {
        Status::Online => (Tone::Success, "Online"),
        Status::Offline => (Tone::Error, "Offline"),
        Status::Checking => (Tone::Warning, "Checking…"),
    };
    let dt_w = label("Version", 13.0, 19.5).shape(f32::INFINITY).width();
    let dl_row = |dt: &'static str, dd: String| -> El<'a, Msg> {
        row![
            label(dt, 13.0, 19.5).color(c(p.muted)).width(dt_w),
            gap(16.0),
            label(dd, 12.0, 16.0)
                .mono()
                .color(c(p.text))
                .ellipsis()
                .width(Length::Fill),
        ]
        .align_y(Alignment::Center)
        .into()
    };
    let backend_body = column![
        dl_row("Server", h.backend),
        vgap(6.0),
        dl_row("Version", h.version.unwrap_or_else(|| "—".into())),
        vgap(20.0),
        row![
            rich(
                vec![Span::new("Run "), Span::new("make dev-server").mono()],
                12.0,
                16.0
            )
            .color(c(p.muted))
            .width(Length::Fill),
            button(&card_cx)
                .secondary()
                .sm()
                .push(icon(Ico::RefreshCw, 13.0).spin(h.status == Status::Checking))
                .label("Ping")
                .on_press(Msg::Ping),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    ];
    let backend = card(cx, "Backend")
        .icon(Ico::Server)
        .action(badge(&card_cx, tone, status_text).dot())
        .body(backend_body);

    let notif_body = column![
        rich(
            vec![
                // Differs from Electron's wording on purpose (this client has no
                // main process or browser fallback), so Home's diff against the
                // Electron reference includes this line.
                Span::new("A native OS notification, sent over "),
                Span::new("D-Bus").mono().size(12.0),
                Span::new(". Clicking it brings this window to the front."),
            ],
            13.0,
            21.125
        )
        .color(c(p.subtext))
        .wrap(),
        vgap(20.0),
        row![
            Space::new().width(Length::Fill),
            button(&card_cx)
                .secondary()
                .sm()
                .icon(Ico::Bell)
                .label("Notify")
                .on_press(Msg::Notify)
        ],
    ];
    let notif = card(cx, "Notifications").icon(Ico::Bell).body(notif_body);

    let name_field = column![
        label("Your name", 12.0, 16.0).medium().color(c(p.subtext)),
        vgap(6.0),
        input(&card_cx, "Type something…", h.name, Msg::Name),
    ];
    let hello = if h.name.is_empty() {
        "Success toast".to_string()
    } else {
        format!("Hello, {}!", h.name)
    };
    let buttons = row![
        button(&card_cx)
            .label("Primary")
            .on_press(toast(hello, Tone::Success)),
        button(&card_cx)
            .secondary()
            .label("Secondary")
            .on_press(toast("Just so you know", Tone::Info)),
        button(&card_cx)
            .ghost()
            .label("Ghost")
            .on_press(toast("Quiet action", Tone::Info)),
        button(&card_cx)
            .success()
            .label("Success")
            .on_press(toast("All good", Tone::Success)),
        button(&card_cx)
            .danger()
            .label("Danger")
            .on_press(toast("Something went wrong", Tone::Error)),
    ]
    .spacing(8);
    let badges = row![
        badge(&card_cx, Tone::Neutral, "Neutral"),
        badge(&card_cx, Tone::Info, "Info"),
        badge(&card_cx, Tone::Success, "Success"),
        badge(&card_cx, Tone::Warning, "Warning"),
        badge(&card_cx, Tone::Error, "Error"),
    ]
    .spacing(8);
    let update_row = column![
        hline(c(card_cx.line())),
        vgap(20.0),
        row![
            column![
                label("Update automatically", 13.0, 19.5)
                    .medium()
                    .color(c(p.text)),
                vgap(2.0),
                label("Download new versions in the background.", 12.0, 16.0).color(c(p.muted)),
            ]
            .width(Length::Fill),
            switch(&card_cx, h.auto_update, Msg::AutoUpdate),
        ]
        .spacing(24)
        .align_y(Alignment::Center),
    ];
    let mut spans = vec![
        Span::new("Open "),
        Span::new("App Options").weight(500).color(c(p.subtext)),
        Span::new(" in the sidebar to switch theme, accent and scale."),
    ];
    if cfg!(feature = "examples") {
        spans.extend([
            Span::new(" The "),
            Span::new("Examples").weight(500).color(c(p.subtext)),
            Span::new(" page has the full component kit."),
        ]);
    }
    let hint = rich(spans, 12.0, 16.0).color(c(p.muted)).wrap();
    let comp_body = column![name_field, buttons, badges, update_row, hint].spacing(24);
    let version_badge = badge(&card_cx, Tone::Info, format!("v{}", h.brand.version)).mono();
    let components = card(cx, "Components")
        .icon(Ico::Blocks)
        .action(version_badge)
        .body(comp_body);

    page(
        column![
            header,
            vgap(40.0),
            grid(vec![backend.into(), notif.into()], 16.0),
            vgap(16.0),
            components
        ],
        wide,
        [48.0, 32.0],
    )
}
