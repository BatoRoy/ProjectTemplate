#!/usr/bin/env bash
# Off-screen test output for native clients on Hyprland (real GPU, invisible).
#   native/tools/offscreen.sh up      # create output BATOPOC + route class ^batopoc-.* to it
#   native/tools/offscreen.sh down    # remove both
#   native/tools/offscreen.sh check   # exit 0 only if routing is safe right now
# While up, any window whose app_id starts with "batopoc-" opens on the named
# workspace "batopoc", which lives only on BATOPOC, floating at 1280x800.
# Screenshot with native/tools/shot.sh, which refuses windows anywhere else.
#
# Fail-safe: the window rule is installed only after the output is verified to
# show workspace "batopoc". (An earlier version read the new output's workspace
# id before Hyprland had assigned one, the rule pointed nowhere, and a test
# window opened on the user's focused monitor.) Runtime-only state: a Hyprland
# reload clears it. Not concurrency-safe: one user of `up`/`down` at a time.
set -euo pipefail

OUT=BATOPOC
WS=batopoc

monitor_ws() {
  hyprctl monitors -j | python3 -c "
import json,sys
ms=[m for m in json.load(sys.stdin) if m['name']=='$OUT']
print(ms[0]['activeWorkspace']['name'] if ms else '')"
}

rule_off() {
  # A rule that can match nothing: replaces the routing rule by name.
  hyprctl eval "hl.window_rule({ name = \"batopoc-offscreen\", match = { class = \"^batopoc-never-matches$\" }, workspace = \"name:$WS silent\" })" >/dev/null 2>&1 || true
}

case ${1:-} in
  up)
    # Bind the workspace to the output *before* the output exists, so it is the
    # output's first and default workspace.
    hyprctl eval "hl.workspace_rule({ workspace = \"name:$WS\", monitor = \"$OUT\", default = true })" >/dev/null
    if [ -z "$(monitor_ws)" ]; then
      hyprctl output create headless $OUT >/dev/null
    fi
    # Far from the real monitors so the cursor can never wander onto it.
    hyprctl eval "hl.monitor({ output = \"$OUT\", mode = \"1920x1200@60\", position = \"-3000x-3000\", scale = 1 })" >/dev/null
    for _ in $(seq 50); do
      [ "$(monitor_ws)" = "$WS" ] && break
      sleep 0.1
    done
    if [ "$(monitor_ws)" != "$WS" ]; then
      echo "offscreen: $OUT shows workspace '$(monitor_ws)', not '$WS'; NOT installing the window rule" >&2
      rule_off
      hyprctl output remove $OUT >/dev/null 2>&1 || true
      exit 1
    fi
    hyprctl eval "hl.window_rule({ name = \"batopoc-offscreen\", match = { class = \"^batopoc-.*\" }, workspace = \"name:$WS silent\", float = true, size = \"1280 800\", move = \"40 80\", no_initial_focus = true })" >/dev/null
    echo "offscreen: up ($OUT shows workspace $WS)" ;;
  down)
    rule_off
    hyprctl output remove $OUT >/dev/null 2>&1 || true
    echo "offscreen: down" ;;
  check)
    [ "$(monitor_ws)" = "$WS" ] || { echo "offscreen: not up" >&2; exit 1; }
    echo "offscreen: ok" ;;
  *) echo "usage: $0 up|down|check" >&2; exit 2 ;;
esac
