#!/usr/bin/env bash
# Hands-on comparison of the client flavors against one backend.
#   native/try.sh iced | gpui | electron
# Starts app-server on :42990 (bato-auth off, so /api/info answers), opens the
# client on your screen, and stops the server when the client window closes.
# Extra env passes through, e.g. BATO_SCROLL_DEBUG=1 native/try.sh iced
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
flavor=${1:?usage: native/try.sh iced|gpui|electron}
port=42990

case $flavor in
  iced)     bin=$root/native/poc-iced/target/release/batopoc-iced ;;
  gpui)     bin=$root/native/poc-gpui/target/release/batopoc-gpui ;;
  electron) bin=$root/dist/electron/linux-unpacked/app ;;
  *) echo "unknown flavor: $flavor" >&2; exit 2 ;;
esac
[ -x "$bin" ] || { echo "not built: $bin" >&2; exit 1; }

(cd "$root/app-server" && go build -o "$root/dist/try-app-server" .)
env -u BATO_AUTH_URL "$root/dist/try-app-server" --port $port >/dev/null 2>&1 &
server=$!
trap 'kill $server 2>/dev/null' EXIT

# A visible app_id: batopoc-* is routed off-screen during automated testing.
export BATO_APP_ID=bato-native-$flavor
export BATO_BACKEND_URL=http://127.0.0.1:$port
"$bin" "${@:2}"
