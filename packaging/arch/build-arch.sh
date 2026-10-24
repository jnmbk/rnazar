#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUTPUT_DIR="${ROOT_DIR}/target/arch"
BUILD_DIR="${OUTPUT_DIR}/build"

VERSION="${RNAZAR_VERSION:-$(grep '^version' "${ROOT_DIR}/Cargo.toml" | head -n1 | cut -d '"' -f2 || echo "0.1.0")}"
VERSION="${VERSION#v}"

echo "==> Building rnazar release binary..."
cargo build --release --manifest-path "${ROOT_DIR}/Cargo.toml"

echo "==> Preparing Arch package build..."
rm -rf "${BUILD_DIR}"
mkdir -p "${BUILD_DIR}/pkg"
mkdir -p "${OUTPUT_DIR}"

PKGDIR="${BUILD_DIR}/pkg"

# Binary
install -s -Dm755 "${ROOT_DIR}/target/release/rnazar" "${PKGDIR}/usr/bin/rnazar"

# Desktop & DBus
install -Dm644 "${ROOT_DIR}/assets/rnazar.desktop" "${PKGDIR}/usr/share/applications/rnazar.desktop"
install -Dm644 "${ROOT_DIR}/assets/dbus/org.rnazar.Notifier.service" "${PKGDIR}/usr/share/dbus-1/services/org.rnazar.Notifier.service"
install -Dm644 "${ROOT_DIR}/LICENSE" "${PKGDIR}/usr/share/licenses/rnazar/LICENSE"

# Icons
install -Dm644 "${ROOT_DIR}/assets/icons/nazar.svg" "${PKGDIR}/usr/share/icons/hicolor/scalable/apps/rnazar.svg"
for size in 16x16 32x32 48x48 64x64 128x128 256x256 512x512; do
    if [ -f "${ROOT_DIR}/assets/icons/png/${size}.png" ]; then
        install -Dm644 "${ROOT_DIR}/assets/icons/png/${size}.png" "${PKGDIR}/usr/share/icons/hicolor/${size}/apps/rnazar.png"
    fi
done

PKG_FILE="${OUTPUT_DIR}/rnazar-${VERSION}-1-x86_64.pkg.tar.zst"

echo "==> Creating Arch Linux package archive..."
# Generate .PKGINFO
BUILDDATE=$(date -u +%s)
SIZE=$(du -sb "${PKGDIR}" | cut -f1)

cat <<EOF > "${PKGDIR}/.PKGINFO"
pkgname = rnazar
pkgver = ${VERSION}-1
pkgdesc = System tray evil eye ward and crash notifier
url = https://github.com/jnmbk/rnazar
builddate = ${BUILDDATE}
packager = jnmbk <ugur.jnmbk@gmail.com>
size = ${SIZE}
arch = x86_64
license = MIT
license = Apache-2.0
depend = gtk3
EOF

# Package into .pkg.tar.zst with reproducible settings
(
    cd "${PKGDIR}"
    tar -I 'zstd -19 -T0' -cf "${PKG_FILE}" .PKGINFO *
)

echo "==> Successfully created Arch package at ${PKG_FILE}!"

