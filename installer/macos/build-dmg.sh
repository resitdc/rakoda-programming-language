#!/bin/bash
# ====================================================================
# RPL macOS DMG Builder
# ====================================================================
# Usage: ./build-dmg.sh <arch>   (arch = x64 or arm64)
# Output: RPL-1.0.0-<arch>.dmg
# ====================================================================
set -e

VERSION="${RPL_VERSION:-1.0.0}"
ARCH="${1:-arm64}"
PRODUCT_NAME="RPL"
DMG_NAME="RPL-${VERSION}-${ARCH}.dmg"
PKG_NAME="RPL-${VERSION}-${ARCH}.pkg"
STAGING_DIR="$(mktemp -d)"
ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"

echo "🔨 Building RPL ${VERSION} DMG for macOS ${ARCH}..."

# ---- Create Package Layout ----
mkdir -p "${STAGING_DIR}/usr/local/bin"
mkdir -p "${STAGING_DIR}/usr/local/share/rpl"

# Copy binary
cp "${ROOT_DIR}/target/release/rpl" "${STAGING_DIR}/usr/local/bin/rpl"
chmod 755 "${STAGING_DIR}/usr/local/bin/rpl"

# Ad-hoc codesign to prevent macOS Gatekeeper malware warning
codesign --force --deep -s - "${STAGING_DIR}/usr/local/bin/rpl"
xattr -cr "${STAGING_DIR}/usr/local/bin/rpl"

# Copy resources
cp "${ROOT_DIR}/LICENSE" "${STAGING_DIR}/usr/local/share/rpl/"

# ---- Build .pkg ----
echo "📦 Creating .pkg installer..."
pkgbuild \
    --root "${STAGING_DIR}" \
    --identifier "com.rakoda.rpl" \
    --version "${VERSION}" \
    --install-location "/" \
    "${ROOT_DIR}/${PKG_NAME}"

# ---- Build .dmg ----
echo "💿 Creating .dmg..."
DMG_STAGING="$(mktemp -d)"
cp "${ROOT_DIR}/${PKG_NAME}" "${DMG_STAGING}/"
cp "${ROOT_DIR}/LICENSE" "${DMG_STAGING}/"

# Create README for the DMG
cat > "${DMG_STAGING}/BACA-INI.txt" << 'EOF'
Rakoda Programming Language (RPL) v1.0.0
=========================================

Cara Install:
1. Klik dua kali file RPL-*.pkg
2. Ikuti langkah-langkah instalasi
3. Buka Terminal
4. Ketik: rpl --version

Selamat belajar pemrograman!
Salam Restu Dwi Cahyo
EOF

hdiutil create \
    -volname "${PRODUCT_NAME}" \
    -srcfolder "${DMG_STAGING}" \
    -ov \
    -format UDZO \
    "${ROOT_DIR}/${DMG_NAME}"

# ---- Cleanup ----
rm -rf "${STAGING_DIR}" "${DMG_STAGING}"
rm -f "${ROOT_DIR}/${PKG_NAME}"

echo "✅ Successfully created: ${DMG_NAME}"
