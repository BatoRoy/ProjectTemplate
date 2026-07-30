#!/usr/bin/env bash
set -euo pipefail

# Scaffold a new app from this template with a FRESH git history.
#
#   ./new-app.sh <AppName> <dest-dir> [--force]
#
# Exports the template's committed HEAD (git archive — ignored/untracked files
# never leak in), stamps the app name/slug into the identity files, resets the
# version to 0.1.0, records template provenance (version + commit) in the
# initial commit message and a .template file, then git-inits the destination
# as a single-commit repo. No template history is carried over.
#
# Linux-oriented: uses GNU `sed -i` (same as version.sh).

usage() {
  echo "Usage: $0 <AppName> <dest-dir> [--force]"
  echo ""
  echo "  AppName    Display name, e.g. MyApp or \"My App\" (letters, digits, space, - _)"
  echo "  dest-dir   Directory to create (must not exist or be empty)"
  echo "  --force    Skip the dirty-template confirmation"
}

NAME="" DEST="" FORCE=0
for arg in "$@"; do
  case "$arg" in
    --force) FORCE=1 ;;
    -h|--help) usage; exit 0 ;;
    *)
      if [[ -z "$NAME" ]]; then NAME="$arg"
      elif [[ -z "$DEST" ]]; then DEST="$arg"
      else echo "Error: unexpected argument '$arg'" >&2; usage >&2; exit 1
      fi ;;
  esac
done

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

echo "→ Exporting template v$TPL_VERSION ($TPL_SHA) → $DEST"
git -C "$TEMPLATE_ROOT" archive HEAD | tar -x -C "$DEST"
rm -f "$DEST/new-app.sh"   # an app doesn't create apps

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
stamp app-server/internal/config/config.go \
  "s|configDir = \".config/app\"|configDir = \".config/$SLUG\"|" "configDir = \".config/$SLUG\""

# JSON files are edited with node (guaranteed present for this stack) instead
# of sed, so structure changes can't corrupt them.
node - "$DEST" "$NAME" "$SLUG" "$EXEC_NAME" "$NEW_VERSION" <<'EOF'
const fs = require('fs')
const path = require('path')
const [dest, name, slug, execName, version] = process.argv.slice(2)

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
  p.deploy.app = `${execName}Server`
  p.deploy.image = `${slug}-server`
  p.deploy.domain = `${slug}.bato.lan`
  p.deploy.volumes = [{ name: `${slug}-server-data`, path: '/data' }]
})
EOF
grep -qF "\"productName\": \"$NAME\"" "$DEST/app-client/package.json" || {
  echo "✗ Stamping failed for app-client/package.json — update new-app.sh." >&2
  exit 1
}

# ── Provenance + fresh git history ───────────────────────────────────────────
echo "ProjectTemplate v$TPL_VERSION $TPL_SHA (created $(date +%F))" > "$DEST/.template"

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
  Version:  $NEW_VERSION (package-lock files catch up on the first npm install)

Next steps:
  cd $DEST
  git remote add origin <your-repo-url>   # whenever you're ready
  make dev-setup && make dev-client

Deploying the server (see README → "Deploying the server as a container"):
  • Claim a 42xxx port in BatoApps/PORTS.md and set it as "port" in
    app-server/bato.json — deploy.domain has nothing to route to without it
  • Hostname stamped as $SLUG.bato.lan; change it in app-server/bato.json if
    you want a shorter one. Wildcard DNS + cert already cover it
  • bato secrets set $SLUG-server BATO_AUTH_URL=…   then:  make deploy

Still yours to brand (see README → "Branding"):
  • brand.ts: tagline, accentHex, icon
  • Icons in app-client/build/ (appicon.png, windows/icon.ico)
  • appId domain: com.example.$SLUG → your own reverse-DNS
  • app-client/package.json: author, copyright; LICENSE
  • app-client/package.json build.publish.endpoint (CHANGE-ME IP)
  • README.md: rewrite for your app
  • Optional: rename the app-server go module (go.mod + internal imports)
SUMMARY
