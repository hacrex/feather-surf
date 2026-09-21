# FeatherSurf Installation Instructions

## Windows

### Option 1: MSI Installer (Recommended)
1. Download `FeatherSurf-1.0.0-x64.msi` from [Releases](https://github.com/hacrex/feather-surf/releases)
2. Run the installer
3. Follow the setup wizard
4. Launch FeatherSurf from Start Menu or desktop shortcut

### Option 2: EXE Installer
1. Download `FeatherSurf-1.0.0-Setup.exe` from [Releases](https://github.com/hacrex/feather-surf/releases)
2. Run the installer as Administrator
3. Follow the setup wizard
4. Launch FeatherSurf from Start Menu or desktop shortcut

### System Requirements
- Windows 10 (version 2004 or later) or Windows 11
- 4 GB RAM minimum (8 GB recommended)
- 500 MB disk space

### File Associations
After installation, FeatherSurf will handle:
- HTML files (.html, .htm)
- HTTP/HTTPS links
- FeatherSurf protocol (feathersurf://)

## Linux

### Option 1: DEB Package (Debian/Ubuntu)
```bash
# Download the package
wget https://github.com/hacrex/feather-surf/releases/download/v1.0.0/feathersurf_1.0.0_amd64.deb

# Install
sudo dpkg -i feathersurf_1.0.0_amd64.deb

# Fix dependencies if needed
sudo apt-get install -f
```

### Option 2: RPM Package (Fedora/RHEL)
```bash
# Download the package
wget https://github.com/hacrex/feather-surf/releases/download/v1.0.0/feathersurf-1.0.0-1.x86_64.rpm

# Install
sudo rpm -i feathersurf-1.0.0-1.x86_64.rpm
```

### Option 3: AppImage
```bash
# Download the AppImage
wget https://github.com/hacrex/feather-surf/releases/download/v1.0.0/FeatherSurf-1.0.0.AppImage

# Make executable
chmod +x FeatherSurf-1.0.0.AppImage

# Run
./FeatherSurf-1.0.0.AppImage
```

### Option 4: Flatpak (Coming Soon)
```bash
flatpak install dev.feathersurf.FeatherSurf
flatpak run dev.feathersurf.FeatherSurf
```

### System Requirements
- Ubuntu 22.04+, Fedora 38+, Debian 12+
- 4 GB RAM minimum (8 GB recommended)
- 500 MB disk space
- GTK4 libraries

## macOS

### Option 1: DMG Installer (Recommended)
1. Download `FeatherSurf-1.0.0.dmg` from [Releases](https://github.com/hacrex/feather-surf/releases)
2. Open the DMG file
3. Drag FeatherSurf to the Applications folder
4. Launch from Applications or Spotlight

### Option 2: PKG Installer
1. Download `FeatherSurf-1.0.0.pkg` from [Releases](https://github.com/hacrex/feather-surf/releases)
2. Run the installer
3. Follow the setup wizard
4. Launch from Applications

### System Requirements
- macOS 13 Ventura or later
- Apple Silicon or Intel processor
- 4 GB RAM minimum (8 GB recommended)
- 500 MB disk space

### Gatekeeper
If macOS blocks the app:
1. Right-click the app and select "Open"
2. Click "Open" in the dialog
3. Or go to System Settings > Privacy & Security and click "Open Anyway"

## Android

### Option 1: APK
1. Download `feathersurf-1.0.0.apk` from [Releases](https://github.com/hacrex/feather-surf/releases)
2. Enable "Install from unknown sources" in Settings
3. Open the APK file
4. Follow the installation prompts

### Option 2: Google Play (Coming Soon)
1. Open Google Play Store
2. Search for "FeatherSurf"
3. Tap "Install"

### System Requirements
- Android 10 (API 29) or later
- 4 GB RAM minimum
- 100 MB disk space

## iOS

### Option 1: TestFlight (Beta)
1. Download TestFlight from App Store
2. Open the invitation link
3. Install FeatherSurf via TestFlight

### Option 2: App Store (Coming Soon)
1. Open App Store
2. Search for "FeatherSurf"
3. Tap "Get"

### System Requirements
- iOS 16 or later
- iPhone or iPad
- 100 MB disk space

## Building from Source

### Prerequisites
- Rust 1.98.1 or later
- Platform-specific build tools (see below)

### Windows
```powershell
# Install Visual Studio Build Tools
# Install Rust
rustup default stable

# Clone and build
git clone https://github.com/hacrex/feather-surf.git
cd feather-surf
cargo build --release
```

### Linux
```bash
# Install dependencies
sudo apt-get install build-essential libgtk-3-dev

# Clone and build
git clone https://github.com/hacrex/feather-surf.git
cd feather-surf
cargo build --release
```

### macOS
```bash
# Install Xcode Command Line Tools
xcode-select --install

# Clone and build
git clone https://github.com/hacrex/feather-surf.git
cd feather-surf
cargo build --release
```

## Uninstalling

### Windows
1. Go to Settings > Apps > Installed apps
2. Find FeatherSurf
3. Click "Uninstall"

### Linux
```bash
# DEB
sudo dpkg -r feathersurf

# RPM
sudo rpm -e feathersurf
```

### macOS
1. Open Finder
2. Go to Applications
3. Drag FeatherSurf to Trash

### Android
1. Go to Settings > Apps
2. Find FeatherSurf
3. Tap "Uninstall"

## Troubleshooting

### App won't start
- Check system requirements
- Update to latest version
- Check crash logs in `~/.feather-surf/logs/`

### High memory usage
- Close unused tabs
- Check for memory-hungry extensions
- Report issue with diagnostics

### Privacy concerns
- Review privacy settings in Preferences
- Check what data is stored locally
- Report any suspicious network activity

## Support

- [GitHub Issues](https://github.com/hacrex/feather-surf/issues)
- [Documentation](https://feathersurf.dev/docs)
- [Security Policy](SECURITY.md)
