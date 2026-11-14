//! Persisted client settings, shared by every native client.
//!
//! Lives in `~/.config/<slug>/settings.json`, the same file the Electron shell
//! uses for `backendUrl`, so switching an app between the Electron and native
//! clients keeps the server address. Electron keeps the look settings in
//! localStorage instead; the native client stores them here alongside. Unknown
//! keys are preserved on save so the two clients never erase each other's data.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::ThemeId;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    /// Saved backend address; `None` falls back to [`backend_url`]'s defaults.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend_url: Option<String>,
    pub theme: ThemeId,
    /// Raw accent as chosen (`#rrggbb`); `None` = the app's brand accent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accent: Option<String>,
    /// UI scale, percent (one of [`crate::SCALES`]).
    pub scale: u32,
    /// Content width "Full" (true) vs "Comfortable" 896px (false).
    pub wide: bool,
    pub text_select: bool,
    pub sidebar_collapsed: bool,
    /// Every key this struct does not know, kept verbatim.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            backend_url: None,
            theme: ThemeId::Dark,
            accent: None,
            scale: 100,
            wide: false,
            text_select: false,
            sidebar_collapsed: false,
            extra: Map::new(),
        }
    }
}

/// `~/.config/<slug>`
pub fn config_dir(slug: &str) -> PathBuf {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .unwrap_or_else(|| PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".config"));
    base.join(slug)
}

impl Settings {
    pub fn path(slug: &str) -> PathBuf {
        config_dir(slug).join("settings.json")
    }

    /// Never fails: a missing or corrupt file yields defaults (like main.js).
    pub fn load(slug: &str) -> Self {
        Self::load_from(&Self::path(slug))
    }

    pub fn load_from(path: &Path) -> Self {
        fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, slug: &str) -> io::Result<()> {
        self.save_to(&Self::path(slug))
    }

    /// Atomic write (temp file + rename) so a crash never leaves half a file.
    pub fn save_to(&self, path: &Path) -> io::Result<()> {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        let tmp = path.with_extension("json.tmp");
        fs::write(&tmp, serde_json::to_string_pretty(self)?)?;
        fs::rename(tmp, path)
    }

    /// The accent to derive the palette from.
    pub fn accent_or<'a>(&'a self, brand_default: &'a str) -> &'a str {
        self.accent.as_deref().unwrap_or(brand_default)
    }

    pub fn scale_factor(&self) -> f32 {
        self.scale as f32 / 100.0
    }
}

/// Backend URL resolution, same order as `bridge.ts`: `BATO_BACKEND_URL`
/// (a supervisor handing over a bundled server), then the saved address,
/// then `http://localhost:8080`. Trailing slashes are trimmed.
pub fn backend_url(settings: &Settings) -> String {
    std::env::var("BATO_BACKEND_URL")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| settings.backend_url.clone().filter(|s| !s.trim().is_empty()))
        .unwrap_or_else(|| "http://localhost:8080".into())
        .trim()
        .trim_end_matches('/')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_and_keeps_unknown_keys() {
        let dir = std::env::temp_dir().join(format!("bato-theme-test-{}", std::process::id()));
        let path = dir.join("settings.json");
        fs::create_dir_all(&dir).unwrap();
        fs::write(&path, r#"{"backendUrl":"http://10.0.0.5:42100","electronOnly":{"x":1},"theme":"dim"}"#).unwrap();

        let mut s = Settings::load_from(&path);
        assert_eq!(s.theme, ThemeId::Dim);
        assert_eq!(s.backend_url.as_deref(), Some("http://10.0.0.5:42100"));
        assert_eq!(s.scale, 100);
        s.scale = 110;
        s.save_to(&path).unwrap();

        let raw: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(raw["electronOnly"]["x"], 1);
        assert_eq!(raw["scale"], 110);
        assert_eq!(raw["backendUrl"], "http://10.0.0.5:42100");
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn corrupt_file_gives_defaults() {
        let dir = std::env::temp_dir().join(format!("bato-theme-corrupt-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("settings.json");
        fs::write(&path, "{not json").unwrap();
        assert_eq!(Settings::load_from(&path), Settings::default());
        fs::remove_dir_all(dir).unwrap();
    }
}
