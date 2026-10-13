#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
export PATH="${HOME}/.cargo/bin:${HOME}/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/opt/homebrew/bin:/usr/local/bin:${PATH}"

APP_BUNDLE="${ROOT_DIR}/target/release/bundle/osx/rnazar.app"
OUTPUT_DIR="${ROOT_DIR}/target/macos"
VERSION="${RNAZAR_VERSION:-$(grep '^version' "${ROOT_DIR}/Cargo.toml" | head -n1 | cut -d '"' -f2 || echo "0.1.0")}"
VERSION="${VERSION#v}"

echo "=================================================="
echo " Packaging rnazar v${VERSION} for macOS"
echo "=================================================="

# Check if universal binary can be built
TARGET_AARCH64="aarch64-apple-darwin"
TARGET_X86_64="x86_64-apple-darwin"
INSTALLED_TARGETS="$(rustup target list --installed 2>/dev/null || true)"

CAN_BUILD_UNIVERSAL=false
if echo "${INSTALLED_TARGETS}" | grep -q "${TARGET_AARCH64}" && echo "${INSTALLED_TARGETS}" | grep -q "${TARGET_X86_64}"; then
    CAN_BUILD_UNIVERSAL=true
fi

if [ "${FORCE_SINGLE_ARCH:-0}" = "1" ]; then
    CAN_BUILD_UNIVERSAL=false
fi

if [ "${CAN_BUILD_UNIVERSAL}" = "true" ]; then
    echo "==> Building universal binary (${TARGET_AARCH64} + ${TARGET_X86_64})..."
    cargo build --release --target "${TARGET_AARCH64}" --manifest-path "${ROOT_DIR}/Cargo.toml"
    cargo build --release --target "${TARGET_X86_64}" --manifest-path "${ROOT_DIR}/Cargo.toml"

    UNIVERSAL_DIR="${ROOT_DIR}/target/universal/release"
    mkdir -p "${UNIVERSAL_DIR}"
    lipo -create \
        "${ROOT_DIR}/target/${TARGET_AARCH64}/release/rnazar" \
        "${ROOT_DIR}/target/${TARGET_X86_64}/release/rnazar" \
        -output "${UNIVERSAL_DIR}/rnazar"
    BINARY_PATH="${UNIVERSAL_DIR}/rnazar"
    ARCH_SUFFIX="universal"
else
    echo "==> Building host-native release binary..."
    cargo build --release --manifest-path "${ROOT_DIR}/Cargo.toml"
    BINARY_PATH="${ROOT_DIR}/target/release/rnazar"
    HOST_ARCH="$(uname -m)"
    ARCH_SUFFIX="${HOST_ARCH}"
fi

echo "==> Binary inspection:"
lipo -info "${BINARY_PATH}"

echo "==> Assembling macOS App Bundle..."
rm -rf "${APP_BUNDLE}"
mkdir -p "${APP_BUNDLE}/Contents/MacOS"
mkdir -p "${APP_BUNDLE}/Contents/Resources"
mkdir -p "${OUTPUT_DIR}"

cp "${BINARY_PATH}" "${APP_BUNDLE}/Contents/MacOS/rnazar"
if command -v strip &>/dev/null; then
    strip "${APP_BUNDLE}/Contents/MacOS/rnazar" 2>/dev/null || true
fi
chmod +x "${APP_BUNDLE}/Contents/MacOS/rnazar"
cp "${ROOT_DIR}/packaging/macos/Info.plist" "${APP_BUNDLE}/Contents/Info.plist"

if command -v plutil &>/dev/null; then
    plutil -replace CFBundleShortVersionString -string "${VERSION}" "${APP_BUNDLE}/Contents/Info.plist"
    plutil -replace CFBundleVersion -string "${VERSION}" "${APP_BUNDLE}/Contents/Info.plist"
fi

cp "${ROOT_DIR}/assets/icons/rnazar.icns" "${APP_BUNDLE}/Contents/Resources/rnazar.icns"

echo "==> Code signing App Bundle (ad-hoc)..."
if command -v codesign &>/dev/null; then
    codesign --force --options runtime --deep --sign - "${APP_BUNDLE}"
    codesign --verify --deep --strict --verbose=1 "${APP_BUNDLE}"
fi

# Ensure background image exists
BG_TIFF="${ROOT_DIR}/packaging/macos/background.tiff"
BG_IMAGE="${ROOT_DIR}/packaging/macos/background.png"
if [ ! -f "${BG_TIFF}" ] || [ "${FORCE_GEN_BG:-0}" = "1" ]; then
    echo "==> Generating background graphics..."
    clang -framework Cocoa "${ROOT_DIR}/packaging/macos/generate-bg.m" -o /tmp/gen_dmg_bg
    /tmp/gen_dmg_bg "${ROOT_DIR}/packaging/macos"
    rm -f /tmp/gen_dmg_bg
fi

DMG_PATH="${OUTPUT_DIR}/rnazar-v${VERSION}-macos-${ARCH_SUFFIX}.dmg"
VOL_NAME="rnazar"
rm -f "${DMG_PATH}"

# Primary method: use create-dmg (handles macOS Finder .DS_Store persistence reliably)
if command -v create-dmg &>/dev/null; then
    echo "==> Packaging DMG using create-dmg..."
    DMG_SRC_DIR="${ROOT_DIR}/target/dmg-source"
    rm -rf "${DMG_SRC_DIR}"
    mkdir -p "${DMG_SRC_DIR}"
    cp -R "${APP_BUNDLE}" "${DMG_SRC_DIR}/"

    create-dmg \
        --volname "${VOL_NAME}" \
        --volicon "${ROOT_DIR}/assets/icons/rnazar.icns" \
        --background "${BG_TIFF}" \
        --window-pos 200 120 \
        --window-size 600 400 \
        --icon-size 120 \
        --icon "rnazar.app" 160 200 \
        --app-drop-link 440 200 \
        --hide-extension "rnazar.app" \
        --overwrite \
        "${DMG_PATH}" \
        "${DMG_SRC_DIR}"

    rm -rf "${DMG_SRC_DIR}"
    echo "==> Successfully created final DMG: ${DMG_PATH}"

elif command -v hdiutil &>/dev/null; then
    echo "==> create-dmg not found, packaging with hdiutil..."
    STAGING_DIR="${ROOT_DIR}/target/dmg-staging"
    rm -rf "${STAGING_DIR}"
    mkdir -p "${STAGING_DIR}/.background"

    cp -R "${APP_BUNDLE}" "${STAGING_DIR}/rnazar.app"
    ln -s /Applications "${STAGING_DIR}/Applications"
    cp "${BG_TIFF}" "${STAGING_DIR}/.background/background.tiff" 2>/dev/null || true
    cp "${BG_IMAGE}" "${STAGING_DIR}/.background/background.png"
    cp "${ROOT_DIR}/assets/icons/rnazar.icns" "${STAGING_DIR}/.VolumeIcon.icns"

    TEMP_DMG="${OUTPUT_DIR}/temp-${VERSION}-$$.dmg"
    rm -f "${TEMP_DMG}"

    hdiutil create -srcfolder "${STAGING_DIR}" -volname "${VOL_NAME}" -fs HFS+ -format UDRW -ov "${TEMP_DMG}"
    ATTACH_OUT=$(hdiutil attach -readwrite -noverify -noautoopen "${TEMP_DMG}")
    DEV=$(echo "$ATTACH_OUT" | awk '/\/Volumes\// {print $1}')
    MOUNT_DIR=$(echo "$ATTACH_OUT" | awk -F'\t' '/\/Volumes\// {print $NF}' | sed 's/^[ \t]*//')

    if command -v SetFile &>/dev/null; then
        SetFile -a C "${MOUNT_DIR}" 2>/dev/null || true
        SetFile -a V "${MOUNT_DIR}/.background" "${MOUNT_DIR}/.VolumeIcon.icns" 2>/dev/null || true
    fi

    sync
    hdiutil detach "${DEV}" 2>/dev/null || (sleep 2 && hdiutil detach "${DEV}" -force 2>/dev/null) || true
    hdiutil convert "${TEMP_DMG}" -format UDZO -ov -o "${DMG_PATH}"
    rm -f "${TEMP_DMG}"
    rm -rf "${STAGING_DIR}"

    echo "==> Successfully created final DMG: ${DMG_PATH}"
else
    tar -czf "${OUTPUT_DIR}/rnazar-v${VERSION}-macos-${ARCH_SUFFIX}.app.tar.gz" -C "$(dirname "${APP_BUNDLE}")" "rnazar.app"
    echo "==> Created fallback archive at ${OUTPUT_DIR}/rnazar-v${VERSION}-macos-${ARCH_SUFFIX}.app.tar.gz"
fi
