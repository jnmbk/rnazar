#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CARGO_TOML="${ROOT_DIR}/Cargo.toml"

usage() {
    cat <<EOF
Usage: $(basename "$0") <version|patch|minor|major> [--git]

Options:
  <version>       Explicit version (e.g. 0.8.3)
  patch           Increment patch version (0.8.2 -> 0.8.3)
  minor           Increment minor version (0.8.2 -> 0.9.0)
  major           Increment major version (0.8.2 -> 1.0.0)
  --git, -g       Commit the changes and create a git tag (v<VERSION>)

Example:
  $(basename "$0") patch
  $(basename "$0") 0.8.3 --git
EOF
    exit 1
}

if [ $# -lt 1 ]; then
    usage
fi

TARGET="$1"
DO_GIT=false

if [ "${2:-}" = "--git" ] || [ "${2:-}" = "-g" ]; then
    DO_GIT=true
fi

# 1. Read current version from Cargo.toml
CURRENT_VERSION="$(grep '^version' "${CARGO_TOML}" | head -n1 | cut -d '"' -f2)"
if [ -z "${CURRENT_VERSION}" ]; then
    echo "Error: Could not determine current version from ${CARGO_TOML}" >&2
    exit 1
fi

echo "Current version: ${CURRENT_VERSION}"

# 2. Determine target version
IFS='.' read -r MAJOR MINOR PATCH <<< "${CURRENT_VERSION}"

case "${TARGET}" in
    patch)
        NEW_VERSION="${MAJOR}.${MINOR}.$((PATCH + 1))"
        ;;
    minor)
        NEW_VERSION="${MAJOR}.$((MINOR + 1)).0"
        ;;
    major)
        NEW_VERSION="$((MAJOR + 1)).0.0"
        ;;
    *)
        # Strip leading 'v' if provided
        NEW_VERSION="${TARGET#v}"
        if ! [[ "${NEW_VERSION}" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$ ]]; then
            echo "Error: '${NEW_VERSION}' is not a valid semantic version (expected X.Y.Z)." >&2
            exit 1
        fi
        ;;
esac

echo "Bumping version: ${CURRENT_VERSION} -> ${NEW_VERSION}"

# 3. Update Cargo.toml
sed -i.bak -E "s/^(version = \")[^\"]+(\")/\1${NEW_VERSION}\2/" "${CARGO_TOML}"
rm -f "${CARGO_TOML}.bak"

# 4. Update Cargo.lock
echo "==> Updating Cargo.lock..."
(cd "${ROOT_DIR}" && cargo check --quiet)

# 5. Update packaging/arch/PKGBUILD if present
PKGBUILD="${ROOT_DIR}/packaging/arch/PKGBUILD"
if [ -f "${PKGBUILD}" ]; then
    echo "==> Updating packaging/arch/PKGBUILD..."
    sed -i.bak -E "s/^pkgver=.*/pkgver=${NEW_VERSION}/" "${PKGBUILD}"
    sed -i.bak -E "s/^pkgrel=.*/pkgrel=1/" "${PKGBUILD}"
    rm -f "${PKGBUILD}.bak"
fi

# 6. Update packaging/windows/setup.iss if present
SETUP_ISS="${ROOT_DIR}/packaging/windows/setup.iss"
if [ -f "${SETUP_ISS}" ]; then
    echo "==> Updating packaging/windows/setup.iss..."
    sed -i.bak -E "s/(#define MyAppVersion \")[^\"]+(\")/\1${NEW_VERSION}\2/" "${SETUP_ISS}"
    rm -f "${SETUP_ISS}.bak"
fi

# 7. Update packaging/macos/Info.plist if present
PLIST="${ROOT_DIR}/packaging/macos/Info.plist"
if [ -f "${PLIST}" ]; then
    echo "==> Updating packaging/macos/Info.plist..."
    sed -i.bak -E "/CFBundleShortVersionString/ {n; s#<string>[^<]+</string>#<string>${NEW_VERSION}</string>#;}" "${PLIST}"
    rm -f "${PLIST}.bak"
fi

# 8. Git commit & tag if requested
if [ "${DO_GIT}" = true ]; then
    echo "==> Creating git commit and tag..."
    git -C "${ROOT_DIR}" add "${CARGO_TOML}" "${ROOT_DIR}/Cargo.lock"
    [ -f "${PKGBUILD}" ] && git -C "${ROOT_DIR}" add "${PKGBUILD}"
    [ -f "${SETUP_ISS}" ] && git -C "${ROOT_DIR}" add "${SETUP_ISS}"
    [ -f "${PLIST}" ] && git -C "${ROOT_DIR}" add "${PLIST}"
    git -C "${ROOT_DIR}" commit -m "chore: bump version to ${NEW_VERSION}"
    git -C "${ROOT_DIR}" tag -a "v${NEW_VERSION}" -m "Release v${NEW_VERSION}"
    echo "Successfully committed and tagged v${NEW_VERSION}!"
    echo "Run 'git push origin main --tags' to publish the release."
else
    echo ""
    echo "Version bump complete. To review and commit manually:"
    echo "  git diff"
    echo "  git commit -am 'chore: bump version to ${NEW_VERSION}'"
    echo "  git tag -a v${NEW_VERSION} -m 'Release v${NEW_VERSION}'"
fi
