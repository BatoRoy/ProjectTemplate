#!/usr/bin/env bash
set -euo pipefail

# Scaffold a new app from this template with a FRESH git history.
#
#   ./new-app.sh <AppName> <dest-dir> [--electron|--qml] [--force]
#
# Exports the template's committed HEAD (git archive — ignored/untracked files
# never leak in), stamps the app name/slug into the identity files, resets the
# version to 0.1.0, records template provenance (version + commit) in the
# initial commit message and a .template file, then git-inits the destination
# as a single-commit repo. No template history is carried over.
#
# Linux-oriented: uses GNU `sed -i` (same as version.sh).

usage() {
  echo "Usage: $0 <AppName> <dest-dir> [--electron|--qml] [--force]"
  echo ""
  echo "  AppName      Display name, e.g. MyApp or \"My App\" (letters, digits, space, - _)"
  echo "  dest-dir     Directory to create (must not exist or be empty)"
  echo "  --electron   Electron + React client (default)"
  echo "  --qml        Qt/QML client — lighter and faster to start, no web bundle"
  echo "  --force      Skip the dirty-template confirmation"
  echo ""
  echo "The template carries both clients; exactly one survives in the new app."
  echo "See README → \"Choosing a client\" for the tradeoff."
}

NAME="" DEST="" FORCE=0 FLAVOR=""
for arg in "$@"; do
  case "$arg" in
    --force) FORCE=1 ;;
    --qml) FLAVOR="qml" ;;
    --electron) FLAVOR="electron" ;;
    -h|--help) usage; exit 0 ;;
    *)
      if [[ -z "$NAME" ]]; then NAME="$arg"
      elif [[ -z "$DEST" ]]; then DEST="$arg"
      else echo "Error: unexpected argument '$arg'" >&2; usage >&2; exit 1
      fi ;;
  esac
done

# Electron by default: it is what every existing app in the suite uses, so an
# unqualified invocation keeps behaving the way it always has.
FLAVOR="${FLAVOR:-electron}"

if [[ -z "$NAME" || -z "$DEST" ]]; then
  usage >&2
  exit 1
fi

if ! [[ "$NAME" =~ ^[A-Za-z][A-Za-z0-9\ _-]*$ ]]; then
  echo "Error: app name must start with a letter and contain only letters, digits, spaces, '-' or '_'" >&2
  exit 1
fi

# Derived identities:
#   SLUG      lowercase, spaces→'-', only [a-z0-9-]  → config dirs, appId, s3 path
#   EXEC_NAME NAME with spaces stripped              → bato.json executableName
SLUG="$(echo "$NAME" | tr '[:upper:]' '[:lower:]' | tr ' _' '--' | tr -cd 'a-z0-9-')"
EXEC_NAME="${NAME// /}"

TEMPLATE_ROOT="$(cd "$(dirname "$(realpath "$0")")" && pwd)"
if [[ ! -f "$TEMPLATE_ROOT/VERSION" ]] || ! git -C "$TEMPLATE_ROOT" rev-parse --git-dir >/dev/null 2>&1; then
  echo "Error: $TEMPLATE_ROOT does not look like the template repo (no VERSION or not a git repo)" >&2
  exit 1
fi

if [[ -e "$DEST" ]] && [[ -n "$(ls -A "$DEST" 2>/dev/null)" ]]; then
  echo "Error: destination '$DEST' exists and is not empty" >&2
  exit 1
fi

# The snapshot is taken from committed HEAD; warn if the template tree is dirty.
DIRTY="$(git -C "$TEMPLATE_ROOT" status --porcelain)"
if [[ -n "$DIRTY" && "$FORCE" -ne 1 ]]; then
  echo "⚠ Template working tree has uncommitted changes:"
  echo "$DIRTY" | sed 's/^/    /'
  echo "  The new app is exported from committed HEAD — these changes will NOT be included."
  if [[ -t 0 ]]; then
    read -r -p "Continue anyway? [y/N] " reply
    [[ "$reply" =~ ^[Yy]$ ]] || { echo "Aborted."; exit 1; }
  else
    echo "  Re-run with --force to proceed." >&2
    exit 1
  fi
fi

TPL_VERSION="$(tr -d '[:space:]' < "$TEMPLATE_ROOT/VERSION")"
TPL_SHA="$(git -C "$TEMPLATE_ROOT" rev-parse --short HEAD)"

mkdir -p "$DEST"
DEST="$(cd "$DEST" && pwd)"

echo "→ Exporting template v$TPL_VERSION ($TPL_SHA) → $DEST  [client: $FLAVOR]"
git -C "$TEMPLATE_ROOT" archive HEAD | tar -x -C "$DEST"
rm -f "$DEST/new-app.sh"   # an app doesn't create apps

# ── Keep one client ─────────────────────────────────────────────────────────
# Both clients ship in the template so improvements to either can be made in one
# place, but an app wants exactly one: two clients means two things to keep working
# and an ambiguous `make dev-client`. The Makefile detects which one is present, so
# deleting the other is all that is needed — nothing to configure.
#
# The release workflow is per-flavor for the same reason. BatoAI is the cautionary
# tale: it swapped its renderer for QML and left the template's Electron workflow in
# place, so pushing a version tag has failed there ever since — CI still runs `npm ci`
# in a frontend directory that no longer exists.
if [[ "$FLAVOR" == "qml" ]]; then
  rm -rf "$DEST/app-client"
  rm -f  "$DEST/AUTOUPDATE.md"      # electron-updater only; a desktop release has no self-update
  rm -f  "$DEST/.github/workflows/release-electron.yml"
else
  rm -rf "$DEST/app-client-qml"
  rm -f  "$DEST/QML-CLIENT.md" "$DEST/PARITY.md"
  rm -f  "$DEST/.github/workflows/release-qml.yml"
  rm -f  "$DEST/tools/check-qml.py" "$DEST/tools/check-desktop-manifest.mjs" "$DEST/tools/check-deps.mjs"
  rmdir  "$DEST/tools" 2>/dev/null || true
fi

# ── Version reset to 0.1.0 ──────────────────────────────────────────────────
# Same sync targets as version.sh, which can't be reused here (it commits+tags).
# Each file is optional, mirroring version.sh's skip-if-missing convention.
NEW_VERSION="0.1.0"
echo "$NEW_VERSION" > "$DEST/VERSION"
if [[ -f "$DEST/app-client/frontend/src/lib/version.ts" ]]; then
  sed -i "s/export const VERSION = '[0-9]*\.[0-9]*\.[0-9]*'/export const VERSION = '$NEW_VERSION'/" \
    "$DEST/app-client/frontend/src/lib/version.ts"
fi
if [[ -f "$DEST/app-server/version.go" ]]; then
  sed -i "s/var Version = \"[0-9]*\.[0-9]*\.[0-9]*\"/var Version = \"$NEW_VERSION\"/" \
    "$DEST/app-server/version.go"
fi
if [[ -f "$DEST/app-client-qml/version.go" ]]; then
  sed -i "s/var Version = \"[0-9]*\.[0-9]*\.[0-9]*\"/var Version = \"$NEW_VERSION\"/" \
    "$DEST/app-client-qml/version.go"
fi

# ── Identity stamping ────────────────────────────────────────────────────────
# Each sed is anchored to the exact placeholder in the template tree and
# verified afterwards, so template drift fails loudly instead of producing a
# half-branded app.
stamp() { # stamp <file> <sed-expr> <expected-substring>
  local file="$DEST/$1" expr="$2" expect="$3"
  sed -i "$expr" "$file"
  grep -qF "$expect" "$file" || {
    echo "✗ Stamping failed: expected '$expect' in $1." >&2
    echo "  The template layout changed — update new-app.sh to match." >&2
    exit 1
  }
}

echo "→ Stamping identity: name='$NAME' slug='$SLUG' exec='$EXEC_NAME'"

if [[ "$FLAVOR" == "electron" ]]; then
  stamp app-client/frontend/src/brand.ts \
    "s/appName: 'App',/appName: '$NAME',/" "appName: '$NAME',"
  stamp app-client/frontend/src/brand.ts \
    "s/slug: 'app',/slug: '$SLUG',/" "slug: '$SLUG',"
  stamp app-client/electron/identity.js \
    "s/appId: 'com\.example\.app',/appId: 'com.example.$SLUG',/" "appId: 'com.example.$SLUG',"
  stamp app-client/electron/identity.js \
    "s/productName: 'App',/productName: '$NAME',/" "productName: '$NAME',"
  stamp app-client/electron/identity.js \
    "s/slug: 'app',/slug: '$SLUG',/" "slug: '$SLUG',"
  stamp app-client/frontend/index.html \
    "s|<title>App</title>|<title>$NAME</title>|" "<title>$NAME</title>"
else
  # identity.go is the host process's own name; Brand.qml is what the UI displays.
  # Two files rather than one because the Go side cannot read QML and the QML side is
  # not allowed to depend on a build step — the same split as the Electron flavor's
  # identity.js / brand.ts.
  stamp app-client-qml/identity.go \
    "s|appID:       \"com\.example\.app\",|appID:       \"com.example.$SLUG\",|" "appID:       \"com.example.$SLUG\","
  stamp app-client-qml/identity.go \
    "s|productName: \"App\",|productName: \"$NAME\",|" "productName: \"$NAME\","
  stamp app-client-qml/identity.go \
    "s|slug:        \"app\",|slug:        \"$SLUG\",|" "slug:        \"$SLUG\","
  stamp app-client-qml/qml/App/Brand.qml \
    "s|readonly property string appName: \"App\"|readonly property string appName: \"$NAME\"|" \
    "readonly property string appName: \"$NAME\""
  stamp app-client-qml/qml/App/Brand.qml \
    "s|readonly property string slug: \"app\"|readonly property string slug: \"$SLUG\"|" \
    "readonly property string slug: \"$SLUG\""
fi

stamp app-server/internal/config/config.go \
  "s|configDir = \".config/app\"|configDir = \".config/$SLUG\"|" "configDir = \".config/$SLUG\""

# JSON files are edited with node (guaranteed present for this stack) instead
# of sed, so structure changes can't corrupt them.
node - "$DEST" "$NAME" "$SLUG" "$EXEC_NAME" "$NEW_VERSION" "$FLAVOR" <<'EOF'
const fs = require('fs')
const path = require('path')
const [dest, name, slug, execName, version, flavor] = process.argv.slice(2)

function edit(rel, fn) {
  const file = path.join(dest, rel)
  if (!fs.existsSync(file)) return
  const json = JSON.parse(fs.readFileSync(file, 'utf8'))
  fn(json)
  fs.writeFileSync(file, JSON.stringify(json, null, 2) + '\n')
}

edit('app-client/package.json', (p) => {
  p.version = version
  p.build.appId = `com.example.${slug}`
  p.build.productName = name
  p.build.publish.path = `apps/${slug}`
  p.build.linux.executableName = slug
})
edit('app-client/frontend/package.json', (p) => { p.version = version })
edit('app-client/bato.json', (p) => {
  p.name = slug
  p.executableName = execName
  p.s3Path = `apps/${slug}`
})
// The QML client publishes as type "desktop": a tarball plus a manifest saying what
// the install should do.
//
// The slug is used for the binary, the artifact, the PATH command and the launcher's
// Exec — not ExecName. Unlike the Electron flavor, where `executableName` names an
// AppImage file, this name becomes a command you type, and `myapp` is a better command
// than `MyApp`. It also keeps one identifier where the Makefile, the manifest and
// `bato install <name>` all have to agree; BatoAI does the same.
//
// Only the launcher's *label* gets the display name.
//
// Not stamped, deliberately: `startupWMClass`. It stays org.qt-project.qml because
// that is what the stock qml6 runtime reports for every app it launches, regardless of
// the binary's name — verified, and unchangeable from QML. See app-client-qml/CLIENT.md.
edit('app-client-qml/bato.json', (p) => {
  p.name = slug
  p.description = `Desktop client for ${name} (Qt/QML + Go)`
  p.artifacts = [slug, 'app-server', 'qml']
  p.bin = [{ name: slug, target: slug }]
  p.desktop = { ...p.desktop, name, exec: slug }
})
// The deployable-server identity. `port` is deliberately left unset: it has to
// be claimed in BatoApps/PORTS.md by hand, and a stamped-in default would have
// every generated app collide on the same one. `bato deploy` says so plainly
// when it's missing rather than deploying something unreachable.
//
// `domain`, by contrast, IS stamped: *.bato.lan is covered by wildcard DNS and
// a wildcard certificate, so a new hostname costs no DNS, Traefik or cert work,
// and nothing is created until `bato deploy provision` runs. Note it is the
// bare slug, not `${slug}-server` — a hostname is public-facing, and
// myapp-server.bato.lan reads like an implementation detail.
edit('app-server/bato.json', (p) => {
  p.name = `${slug}-server`
  p.description = `Backend for ${name}`
  // Where the daemon keeps its data when it runs under systemd. `bato install`
  // writes this into the unit as Environment=, expanding `~` itself — systemd
  // expands neither `~` nor $HOME there. Without it the server would default to
  // a data dir inside the release directory and lose it on the next upgrade.
  p.env = { ...p.env, APP_DATA_DIR: `~/.local/share/${slug}` }
  p.deploy.app = `${execName}Server`
  p.deploy.image = `${slug}-server`
  p.deploy.domain = `${slug}.bato.lan`
  p.deploy.volumes = [{ name: `${slug}-server-data`, path: '/data' }]
})
EOF
if [[ "$FLAVOR" == "electron" ]]; then
  grep -qF "\"productName\": \"$NAME\"" "$DEST/app-client/package.json" || {
    echo "✗ Stamping failed for app-client/package.json — update new-app.sh." >&2
    exit 1
  }
else
  grep -qF "\"name\": \"$SLUG\"" "$DEST/app-client-qml/bato.json" || {
    echo "✗ Stamping failed for app-client-qml/bato.json — update new-app.sh." >&2
    exit 1
  }
fi

# ── Provenance + fresh git history ───────────────────────────────────────────
# The flavor is recorded because it is not otherwise recoverable from a diff against
# the template: half the tree was deleted, and knowing which half was deliberate is
# what makes `git diff <sha>..HEAD` usable for porting template improvements later.
echo "ProjectTemplate v$TPL_VERSION $TPL_SHA (created $(date +%F)) client=$FLAVOR" > "$DEST/.template"

COMMIT_MSG="Initial commit from ProjectTemplate v$TPL_VERSION ($TPL_SHA)"
git -C "$DEST" init -b main -q
git -C "$DEST" add -A
if git -C "$DEST" config --get user.email >/dev/null 2>&1; then
  git -C "$DEST" commit -q -m "$COMMIT_MSG"
  echo "✓ Fresh repo created with a single commit."
else
  echo "⚠ git user.name/user.email not configured — everything is staged but not committed."
  echo "  Configure git, then run:"
  echo "    git -C '$DEST' commit -m '$COMMIT_MSG'"
fi

cat <<SUMMARY

✓ $NAME created at $DEST
  Template: v$TPL_VERSION ($TPL_SHA) — recorded in .template and the initial commit.
  Client:   $FLAVOR
  Version:  $NEW_VERSION
SUMMARY

if [[ "$FLAVOR" == "qml" ]]; then
  cat <<SUMMARY

Next steps:
  cd $DEST
  git remote add origin <your-repo-url>   # whenever you're ready
  make doctor-qml                          # check qml6 and the fonts are present
  make dev-qml                             # host + UI + a bundled server

Publishing the client (see app-client-qml/CLIENT.md):
  • make publish-check-qml   — what would ship, and whether bato would accept it
  • make publish-qml         — then: bato install $SLUG
  • The launcher icon is fetched from the registry, NOT the tarball. Register
    "$SLUG" in BatoApps/bato/icons (ACCENTS + a glyph + distribute.sh CENTRAL),
    then \`make publish-icons\` there — otherwise the launcher has no icon.

Still yours to brand:
  • qml/App/Brand.qml: tagline, accentHex (match the icon colour in
    bato/icons/generate.py ACCENTS)
  • qml/App/assets/icon.svg: your app's mark
  • app-client-qml/identity.go: appId domain com.example.$SLUG → your reverse-DNS
  • LICENSE, README.md
  • Optional: rename the Go modules (app-server, app-client-qml)

Known limitation, not a bug: every QML app launched through the stock qml6
runtime reports the WM class org.qt-project.qml, so two of them running at once
share a taskbar identity. See PARITY.md.
SUMMARY
else
  cat <<SUMMARY
  (package-lock files catch up on the first npm install)

Next steps:
  cd $DEST
  git remote add origin <your-repo-url>   # whenever you're ready
  make dev-setup && make dev-client

Still yours to brand (see README → "Branding"):
  • brand.ts: tagline, accentHex, icon
  • Icons in app-client/build/ (appicon.png, windows/icon.ico)
  • appId domain: com.example.$SLUG → your own reverse-DNS
  • app-client/package.json: author, copyright; LICENSE
  • app-client/package.json build.publish.endpoint (CHANGE-ME IP)
  • README.md: rewrite for your app
SUMMARY
fi

cat <<SUMMARY

Deploying the server (see README → "Deploying the server as a container"):
  • Claim a 42xxx port in BatoApps/PORTS.md and set it as "port" in
    app-server/bato.json — deploy.domain has nothing to route to without it
  • Hostname stamped as $SLUG.bato.lan; change it in app-server/bato.json if
    you want a shorter one. Wildcard DNS + cert already cover it
  • bato secrets set $SLUG-server BATO_AUTH_URL=…   then:  make deploy
SUMMARY
