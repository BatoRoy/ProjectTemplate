#!/usr/bin/env bash
# Screenshot a window running on the off-screen BATOPOC output (real GPU, invisible to the user).
# Usage: poc-shot.sh <window-class> <out.png>
# Any window whose Wayland app_id/class starts with "batopoc-" is routed there automatically
# (floating, 1280x800). Never focus, move or switch workspaces on the user's real monitors.
set -euo pipefail
cls=$1; out=$2
geo=$(hyprctl clients -j | python3 -c "
import json,sys
cs=[c for c in json.load(sys.stdin) if c['class']==sys.argv[1]]
if not cs: sys.exit('no window with class '+sys.argv[1])
c=cs[0]; print(f\"{c['at'][0]},{c['at'][1]} {c['size'][0]}x{c['size'][1]}\")" "$cls")
grim -g "$geo" "$out"
echo "$out ($geo)"
