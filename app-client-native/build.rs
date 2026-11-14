//! The app version lives in the repo's `VERSION` file (version.sh bumps it),
//! not in Cargo.toml: `APP_VERSION` for `brand.rs`.

use std::path::Path;

fn main() {
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("../VERSION");
    println!("cargo:rerun-if-changed={}", file.display());
    let version = std::fs::read_to_string(&file).unwrap_or_else(|_| "0.0.0".into());
    println!("cargo:rustc-env=APP_VERSION={}", version.trim());
}
