#!/usr/bin/env bash
# FeatherSurf iOS Packaging Script
#
# Builds the iOS app for App Store or TestFlight.

set -euo pipefail

VERSION="${1:-0.0.1}"
OUTPUT_DIR="${2:-target/release/bundle}"
CONFIG="${3:-release}"

echo "FeatherSurf iOS Packaging"
echo "Version: $VERSION"
echo "Configuration: $CONFIG"

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Build the Rust binaries for iOS targets
echo "Building Rust binaries for iOS..."

# Build for different architectures
TARGETS=(
    "aarch64-apple-ios"
    "aarch64-apple-ios-sim"
)

for target in "${TARGETS[@]}"; do
    echo "Building for $target..."
    cargo build --release --target "$target"
done

# ── Xcode Project Setup ───────────────────────────────────────────

echo "Setting up Xcode project..."
IOS_DIR="$OUTPUT_DIR/ios"
mkdir -p "$IOS_DIR/FeatherSurf"

# Create Info.plist
cat > "$IOS_DIR/FeatherSurf/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleDisplayName</key>
    <string>FeatherSurf</string>
    <key>CFBundleExecutable</key>
    <string>FeatherSurf</string>
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
    <key>LSRequiresIPhoneOS</key>
    <true/>
    <key>MinimumOSVersion</key>
    <string>16.0</string>
    <key>UIDeviceFamily</key>
    <array>
        <integer>1</integer>
        <integer>2</integer>
    </array>
    <key>UILaunchStoryboardName</key>
    <string>LaunchScreen</string>
    <key>UIRequiredDeviceCapabilities</key>
    <array>
        <string>arm64</string>
    </array>
    <key>UISupportedInterfaceOrientations</key>
    <array>
        <string>UIInterfaceOrientationPortrait</string>
        <string>UIInterfaceOrientationLandscapeLeft</string>
        <string>UIInterfaceOrientationLandscapeRight</string>
    </array>
    <key>UISupportedInterfaceOrientations~ipad</key>
    <array>
        <string>UIInterfaceOrientationPortrait</string>
        <string>UIInterfaceOrientationPortraitUpsideDown</string>
        <string>UIInterfaceOrientationLandscapeLeft</string>
        <string>UIInterfaceOrientationLandscapeRight</string>
    </array>
    <key>UIStatusBarStyle</key>
    <string>UIStatusBarStyleDefault</string>
    <key>UIViewControllerBasedStatusBarAppearance</key>
    <false/>
</dict>
</plist>
EOF

# Create entitlements
cat > "$IOS_DIR/FeatherSurf/FeatherSurf.entitlements" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>com.apple.developer.networking.wifi-info</key>
    <true/>
    <key>com.apple.developer.networking.HotspotHelper</key>
    <true/>
    <key>com.apple.security.network.client</key>
    <true/>
    <key>com.apple.security.network.server</key>
    <true/>
</dict>
</plist>
EOF

# Create privacy manifest
cat > "$IOS_DIR/FeatherSurf/PrivacyInfo.xcprivacy" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>NSPrivacyTracking</key>
    <false/>
    <key>NSPrivacyTrackingDomains</key>
    <array/>
    <key>NSPrivacyCollectedDataTypes</key>
    <array/>
    <key>NSPrivacyAccessedAPITypes</key>
    <array/>
</dict>
</plist>
EOF

echo "iOS project files created at $IOS_DIR"

# ── Build iOS App ──────────────────────────────────────────────────

echo "Building iOS app..."
cd "$IOS_DIR"

# Build for device (requires signing)
if [ "$CONFIG" = "release" ]; then
    echo "Building for iOS device..."
    xcodebuild -project FeatherSurf.xcodeproj \
        -scheme FeatherSurf \
        -sdk iphoneos \
        -configuration Release \
        archive \
        -archivePath "$OUTPUT_DIR/FeatherSurf.xcarchive"
    
    echo "Exporting IPA..."
    xcodebuild -exportArchive \
        -archivePath "$OUTPUT_DIR/FeatherSurf.xcarchive" \
        -exportOptionsPlist ExportOptions.plist \
        -exportPath "$OUTPUT_DIR"
    
    echo "IPA created: $OUTPUT_DIR/FeatherSurf.ipa"
else
    echo "Building for iOS simulator..."
    xcodebuild -project FeatherSurf.xcodeproj \
        -scheme FeatherSurf \
        -sdk iphonesimulator \
        -destination 'platform=iOS Simulator,name=iPhone 15,OS=17.0' \
        build
    
    echo "iOS simulator build complete"
fi

# ── Summary ────────────────────────────────────────────────────────

echo ""
echo "iOS packaging complete!"
echo "Output: $OUTPUT_DIR"
ls -la "$OUTPUT_DIR"
