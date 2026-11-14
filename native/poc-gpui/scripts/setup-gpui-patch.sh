#!/usr/bin/env bash
# Recreates the three patched GPUI crates that Cargo.toml's [patch.crates-io]
# points at (gpui-pre 0.3.12 = zed@d2a4fb6ded7c). Run once before `cargo build`.
# The checkouts live outside the repo, under ~/.cache/bato-native/.
#   gpui-pre-linux  (crates/gpui_linux): Wayland axis_stop → TouchPhase::Ended; wl_touch → TouchEvent
#   gpui-pre        (crates/gpui):       CSS-like line heights, wrapping and pixel snapping
#   gpui-pre-wgpu   (crates/gpui_wgpu):  honour WGPU_BACKEND (skip the GL probe on Vulkan systems)
set -euo pipefail
here=$(cd "$(dirname "$0")/.." && pwd)
cache="$HOME/.cache/bato-native"
mkdir -p "$cache"

setup() { # crate version diff...
  local crate=$1 ver=$2 dest="$cache/$1-$2"; shift 2
  local all; all=$(cd "$here/patches" && cat "$@")
  if [ -f "$dest/.bato-patched" ] && [ "$(cat "$dest/.bato-patched")" = "$all" ]; then
    echo "up to date: $dest"; return
  fi
  local tmp; tmp=$(mktemp -d)
  curl -sSfL "https://static.crates.io/crates/$crate/$crate-$ver.crate" | tar xz -C "$tmp"
  rm -rf "$dest"
  mv "$tmp/$crate-$ver" "$dest"
  rmdir "$tmp"
  # The diffs use upstream zed paths (crates/<name>/...): strip 3 components.
  local d
  for d in "$@"; do patch -s -d "$dest" -p3 < "$here/patches/$d"; done
  printf '%s' "$all" > "$dest/.bato-patched"
  echo "patched: $dest"
}

setup gpui-pre-linux 0.3.12 gpui-linux-wayland-axis-stop.diff gpui-linux-wayland-touch.diff
setup gpui-pre 0.3.12 gpui-fractional-text-metrics.diff gpui-wrap-hanging-spaces.diff gpui-css-pixel-snapping.diff
setup gpui-pre-wgpu 0.3.12 gpui-wgpu-backend-env.diff
