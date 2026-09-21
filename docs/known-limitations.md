# FeatherSurf Known Limitations

This document tracks known limitations and issues in the beta release.

## Version: 0.0.1-beta

### Platform-Specific Limitations

#### Windows
- [ ] CEF integration requires CEF binary to be installed separately
- [ ] Auto-update via Sparkle not yet implemented
- [ ] File associations require manual registry setup

#### Linux
- [ ] GTK4 shell requires GTK4 runtime libraries
- [ ] Flatpak/Snap packages not yet available
- [ ] Auto-update via package manager not yet implemented

#### macOS
- [ ] Code signing requires Apple Developer ID certificate
- [ ] Notarization requires Apple notarytool access
- [ ] Auto-update via Sparkle not yet implemented

#### Android
- [ ] Native libraries built for arm64-v8a, armeabi-v7a, x86_64, x86
- [ ] Play Store listing not yet configured
- [ ] In-app update API not yet implemented

#### iOS
- [ ] Requires Xcode 15+ for building
- [ ] App Store submission not yet configured
- [ ] Keychain access requires entitlements

### Feature Limitations

#### Privacy
- [ ] Third-party cookie blocking is enabled by default but may break some sites
- [ ] Fingerprint protection may cause visual differences on some pages
- [ ] AI features disabled by default (opt-in only)

#### Memory Management
- [ ] Tab lifecycle state machine may be aggressive on low-memory devices
- [ ] Background tab suspension may cause delays when switching back
- [ ] Renderer process isolation not yet fully implemented

#### Sync
- [ ] End-to-end encryption uses demo key derivation (not production-ready)
- [ ] Conflict resolution is basic (last-write-wins)
- [ ] Device revocation not yet implemented

#### AI Gateway
- [ ] Local AI models not yet integrated
- [ ] Cloud AI requires API keys (not provided)
- [ ] Prompt injection protection is basic

### Known Issues

1. **Tab restoration delay**: Tabs suspended for >1 hour may take 2-3 seconds to restore
2. **Memory spike on startup**: Initial memory usage may be higher than expected due to CEF initialization
3. **Extension compatibility**: Not all Chrome extensions are compatible
4. **WebAuthn**: Passkey support may not work on all sites
5. **Video playback**: Some DRM-protected content may not play

### Security Considerations

1. **Update signing**: Production signing keys not yet configured
2. **Credential storage**: Uses platform-specific storage but key derivation is demo-only
3. **Extension isolation**: Basic isolation implemented but not fully tested
4. **Network requests**: Some telemetry may be sent without explicit consent

### Performance Budgets

| Metric | Target | Current |
|--------|--------|---------|
| Startup time (cold) | <2s | ~3s |
| Startup time (warm) | <1s | ~1.5s |
| Idle memory (10 tabs) | <500MB | ~600MB |
| Memory per tab | <50MB | ~60MB |
| Background CPU | <5% | ~8% |
| Privacy filter latency | <10ms | ~15ms |

### Testing Status

| Category | Status | Notes |
|----------|--------|-------|
| Reliability | Partial | Soak tests need 24h runtime |
| Compatibility | Partial | Needs real device testing |
| Privacy | Complete | All tests passing |
| Security | Complete | All tests passing |
| Performance | Partial | Needs benchmarking on target hardware |

### Recommendations for Beta

1. **Install MSVC Build Tools** for local Windows testing
2. **Test on real devices** before release
3. **Monitor memory usage** closely during soak tests
4. **Review privacy settings** with security team
5. **Update filter lists** before release

### Issues to Resolve Before Release

1. [ ] Fix startup memory spike
2. [ ] Implement production update signing
3. [ ] Complete Android/iOS device testing
4. [ ] Update filter lists to latest versions
5. [ ] Review all privacy settings
