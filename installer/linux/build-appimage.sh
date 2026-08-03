#!/bin/bash
# ====================================================================
# RPL Linux AppImage Builder
# ====================================================================
# Usage: ./build-appimage.sh
# Output: RPL-1.0.0-x86_64.AppImage
# ====================================================================
set -e

VERSION="${RPL_VERSION:-1.0.0}"
PRODUCT_NAME="RPL"
ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
APPDIR="${ROOT_DIR}/RPL.AppDir"

echo "🔨 Building RPL ${VERSION} AppImage..."

# ---- Create AppDir Layout ----
rm -rf "${APPDIR}"
mkdir -p "${APPDIR}/usr/bin"
mkdir -p "${APPDIR}/usr/share/rpl"
mkdir -p "${APPDIR}/usr/share/icons/hicolor/256x256/apps"

# Copy binary
cp "${ROOT_DIR}/target/release/rpl" "${APPDIR}/usr/bin/rpl"
chmod 755 "${APPDIR}/usr/bin/rpl"
cp "${ROOT_DIR}/LICENSE" "${APPDIR}/usr/share/rpl/"

# Copy icon
cp "${ROOT_DIR}/installer/icon/rpl-icon.png" "${APPDIR}/usr/share/icons/hicolor/256x256/apps/rpl.png" 2>/dev/null || true
cp "${ROOT_DIR}/installer/icon/rpl-icon.png" "${APPDIR}/rpl.png" 2>/dev/null || true

# Create .desktop file
cat > "${APPDIR}/rpl.desktop" << EOF
[Desktop Entry]
Type=Application
Name=RPL
Comment=Rakoda Programming Language
Exec=rpl
Icon=rpl
Categories=Development;
Terminal=true
EOF

# Create AppRun
cat > "${APPDIR}/AppRun" << 'EOF'
#!/bin/bash
SELF=$(readlink -f "$0")
HERE=${SELF%/*}
export PATH="${HERE}/usr/bin:${PATH}"
exec "${HERE}/usr/bin/rpl" "$@"
EOF
chmod 755 "${APPDIR}/AppRun"

# ---- Download appimagetool if not present ----
APPIMAGETOOL="${ROOT_DIR}/appimagetool"
if [ ! -f "${APPIMAGETOOL}" ]; then
    echo "📥 Downloading appimagetool..."
    curl -Lo "${APPIMAGETOOL}" "https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage"
    chmod +x "${APPIMAGETOOL}"
fi

# ---- Build AppImage ----
echo "📦 Creating AppImage..."
APPIMAGE_EXTRACT_AND_RUN=1 ARCH=x86_64 "${APPIMAGETOOL}" "${APPDIR}" "${ROOT_DIR}/RPL-${VERSION}-x86_64.AppImage"

# ---- Cleanup ----
rm -rf "${APPDIR}"

echo "✅ Successfully created: RPL-${VERSION}-x86_64.AppImage"
