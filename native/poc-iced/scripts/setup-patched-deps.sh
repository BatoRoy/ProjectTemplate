#!/usr/bin/env bash
# Recreate the patched upstream crates that Cargo.toml's [patch.crates-io]
# points at. They live outside the repo, in ~/.cache/bato-native/ (override
# with BATO_NATIVE_CACHE for a dry run). Idempotent: existing dirs are kept.
set -euo pipefail
here=$(cd "$(dirname "$0")/.." && pwd)
cache=${BATO_NATIVE_CACHE:-$HOME/.cache/bato-native}
mkdir -p "$cache"
git_id=(-c user.name=bato -c user.email=bato@local)

crate() { # name version patch
  local dir="$cache/$1-$2-bato"
  [ -d "$dir" ] && { echo "ok   $dir"; return; }
  local tmp; tmp=$(mktemp -d)
  curl -sSfL "https://static.crates.io/crates/$1/$1-$2.crate" | tar -xz -C "$tmp"
  mv "$tmp/$1-$2" "$dir"; rmdir "$tmp"
  (cd "$dir" && git init -q && git add -A && git "${git_id[@]}" commit -qm "upstream $1 $2" && git "${git_id[@]}" am -q "$here/patches/$3")
  echo "new  $dir"
}

crate cosmic-text 0.15.0 cosmic-text-0.15.0-freetype-light-hinting.patch
crate cryoglyph 0.1.0 cryoglyph-0.1.0-skia-gamma.patch
crate iced_wgpu 0.14.0 iced_wgpu-0.14.0-surface-copy-src.patch

# iced's winit fork (value120 wheel deltas), pinned, with smol_str 0.2.
dir="$cache/winit-bato"
if [ -d "$dir" ]; then echo "ok   $dir"; else
  git clone -q https://github.com/iced-rs/winit.git "$dir"
  (cd "$dir" && git checkout -q -b bato-iced014 48116469f15d3bc52b5e56bb516694be8882870d \
     && git "${git_id[@]}" am -q "$here/patches/iced-winit-fork-smol-str-0.2.patch")
  echo "new  $dir"
fi
