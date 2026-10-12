#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
APPDIR="${ROOT_DIR}/AppDir"
OUTPUT_DIR="${ROOT_DIR}/target/appimage"

echo "==> Building rnazar release binary..."
cargo build --release --manifest-path "${ROOT_DIR}/Cargo.toml"

echo "==> Preparing AppDir at ${APPDIR}..."
rm -rf "${APPDIR}"
mkdir -p "${APPDIR}/usr/bin"
mkdir -p "${APPDIR}/usr/share/applications"
mkdir -p "${APPDIR}/usr/share/icons/hicolor/scalable/apps"
mkdir -p "${APPDIR}/usr/share/icons/hicolor/256x256/apps"
mkdir -p "${APPDIR}/usr/share/dbus-1/services"
mkdir -p "${OUTPUT_DIR}"

# Copy binary
cp "${ROOT_DIR}/target/release/rnazar" "${APPDIR}/usr/bin/rnazar"

# Copy AppRun
cp "${ROOT_DIR}/packaging/appimage/AppRun" "${APPDIR}/AppRun"
chmod +x "${APPDIR}/AppRun"

# Copy desktop and icon files
cp "${ROOT_DIR}/assets/rnazar.desktop" "${APPDIR}/rnazar.desktop"
cp "${ROOT_DIR}/assets/rnazar.desktop" "${APPDIR}/usr/share/applications/rnazar.desktop"
cp "${ROOT_DIR}/assets/icons/png/256x256.png" "${APPDIR}/rnazar.png"
ln -sf rnazar.png "${APPDIR}/.DirIcon"

cp "${ROOT_DIR}/assets/icons/nazar.svg" "${APPDIR}/usr/share/icons/hicolor/scalable/apps/rnazar.svg"
cp "${ROOT_DIR}/assets/icons/png/256x256.png" "${APPDIR}/usr/share/icons/hicolor/256x256/apps/rnazar.png"
cp "${ROOT_DIR}/assets/dbus/org.rnazar.Notifier.service" "${APPDIR}/usr/share/dbus-1/services/org.rnazar.Notifier.service"

echo "==> Checking for appimagetool..."
if ! command -v appimagetool &> /dev/null; then
    APPIMAGETOOL="${OUTPUT_DIR}/appimagetool"
    if [ ! -f "${APPIMAGETOOL}" ]; then
        echo "Downloading appimagetool..."
        curl -fsSL -o "${APPIMAGETOOL}" "https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage"
        chmod +x "${APPIMAGETOOL}"
    fi
else
    APPIMAGETOOL="appimagetool"
fi

VERSION="$(grep '^version' "${ROOT_DIR}/Cargo.toml" | head -n1 | cut -d '"' -f2 || echo "0.1.0")"
OUTPUT_FILE="${OUTPUT_DIR}/rnazar-${VERSION}-x86_64.AppImage"

echo "==> Creating AppImage at ${OUTPUT_FILE}..."
ARCH=x86_64 "${APPIMAGETOOL}" "${APPDIR}" "${OUTPUT_FILE}"

echo "==> Successfully created ${OUTPUT_FILE}!"

