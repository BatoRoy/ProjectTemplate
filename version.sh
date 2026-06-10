#!/usr/bin/env bash
set -euo pipefail

VERSION_FILE="VERSION"
JS_VERSION_FILE="app-client/frontend/src/lib/version.ts"
GO_VERSION_FILE="app-server/version.go"
PACKAGE_JSON="app-client/package.json"
FRONTEND_PACKAGE_JSON="app-client/frontend/package.json"
PACKAGE_LOCK="app-client/package-lock.json"
FRONTEND_PACKAGE_LOCK="app-client/frontend/package-lock.json"

# Read current version from VERSION file
current_version() {
  tr -d '[:space:]' < "$VERSION_FILE"
}

sync_files() {
  local ver="$1"
  sed -i "s/export const VERSION = '[0-9]*\.[0-9]*\.[0-9]*'/export const VERSION = '$ver'/" "$JS_VERSION_FILE"
  sed -i "s/var Version = \"[0-9]*\.[0-9]*\.[0-9]*\"/var Version = \"$ver\"/" "$GO_VERSION_FILE"
  npm version "$ver" --no-git-tag-version --prefix app-client > /dev/null
  npm version "$ver" --no-git-tag-version --prefix app-client/frontend > /dev/null
}

usage() {
  echo "Usage: $0 [bump major|minor|patch | set X.Y.Z]"
  echo ""
  echo "  (no args)           Show current version"
  echo "  bump major          Increment major version (1.2.3 → 2.0.0)"
  echo "  bump minor          Increment minor version (1.2.3 → 1.3.0)"
  echo "  bump patch          Increment patch version (1.2.3 → 1.2.4)"
  echo "  set X.Y.Z           Set an explicit version"
}

# Must be run from the repo root
if [[ ! -f "$VERSION_FILE" ]]; then
  echo "Error: run this script from the repository root" >&2
  exit 1
fi

if [[ $# -eq 0 ]]; then
  current_version
  exit 0
fi

cmd="$1"

case "$cmd" in
  bump)
    if [[ $# -lt 2 ]]; then
      echo "Error: bump requires major, minor, or patch" >&2
      exit 1
    fi
    part="$2"
    IFS='.' read -r major minor patch <<< "$(current_version)"
    case "$part" in
      major) major=$((major + 1)); minor=0; patch=0 ;;
      minor) minor=$((minor + 1)); patch=0 ;;
      patch) patch=$((patch + 1)) ;;
      *) echo "Error: unknown bump type '$part' (use major, minor, or patch)" >&2; exit 1 ;;
    esac
    new_version="${major}.${minor}.${patch}"
    ;;
  set)
    if [[ $# -lt 2 ]]; then
      echo "Error: set requires a version argument" >&2
      exit 1
    fi
    new_version="$2"
    ;;
  help|--help|-h)
    usage
    exit 0
    ;;
  *)
    echo "Error: unknown command '$cmd'" >&2
    usage >&2
    exit 1
    ;;
esac

# Validate semver
if ! [[ "$new_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "Error: version must be in X.Y.Z format" >&2
  exit 1
fi

# Refuse to clobber an existing tag
if git rev-parse --verify --quiet "refs/tags/v${new_version}" >/dev/null; then
  echo "Error: tag v${new_version} already exists" >&2
  exit 1
fi

# Warn if not on main (release pipeline only triggers on tag pushes, but tags
# should still come from main for a clean history)
current_branch="$(git rev-parse --abbrev-ref HEAD)"
if [[ "$current_branch" != "main" ]]; then
  echo "Warning: you are on branch '$current_branch', not 'main'." >&2
fi

# Write VERSION file
echo "$new_version" > "$VERSION_FILE"

# Sync dependent files
sync_files "$new_version"

echo "Version set to $new_version"

# Stage version files plus the lockfiles npm version rewrites
git add "$VERSION_FILE" "$JS_VERSION_FILE" "$GO_VERSION_FILE" \
        "$PACKAGE_JSON" "$FRONTEND_PACKAGE_JSON" \
        "$PACKAGE_LOCK" "$FRONTEND_PACKAGE_LOCK"

# Commit
git commit -m "chore: bump version to $new_version"

# Tag
git tag "v${new_version}"

echo ""
echo "Tagged v${new_version}. Push with:"
echo "  git push && git push origin v${new_version}"
