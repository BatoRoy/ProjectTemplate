//! Bundled fonts. Inter 4 (wght axis, opsz pinned to 14) and JetBrains Mono,
//! loaded from `native/assets/fonts` at compile time.
//!
//! The family names are pinned: this machine also has a system `Inter`
//! (static instances in `/usr/share/fonts/inter/Inter.ttc`) and several
//! `JetBrains Mono` variants, and fontdb would happily resolve "Inter" weight
//! 600 to the system SemiBold. So every system face of those families is
//! removed from the font database before anything is shaped.

use std::borrow::Cow;

use iced::advanced::graphics::text::cosmic_text::fontdb;
use iced::advanced::graphics::text::font_system;

pub const INTER: &[u8] = include_bytes!("../../assets/fonts/InterVariable-latin.ttf");
pub const JBM_REGULAR: &[u8] = include_bytes!("../../assets/fonts/JetBrainsMono-Regular-latin.ttf");
pub const JBM_MEDIUM: &[u8] = include_bytes!("../../assets/fonts/JetBrainsMono-Medium-latin.ttf");

pub const SANS: &str = "Inter";
pub const MONO: &str = "JetBrains Mono";

/// hhea ascent / descent in em (spec §2.1), for the CSS line-box model.
pub const INTER_ASC: f32 = 1984.0 / 2048.0;
pub const INTER_DESC: f32 = 494.0 / 2048.0;
pub const JBM_ASC: f32 = 1020.0 / 1000.0;
pub const JBM_DESC: f32 = 300.0 / 1000.0;

pub fn setup() {
    let mut fs = font_system().write().expect("font system");
    // Drop system faces that share our family names.
    {
        let db = fs.raw().db_mut();
        let shadowing: Vec<fontdb::ID> = db
            .faces()
            .filter(|f| !matches!(f.source, fontdb::Source::Binary(_)))
            .filter(|f| {
                f.families.iter().any(|(name, _)| {
                    name == "Inter" || name.starts_with("Inter ") || name.starts_with("JetBrains Mono")
                })
            })
            .map(|f| f.id)
            .collect();
        for id in shadowing {
            db.remove_face(id);
        }
    }
    fs.load_font(Cow::Borrowed(INTER));
    fs.load_font(Cow::Borrowed(JBM_REGULAR));
    fs.load_font(Cow::Borrowed(JBM_MEDIUM));

    // fontdb registers a variable font once, at its default weight (400).
    // cosmic-text only accepts exact weight matches inside a family, so a
    // request for Inter 500 or 600 silently fell back to *another family*
    // (NotoSans-Medium, FreeSans Bold). Register the Inter face once per
    // CSS weight; cosmic-text then applies `wght` = the requested weight
    // for both shaping (harfrust) and rasterization.
    let db = fs.raw().db_mut();
    let inter: Vec<fontdb::FaceInfo> = db
        .faces()
        .filter(|f| matches!(f.source, fontdb::Source::Binary(_)) && f.families.iter().any(|(n, _)| n == SANS))
        .cloned()
        .collect();
    for face in inter {
        for w in (100..=900).step_by(100) {
            if w != face.weight.0 {
                let mut copy = face.clone();
                copy.weight = fontdb::Weight(w);
                db.push_face_info(copy);
            }
        }
    }
}

pub fn sans() -> iced::Font {
    iced::Font::with_name(SANS)
}

pub fn mono() -> iced::Font {
    iced::Font::with_name(MONO)
}
