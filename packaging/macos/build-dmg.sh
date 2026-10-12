#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
APP_BUNDLE="${ROOT_DIR}/target/release/bundle/osx/rnazar.app"
OUTPUT_DIR="${ROOT_DIR}/target/macos"

echo "==> Building rnazar release binary..."
cargo build --release --manifest-path "${ROOT_DIR}/Cargo.toml"

echo "==> Creating macOS App Bundle..."
rm -rf "${APP_BUNDLE}"
mkdir -p "${APP_BUNDLE}/Contents/MacOS"
mkdir -p "${APP_BUNDLE}/Contents/Resources"
mkdir -p "${OUTPUT_DIR}"

cp "${ROOT_DIR}/target/release/rnazar" "${APP_BUNDLE}/Contents/MacOS/rnazar"
cp "${ROOT_DIR}/packaging/macos/Info.plist" "${APP_BUNDLE}/Contents/Info.plist"
cp "${ROOT_DIR}/assets/icons/rnazar.icns" "${APP_BUNDLE}/Contents/Resources/rnazar.icns"

VERSION="${RNAZAR_VERSION:-$(grep '^version' "${ROOT_DIR}/Cargo.toml" | head -n1 | cut -d '"' -f2 || echo "0.1.0")}"
VERSION="${VERSION#v}"
DMG_PATH="${OUTPUT_DIR}/rnazar-v${VERSION}-macos-universal.dmg"

echo "==> Packaging DMG at ${DMG_PATH}..."
if command -v hdiutil &> /dev/null; then
    hdiutil create -volname "rnazar" -srcfolder "${APP_BUNDLE}" -ov -format UDZO "${DMG_PATH}"
    echo "==> Successfully created ${DMG_PATH}!"
else
    # Fallback to tarball/zip if hdiutil is not available (e.g. cross-packaging environment)
    tar -czf "${OUTPUT_DIR}/rnazar-v${VERSION}-macos.app.tar.gz" -C "$(dirname "${APP_BUNDLE}")" "rnazar.app"
    echo "==> Created fallback tarball at ${OUTPUT_DIR}/rnazar-v${VERSION}-macos.app.tar.gz"
fi

