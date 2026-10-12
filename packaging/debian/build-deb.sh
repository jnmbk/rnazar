#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUTPUT_DIR="${ROOT_DIR}/target/debian"
BUILD_DIR="${OUTPUT_DIR}/build"

VERSION="${RNAZAR_VERSION:-$(grep '^version' "${ROOT_DIR}/Cargo.toml" | head -n1 | cut -d '"' -f2 || echo "0.1.0")}"
VERSION="${VERSION#v}"
ARCH="amd64"

echo "==> Building rnazar release binary..."
cargo build --release --manifest-path "${ROOT_DIR}/Cargo.toml"

echo "==> Staging Debian package root..."
rm -rf "${BUILD_DIR}"
mkdir -p "${BUILD_DIR}/DEBIAN"
mkdir -p "${BUILD_DIR}/usr/bin"
mkdir -p "${BUILD_DIR}/usr/share/applications"
mkdir -p "${BUILD_DIR}/usr/share/icons/hicolor/scalable/apps"
mkdir -p "${BUILD_DIR}/usr/share/dbus-1/services"
mkdir -p "${BUILD_DIR}/usr/share/licenses/rnazar"
mkdir -p "${OUTPUT_DIR}"

# Binary
install -m 755 "${ROOT_DIR}/target/release/rnazar" "${BUILD_DIR}/usr/bin/rnazar"

# Desktop & DBus
install -m 644 "${ROOT_DIR}/assets/rnazar.desktop" "${BUILD_DIR}/usr/share/applications/rnazar.desktop"
install -m 644 "${ROOT_DIR}/assets/dbus/org.rnazar.Notifier.service" "${BUILD_DIR}/usr/share/dbus-1/services/org.rnazar.Notifier.service"
install -m 644 "${ROOT_DIR}/LICENSE" "${BUILD_DIR}/usr/share/licenses/rnazar/LICENSE"

# Icons
install -m 644 "${ROOT_DIR}/assets/icons/nazar.svg" "${BUILD_DIR}/usr/share/icons/hicolor/scalable/apps/rnazar.svg"
for size in 16x16 32x32 48x48 64x64 128x128 256x256 512x512; do
    if [ -f "${ROOT_DIR}/assets/icons/png/${size}.png" ]; then
        mkdir -p "${BUILD_DIR}/usr/share/icons/hicolor/${size}/apps"
        install -m 644 "${ROOT_DIR}/assets/icons/png/${size}.png" "${BUILD_DIR}/usr/share/icons/hicolor/${size}/apps/rnazar.png"
    fi
done

# Calculate installed size in KB
INSTALLED_SIZE=$(du -sk "${BUILD_DIR}/usr" | cut -f1)

# Generate control file
sed -e "s/@PACKAGE@/rnazar/g" \
    -e "s/@VERSION@/${VERSION}/g" \
    -e "s/@ARCH@/${ARCH}/g" \
    -e "s/@INSTALLED_SIZE@/${INSTALLED_SIZE}/g" \
    "${ROOT_DIR}/packaging/debian/control.in" > "${BUILD_DIR}/DEBIAN/control"

# Maintainer scripts
install -m 755 "${ROOT_DIR}/packaging/debian/postinst" "${BUILD_DIR}/DEBIAN/postinst"
install -m 755 "${ROOT_DIR}/packaging/debian/postrm" "${BUILD_DIR}/DEBIAN/postrm"

DEB_FILE="${OUTPUT_DIR}/rnazar_${VERSION}_${ARCH}.deb"

echo "==> Building Debian package at ${DEB_FILE}..."
if command -v dpkg-deb >/dev/null 2>&1; then
    dpkg-deb --build --root-owner-group "${BUILD_DIR}" "${DEB_FILE}"
else
    # Fallback to standard ar / tar format if dpkg-deb is not installed
    echo "dpkg-deb not found, assembling .deb archive via tar and ar..."
    (
        cd "${BUILD_DIR}"
        echo "2.0" > debian-binary
        tar --numeric-owner --group=0 --owner=0 -czf control.tar.gz -C DEBIAN .
        tar --numeric-owner --group=0 --owner=0 -czf data.tar.gz ./usr
        ar rcs "${DEB_FILE}" debian-binary control.tar.gz data.tar.gz
    )
fi

echo "==> Successfully created ${DEB_FILE}!"

