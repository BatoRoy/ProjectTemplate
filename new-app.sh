#!/usr/bin/env bash
set -euo pipefail

# Scaffold a new app from this template with a FRESH git history.
#
#   ./new-app.sh <AppName> <dest-dir> [--electron|--native] [--force]
#
# Exports the template's committed HEAD (git archive — ignored/untracked files
# never leak in), stamps the app name/slug into the identity files, resets the
# version to 0.1.0, records template provenance (version + commit) in the
# initial commit message and a .template file, then git-inits the destination
# as a single-commit repo. No template history is carried over.
#
# Linux-oriented: uses GNU `sed -i` (same as version.sh).

usage() {
  echo "Usage: $0 <AppName> <dest-dir> [--electron|--native] [--force]"
  echo ""
  echo "  AppName      Display name, e.g. MyApp or \"My App\" (letters, digits, space, - _)"
  echo "  dest-dir     Directory to create (must not exist or be empty)"
  echo "  --electron   Electron + React client (the default)"
  echo "  --native     Native Rust (iced) client on bato-native: one process, about a third"
  echo "               of the memory, no web bundle, no bato-hub, Linux only"
  echo "  --force      Skip the dirty-template confirmation"
  echo ""
  echo "The client flavor is recorded in .template and drives which client directory the"
  echo "new app keeps, which identity files are stamped and which release workflow survives."
}

NAME="" DEST="" FORCE=0 FLAVOR=""
for arg in "$@"; do
  case "$arg" in
    --force) FORCE=1 ;;
    --electron) FLAVOR="electron" ;;
    --native) FLAVOR="native" ;;
    -h|--help) usage; exit 0 ;;
    # Caught here rather than falling through to the positional branch, where an
    # unknown flag becomes the app name and fails the name regex instead.
    --*) echo "Error: unknown option '$arg'" >&2; usage >&2; exit 1 ;;
    *)
      if [[ -z "$NAME" ]]; then NAME="$arg"
      elif [[ -z "$DEST" ]]; then DEST="$arg"
      else echo "Error: unexpected argument '$arg'" >&2; usage >&2; exit 1
      fi ;;
  esac
done

# Electron by default. Everything per-flavor below keys off this variable.
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
# place, but an app wants exactly one: two means two things to keep working and an
# ambiguous `make dev-client`. The Makefile detects which one is present, so deleting
# the other is all that is needed — nothing to configure.
if [[ "$FLAVOR" == "native" ]]; then
  rm -rf "$DEST/app-client"
  # Electron-only documents: electron-updater, the AppImage server bundling (the native
  # supervisor is in bato-shell, CLIENT.md), the React restyle port, and the
  # Electron-driven contrast check (the native palette maths is tested in bato-native).
  rm -f "$DEST/AUTOUPDATE.md" "$DEST/BUNDLED-SERVICES.md" "$DEST/STYLE-MIGRATION.md"
  rm -f "$DEST/tools/check-contrast.mjs"
else
  rm -rf "$DEST/app-client-native"
  # The native block at the end of .gitignore, then the blank line before it.
  sed -i '/^# ── Native client/,$d' "$DEST/.gitignore"
  sed -i -e :a -e '/^\n*$/{$d;N;ba' -e '}' "$DEST/.gitignore"
fi

# The release workflow is named per flavor, and the one that does not match the
# client has to go.
#
# BatoAI is the cautionary tale: it swapped its renderer and left the template's
# Electron workflow in place, so pushing a version tag has failed there ever since —
# CI still runs `npm ci` in a frontend directory that no longer exists. That is why
# `.github/workflows/` is pruned by flavor rather than left whole, even with one
# workflow in it.
find "$DEST/.github/workflows" -name 'release-*.yml' \
  ! -name "release-$FLAVOR.yml" -delete 2>/dev/null || true

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

# Per-flavor, because each client keeps its identity in its own files.
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
  # brand.rs is the one identity file the code reads (window title, app_id, config
  # dir, registry name, About). The [[bin]] name is the binary's file name; the Cargo
  # package (`app-client`) is deliberately not renamed, so Cargo.lock stays valid and
  # `--locked` builds keep working. bato.json is stamped with node below, and
  # `cargo test` checks all three agree.
  stamp app-client-native/src/brand.rs \
    "s|    \"App\", // name: stamped by new-app.sh|    \"$NAME\", // name: stamped by new-app.sh|" \
    "    \"$NAME\", // name: stamped by new-app.sh"
  stamp app-client-native/src/brand.rs \
    "s|    \"app\", // slug: stamped by new-app.sh|    \"$SLUG\", // slug: stamped by new-app.sh|" \
    "    \"$SLUG\", // slug: stamped by new-app.sh"
  stamp app-client-native/Cargo.toml \
    "s|^name = \"app\" # bin: stamped by new-app.sh|name = \"$SLUG\" # bin: stamped by new-app.sh|" \
    "name = \"$SLUG\" # bin: stamped by new-app.sh"
fi

stamp app-server/internal/config/config.go \
  "s|configDir = \".config/app\"|configDir = \".config/$SLUG\"|" "configDir = \".config/$SLUG\""
# The .env lookup is a separate string from configDir (LoadEnvFiles joins it
# onto os.UserConfigDir), and it went unstamped for every app through 0.4.1:
# each one read ~/.config/app/.env, so two apps would share one secrets file.
stamp app-server/main.go \
  "s|config.LoadEnvFiles(\"app\")|config.LoadEnvFiles(\"$SLUG\")|" "config.LoadEnvFiles(\"$SLUG\")"
stamp app-server/.env.example \
  "s|~/.config/app/.env|~/.config/$SLUG/.env|" "~/.config/$SLUG/.env"

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
// The native client publishes as type "desktop". iced sets the Wayland app_id and
// the X11 WM_CLASS from the slug (brand.rs), so startupWMClass is the slug too —
// measured with hyprctl clients, not guessed.
edit('app-client-native/bato.json', (p) => {
  p.name = slug
  p.description = `Desktop client for ${name}`
  p.artifacts = p.artifacts.map((a) => (a === 'app' ? slug : a))
  p.bin = [{ name: slug, target: slug }]
  p.desktop = { ...p.desktop, name, exec: slug, startupWMClass: slug }
})
// BatoGit Ship's mapping for this layout, for an app that needs a ship.json.
if (flavor === 'native') {
  edit('ship.example.json', (p) => {
    p.targets['publish-client'] = { paths: ['app-client-native/'], registry: slug }
  })
}
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
  grep -qF "\"startupWMClass\": \"$SLUG\"" "$DEST/app-client-native/bato.json" || {
    echo "✗ Stamping failed for app-client-native/bato.json — update new-app.sh." >&2
    exit 1
  }
fi

# ── README: make it the app's, not the template's ────────────────────────────
# The template's README compares the two clients and documents both. That is the right
# thing for the template and noise in an app, which has exactly one client: the
# comparison is replaced with one line naming it, and the per-flavor blocks
# (`<!-- electron -->…<!-- /electron -->`, `<!-- native -->…`) are resolved: the other
# flavor's are dropped, this one's lose their markers. `<!-- template -->` blocks
# (headings that only make sense with both) go in every app.
#
# node rather than sed: the section is multi-line and node is already required (the
# Makefile reads bato.json with it).
node - "$DEST" "$NAME" "$FLAVOR" <<'EOF'
const fs = require('fs')
const path = require('path')
const [dest, name, flavor] = process.argv.slice(2)

const file = path.join(dest, 'README.md')
if (!fs.existsSync(file)) process.exit(0)
let md = fs.readFileSync(file, 'utf8')

const names = {
  electron: '**Electron + React** client (`app-client/`)',
  native: '**native Rust (iced)** client (`app-client-native/`), built on\n' +
    "[bato-native](https://github.com/BatoRoy/bato-native)'s kit and shell. See\n" +
    '`app-client-native/CLIENT.md`',
}

const start = md.indexOf('## Clients')
const end = md.indexOf('## Stack')
if (start !== -1 && end > start) {
  md = md.slice(0, start) +
    `## Client\n\n${name} uses the ${names[flavor] ?? flavor}.\n\n` +
    md.slice(end)
}

// Per-flavor blocks.
const other = flavor === 'native' ? 'electron' : 'native'
for (const tag of [other, 'template']) {
  md = md.replace(new RegExp(`<!-- ${tag} -->\\n[\\s\\S]*?<!-- /${tag} -->\\n`, 'g'), '')
}
md = md.replace(new RegExp(`<!-- /?${flavor} -->\\n`, 'g'), '')

// Stack rows for the client this app does not have.
const dropRows = flavor === 'native' ? ['| Frontend ', '| Desktop shell '] : ['| Native client ']
md = md.split('\n').filter((l) => !dropRows.some((r) => l.startsWith(r))).join('\n')

// The opening sentence describes a template, not an app.
md = md.replace(
  'A starting template for desktop apps: a **Go** backend, a choice of two clients that look\n' +
  'the same — **Electron + React** or **native Rust (iced)** — a polished theming system, and a\n' +
  'version-tagged build/release pipeline.',
  flavor === 'native'
    ? `${name} — a **Go** backend, a **native Rust (iced)** client, a polished theming\n` +
      'system, and a version-tagged build/release pipeline.'
    : `${name} — a **Go** backend, an **Electron + React** client, a polished theming\n` +
      'system, and a version-tagged build/release pipeline.')
if (flavor === 'native') {
  // "Look & contrast" is an Electron section.
  md = md.replace(' (see [Look & contrast](#look--contrast))', '')
}

fs.writeFileSync(file, md)
EOF

# ── Provenance + fresh git history ───────────────────────────────────────────
# The flavor is recorded because it is not otherwise recoverable from a diff against the
# template once an app starts changing its client, and knowing which differences were
# deliberate is what makes `git diff <sha>..HEAD` usable for porting template
# improvements later.
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

if [[ "$FLAVOR" == "native" ]]; then
  cat <<SUMMARY

Next steps:
  cd $DEST
  git remote add origin <your-repo-url>   # whenever you're ready
  make dev-setup && make dev-client       # first build: a few minutes

Publishing the client (see app-client-native/CLIENT.md):
  • make publish-check    — what would ship, and whether bato would accept it
  • make publish-client   — then: bato install $SLUG
  • The launcher icon is fetched from the registry, NOT the release. Register
    "$SLUG" in BatoApps/bato/icons (ACCENTS + a glyph + distribute.sh CENTRAL),
    then \`make publish-icons\` there — otherwise the launcher has no icon.

Still yours to brand (see README → "Branding"):
  • app-client-native/src/brand.rs: tagline, accent (match the icon colour in
    bato/icons/generate.py ACCENTS), icon
  • LICENSE, README.md
SUMMARY
fi

if [[ "$FLAVOR" == "electron" ]]; then
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

Mirroring to the local Forgejo (git.bato.lan — bato/docs/todo/LOCAL-GIT-HOST.md):
  • Once this repo has its GitHub origin, run
      /home/batoroy/Code/BatoApps/bato/scripts/git-dual-remote.sh $DEST
    so every \`git push\` lands on both hosts. gitsync imports the repo on its
    next scan either way; the script just makes your pushes instant on both
SUMMARY
