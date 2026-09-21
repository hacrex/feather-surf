#!/usr/bin/env bash
# FeatherSurf macOS Packaging Script
#
# Builds DMG and PKG installers for macOS.

set -euo pipefail

VERSION="${1:-0.0.1}"
OUTPUT_DIR="${2:-target/release/bundle}"
SIGN="${3:-false}"
NOTARIZE="${4:-false}"

echo "FeatherSurf macOS Packaging"
echo "Version: $VERSION"

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Build the Rust binaries
echo "Building Rust binaries..."
cargo build --release --target aarch64-apple-darwin
cargo build --release --target x86_64-apple-darwin

# Create universal binary
echo "Creating universal binary..."
mkdir -p target/release/universal
lipo -create \
    target/aarch64-apple-darwin/release/feathersurf \
    target/x86_64-apple-darwin/release/feathersurf \
    -output target/release/universal/feathersurf

# Create .app bundle
echo "Creating .app bundle..."
APP_DIR="$OUTPUT_DIR/FeatherSurf.app"
mkdir -p "$APP_DIR/Contents/MacOS"
mkdir -p "$APP_DIR/Contents/Resources"

cp target/release/universal/feathersurf "$APP_DIR/Contents/MacOS/"

cat > "$APP_DIR/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleExecutable</key>
    <string>feathersurf</string>
    <key>CFBundleIconFile</key>
    <string>feathersurf.icns</string>
    <key>CFBundleIdentifier</key>
    <string>dev.feathersurf.FeatherSurf</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>FeatherSurf</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>$VERSION</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>LSMinimumSystemVersion</key>
    <string>13.0</string>
    <key>LSUIElement</key>
    <false/>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSHumanReadableCopyright</key>
    <string>Copyright © 2024 FeatherSurf. All rights reserved.</string>
    <key>NSRequiresAquaSystemAppearance</key>
    <false/>
    <key>CFBundleDocumentTypes</key>
    <array>
        <dict>
            <key>CFBundleTypeName</key>
            <string>HTML Document</string>
            <key>CFBundleTypeExtensions</key>
            <array>
                <string>html</string>
                <string>htm</string>
            </array>
            <key>CFBundleTypeRole</key>
            <string>Viewer</string>
        </dict>
        <dict>
            <key>CFBundleTypeName</key>
            <string>URL</string>
            <key>CFBundleTypeExtensions</key>
            <array>
                <string>url</string>
            </array>
            <key>CFBundleTypeRole</key>
            <string>Viewer</string>
        </dict>
    </array>
    <key>CFBundleURLTypes</key>
    <array>
        <dict>
            <key>CFBundleURLName</key>
            <string>FeatherSurf URL</string>
            <key>CFBundleURLSchemes</key>
            <array>
                <string>http</string>
                <string>https</string>
                <string>feathersurf</string>
            </array>
        </dict>
    </array>
</dict>
</plist>
EOF

# Create PkgInfo
echo -n "APPL????" > "$APP_DIR/Contents/PkgInfo"

echo ".app bundle created at $APP_DIR"

# ── Code Signing ───────────────────────────────────────────────────

if [ "$SIGN" = "true" ]; then
    echo "Signing with Apple Developer ID..."
    codesign --force --deep --sign "Developer ID Application: FeatherSurf" "$APP_DIR"
    echo "Signed .app bundle"
fi

# ── Notarization ───────────────────────────────────────────────────

if [ "$NOTARIZE" = "true" ]; then
    echo "Notarizing with Apple..."
    # Create ZIP for notarization
    ditto -c -k --keepParent "$APP_DIR" "$OUTPUT_DIR/FeatherSurf-$VERSION.zip"
    
    # Submit for notarization
    xcrun notarytool submit "$OUTPUT_DIR/FeatherSurf-$VERSION.zip" \
        --apple-id "developer@feathersurf.dev" \
        --team-id "YOUR_TEAM_ID" \
        --password "YOUR_APP_PASSWORD" \
        --wait
    
    # Staple notarization
    xcrun stapler staple "$APP_DIR"
    
    echo "Notarization complete"
fi

# ── Create DMG ─────────────────────────────────────────────────────

echo "Creating DMG installer..."
DMG_DIR="$OUTPUT_DIR/dmg"
mkdir -p "$DMG_DIR"

# Create temporary directory for DMG contents
DMG_TEMP="$DMG_DIR/temp"
mkdir -p "$DMG_TEMP"
cp -r "$APP_DIR" "$DMG_TEMP/"

# Create Applications symlink
ln -s /Applications "$DMG_TEMP/Applications"

# Create DMG
hdiutil create -volname "FeatherSurf" \
    -srcfolder "$DMG_TEMP" \
    -ov -format UDZO \
    "$DMG_DIR/FeatherSurf-$VERSION.dmg"

# Clean up
rm -rf "$DMG_TEMP"

echo "DMG created: $DMG_DIR/FeatherSurf-$VERSION.dmg"

# ── Create PKG ─────────────────────────────────────────────────────

echo "Creating PKG installer..."
PKG_DIR="$OUTPUT_DIR/pkg"
mkdir -p "$PKG_DIR"

# Create package description
cat > "$PKG_DIR/distribution.xml" << EOF
<?xml version="1.0" encoding="utf-8"?>
<installer-gui-script minSpecVersion="2">
    <title>FeatherSurf</title>
    <options customize="never" require-scripts="false" hostArchitectures="x86_64,arm64"/>
    <domains enable_localSystem="true"/>
    <choices-outline>
        <line choice="choice1"/>
    </choices-outline>
    <choice id="choice1" title="FeatherSurf">
        <pkg-ref id="dev.feathersurf.FeatherSurf"/>
    </choice>
    <pkg-ref id="dev.feathersurf.FeatherSurf" version="$VERSION">FeatherSurf.pkg</pkg-ref>
</installer-gui-script>
EOF

# Create pkgbuild
pkgbuild --root "$APP_DIR" \
    --identifier "dev.feathersurf.FeatherSurf" \
    --version "$VERSION" \
    --install-location "/Applications" \
    "$PKG_DIR/FeatherSurf.pkg"

echo "PKG created: $PKG_DIR/FeatherSurf.pkg"

# ── Summary ────────────────────────────────────────────────────────

echo ""
echo "macOS packaging complete!"
echo "Output: $OUTPUT_DIR"
ls -la "$OUTPUT_DIR"
