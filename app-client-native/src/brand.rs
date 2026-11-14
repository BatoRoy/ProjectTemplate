//! Who the app is. `new-app.sh` stamps the name and slug; the rest is yours.

use bato_kit::Ico;
use bato_kit::frame::Brand;
use bato_shell::AppIdentity;

/// The platform identity. `slug` is the registry name, the binary name, the
/// config dir (`~/.config/<slug>/`) and the Wayland app_id / X11 WM_CLASS, so
/// it must match `bato.json` (`name`, `desktop.startupWMClass`). `BATO_APP_ID`
/// overrides the app_id (off-screen tests, side-by-side dev builds).
pub const APP: AppIdentity = AppIdentity::new(
    "App", // name: stamped by new-app.sh
    "app", // slug: stamped by new-app.sh
    env!("APP_VERSION"),
);

/// What the frame shows: sidebar header, Home, About.
pub const BRAND: Brand = Brand {
    name: APP.product_name,
    tagline: "A starting template — native Rust (iced) + Go.",
    version: APP.version,
    slug: APP.slug,
    // Default accent; users can still change it in App Options. Match the
    // icon colour in bato/icons/generate.py ACCENTS.
    accent: "#8b5cf6",
    // Sidebar / About badge: any lucide icon in bato_kit::Ico.
    icon: Ico::Box,
};

/// Where the updater looks for new releases (`BATO_REGISTRY_URL` overrides).
pub const REGISTRY_URL: &str = bato_shell::updater::DEFAULT_REGISTRY;
