#!/usr/bin/env bash
# Screenshot a test window on the off-screen BATOPOC output (see offscreen.sh).
#   native/tools/shot.sh <pid|window-class> <out.png>
# Refuses unless the window is on BATOPOC: it never captures the user's
# real monitors. Prefer a PID — two instances can share a class.
set -euo pipefail
sel=$1; out=$2
geo=$(hyprctl -j monitors | python3 -c "
import json,sys,subprocess
mons={m['id']:m['name'] for m in json.load(sys.stdin)}
cs=json.loads(subprocess.check_output(['hyprctl','-j','clients']))
sel=sys.argv[1]
cs=[c for c in cs if (str(c['pid'])==sel if sel.isdigit() else c['class']==sel)]
if not cs: sys.exit('shot: no window for '+sel)
if len(cs)>1: sys.exit('shot: several windows match '+sel+'; pass a PID')
c=cs[0]
if mons.get(c['monitor'])!='BATOPOC':
    sys.exit('shot: REFUSING: window is on '+str(mons.get(c['monitor']))+', not BATOPOC')
print(f\"{c['at'][0]},{c['at'][1]} {c['size'][0]}x{c['size'][1]}\")" "$sel")
grim -g "$geo" "$out"
echo "$out ($geo)"
