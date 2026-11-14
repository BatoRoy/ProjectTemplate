#!/usr/bin/env bash
# Off-screen test output for native clients on Hyprland (real GPU, invisible).
#   native/tools/offscreen.sh up    # create output BATOPOC + route class ^batopoc-.* to it
#   native/tools/offscreen.sh down  # remove both
# While up, any window whose app_id starts with "batopoc-" opens there, floating
# 1280x800; screenshot it with native/tools/shot.sh <class> <out.png>.
# Runtime-only: a Hyprland reload or restart also clears it.
set -euo pipefail
case ${1:-} in
  up)
    hyprctl output create headless BATOPOC >/dev/null
    sleep 0.5
    # Far from the real monitors so the cursor can never wander onto it.
    hyprctl eval 'hl.monitor({ output = "BATOPOC", mode = "1920x1200@60", position = "-3000x-3000", scale = 1 })' >/dev/null
    sleep 0.5
    ws=$(hyprctl monitors -j | python3 -c "import json,sys;print([m for m in json.load(sys.stdin) if m['name']=='BATOPOC'][0]['activeWorkspace']['id'])")
    hyprctl eval "hl.window_rule({ name = \"batopoc-offscreen\", match = { class = \"^batopoc-.*\" }, workspace = \"$ws silent\", float = true, size = \"1280 800\", move = \"40 80\", no_initial_focus = true })" >/dev/null
    echo "BATOPOC up (workspace $ws)" ;;
  down)
    hyprctl eval 'hl.window_rule({ name = "batopoc-offscreen", match = { class = "^batopoc-.*" }, workspace = "11 silent" }):set_enabled(false)' >/dev/null 2>&1 || true
    hyprctl output remove BATOPOC >/dev/null
    echo "BATOPOC down" ;;
  *) echo "usage: $0 up|down" >&2; exit 2 ;;
esac
