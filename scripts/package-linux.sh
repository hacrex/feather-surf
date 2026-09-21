#!/usr/bin/env bash
# FeatherSurf Linux Packaging Script
#
# Builds DEB, RPM, AppImage, and Flatpak packages for Linux.

set -euo pipefail

VERSION="${1:-0.0.1}"
OUTPUT_DIR="${2:-target/release/bundle}"
ARCH="${3:-amd64}"

echo "FeatherSurf Linux Packaging"
echo "Version: $VERSION"
echo "Architecture: $ARCH"

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Build the Rust binaries
echo "Building Rust binaries..."
cargo build --release --target x86_64-unknown-linux-gnu

# ── DEB Package (Debian/Ubuntu) ────────────────────────────────────

echo "Creating DEB package..."
DEB_DIR="$OUTPUT_DIR/deb/feathersurf_${VERSION}_${ARCH}"
mkdir -p "$DEB_DIR/DEBIAN"
mkdir -p "$DEB_DIR/usr/bin"
mkdir -p "$DEB_DIR/usr/share/applications"
mkdir -p "$DEB_DIR/usr/share/icons/hicolor/256x256/apps"
mkdir -p "$DEB_DIR/usr/share/man/man1"

# Copy binary
cp target/x86_64-unknown-linux-gnu/release/feathersurf "$DEB_DIR/usr/bin/"

# Create control file
cat > "$DEB_DIR/DEBIAN/control" << EOF
Package: feathersurf
Version: $VERSION
Section: web
Priority: optional
Architecture: $ARCH
Depends: libgtk-3-0 (>= 3.24), libglib2.0-0 (>= 2.64), libcef1
Maintainer: FeatherSurf Team <team@feathersurf.dev>
Homepage: https://feathersurf.dev
Description: Privacy-first, RAM-efficient browser
 FeatherSurf is an open-source browser that prioritizes privacy
 and efficient memory usage through intelligent tab lifecycle management.
 It features local-first operation, transparent architecture, and
 cross-platform support.
EOF

# Create desktop file
cat > "$DEB_DIR/usr/share/applications/feathersurf.desktop" << EOF
[Desktop Entry]
Name=FeatherSurf
Comment=Privacy-first, RAM-efficient browser
Exec=feathersurf %u
Icon=feathersurf
Terminal=false
Type=Application
Categories=Network;WebBrowser;
MimeType=text/html;text/xml;application/xhtml+xml;application/xml;application/rss+xml;application/rdf+xml;x-scheme-handler/http;x-scheme-handler/https;x-scheme-handler/feathersurf;
StartupWMClass=feathersurf
EOF

# Build DEB
dpkg-deb --build "$DEB_DIR" "$OUTPUT_DIR/feathersurf_${VERSION}_${ARCH}.deb"
echo "DEB package created: $OUTPUT_DIR/feathersurf_${VERSION}_${ARCH}.deb"

# ── RPM Package (Fedora/RHEL) ─────────────────────────────────────

echo "Creating RPM package..."
RPM_DIR="$OUTPUT_DIR/rpm"
mkdir -p "$RPM_DIR/SOURCES" "$RPM_DIR/SPECS"

cat > "$RPM_DIR/SPECS/feathersurf.spec" << EOF
Name:           feathersurf
Version:        $VERSION
Release:        1%{?dist}
Summary:        Privacy-first, RAM-efficient browser

License:        Apache-2.0
URL:            https://feathersurf.dev
Source0:        feathersurf-%{version}.tar.gz

BuildRequires:  gcc
BuildRequires:  rust-packaging

%description
FeatherSurf is an open-source browser that prioritizes privacy
and efficient memory usage through intelligent tab lifecycle management.

%prep
%setup -q

%build
cargo build --release

%install
mkdir -p %{buildroot}/usr/bin
cp target/release/feathersurf %{buildroot}/usr/bin/

%files
/usr/bin/feathersurf

%changelog
* $(date "+%a %b %d %Y") FeatherSurf Team <team@feathersurf.dev> - $VERSION-1
- Initial package
EOF

echo "RPM spec created at $RPM_DIR/SPECS/feathersurf.spec"

# ── AppImage ───────────────────────────────────────────────────────

echo "Creating AppImage..."
APPDIR="$OUTPUT_DIR/AppDir"
mkdir -p "$APPDIR/usr/bin" "$APPDIR/usr/share/applications"

cp target/x86_64-unknown-linux-gnu/release/feathersurf "$APPDIR/usr/bin/"

cat > "$APPDIR/feathersurf.desktop" << EOF
[Desktop Entry]
Name=FeatherSurf
Comment=Privacy-first, RAM-efficient browser
Exec=feathersurf
Icon=feathersurf
Terminal=false
Type=Application
Categories=Network;WebBrowser;
EOF

echo "AppDir created at $APPDIR (would use appimagetool to package)"

# ── Flatpak ────────────────────────────────────────────────────────

echo "Creating Flatpak manifest..."
cat > "$OUTPUT_DIR/feathersurf.flatpak.yml" << EOF
app-id: dev.feathersurf.FeatherSurf
runtime: org.freedesktop.Platform
runtime-version: '23.08'
sdk: org.freedesktop.Sdk
command: feathersurf
modules:
  - name: feathersurf
    buildsystem: simple
    build-commands:
      - cargo build --release
      - install -Dm755 target/release/feathersurf /app/bin/feathersurf
    sources:
      - type: archive
        url: https://github.com/hacrex/feather-surf/archive/refs/tags/v$VERSION.tar.gz
        sha256: CALCULATE_HASH_HERE
finish-args:
  - --share=ipc
  - --share=network
  - --socket=x11
  - --socket=wayland
  - --device=dri
  - --talk-name=org.freedesktop.secrets
EOF

echo "Flatpak manifest created at $OUTPUT_DIR/feathersurf.flatpak.yml"

# ── .desktop File ──────────────────────────────────────────────────

echo "Creating .desktop file..."
cat > "$OUTPUT_DIR/feathersurf.desktop" << EOF
[Desktop Entry]
Name=FeatherSurf
Comment=Privacy-first, RAM-efficient browser
Exec=feathersurf %u
Icon=feathersurf
Terminal=false
Type=Application
Categories=Network;WebBrowser;
MimeType=text/html;text/xml;application/xhtml+xml;application/xml;application/rss+xml;application/rdf+xml;x-scheme-handler/http;x-scheme-handler/https;x-scheme-handler/feathersurf;
StartupWMClass=feathersurf
EOF

# ── Summary ────────────────────────────────────────────────────────

echo ""
echo "Linux packaging complete!"
echo "Output: $OUTPUT_DIR"
ls -la "$OUTPUT_DIR"
