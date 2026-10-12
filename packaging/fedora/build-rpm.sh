#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUTPUT_DIR="${ROOT_DIR}/target/fedora"
RPMBUILD_DIR="${OUTPUT_DIR}/rpmbuild"

VERSION="${RNAZAR_VERSION:-$(grep '^version' "${ROOT_DIR}/Cargo.toml" | head -n1 | cut -d '"' -f2 || echo "0.1.0")}"
VERSION="${VERSION#v}"

echo "==> Building rnazar release binary..."
cargo build --release --manifest-path "${ROOT_DIR}/Cargo.toml"

echo "==> Preparing RPM build environment..."
rm -rf "${RPMBUILD_DIR}"
mkdir -p "${RPMBUILD_DIR}"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}
mkdir -p "${RPMBUILD_DIR}/SOURCES/png"
mkdir -p "${OUTPUT_DIR}"

# Stage sources
cp "${ROOT_DIR}/target/release/rnazar" "${RPMBUILD_DIR}/SOURCES/rnazar"
cp "${ROOT_DIR}/assets/rnazar.desktop" "${RPMBUILD_DIR}/SOURCES/rnazar.desktop"
cp "${ROOT_DIR}/assets/dbus/org.rnazar.Notifier.service" "${RPMBUILD_DIR}/SOURCES/org.rnazar.Notifier.service"
cp "${ROOT_DIR}/LICENSE" "${RPMBUILD_DIR}/SOURCES/LICENSE"
cp "${ROOT_DIR}/assets/icons/nazar.svg" "${RPMBUILD_DIR}/SOURCES/nazar.svg"
cp -r "${ROOT_DIR}/assets/icons/png/"* "${RPMBUILD_DIR}/SOURCES/png/"
cp "${ROOT_DIR}/packaging/fedora/rnazar.spec" "${RPMBUILD_DIR}/SPECS/rnazar.spec"

echo "==> Building RPM package..."
if command -v rpmbuild >/dev/null 2>&1; then
    rpmbuild --define "_topdir ${RPMBUILD_DIR}" \
             --define "version ${VERSION}" \
             -bb "${RPMBUILD_DIR}/SPECS/rnazar.spec"
    
    find "${RPMBUILD_DIR}/RPMS" -type f -name "*.rpm" -exec cp {} "${OUTPUT_DIR}/" \;
    echo "==> Successfully created RPM package in ${OUTPUT_DIR}!"
else
    echo "Warning: rpmbuild is not installed on this host. In CI (or with rpm/rpmbuild installed), this script will package the RPM directly."
fi

