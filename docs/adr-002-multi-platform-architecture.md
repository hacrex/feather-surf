# ADR-002: Multi-Platform Architecture for Windows, Linux, macOS, Android, iOS

## Status

Accepted

## Context

FeatherSurf is a privacy-first, RAM-efficient browser with a Rust core that
provides tab management, memory intelligence, and privacy controls. The current
architecture targets desktop platforms (Windows, Linux, macOS) using the
Chromium Embedded Framework (CEF) as the rendering engine.

Expanding to mobile platforms introduces fundamental constraints:

- **iOS mandates WebKit**: Apple requires all browsers on iOS to use WKWebView
  (WebKit). Chromium/CEF-based engines are forbidden. This is a non-negotiable
  platform policy.
- **Android WebView is Chromium-based**: Android's `android.webkit.WebView` uses
  Chromium under the hood, but its API differs significantly from CEF's C API.
  Integration requires JNI bridging rather than C FFI.
- **Rust core is platform-agnostic**: The existing Rust crates (`tab-manager`,
  `memory-manager`, `privacy-engine`, etc.) are designed without
  platform-specific dependencies and can compile to any target supported by
  `rustc` — including ARM64 for mobile.

The challenge is to unify these heterogeneous rendering engines behind a single
abstraction while preserving the Rust core's platform independence.

## Decision

### Platform Abstraction Layer (PAL)

Introduce a Platform Abstraction Layer (PAL) in Rust that defines traits for
platform-specific operations:

| Trait | Responsibility |
|---|---|
| `MemoryProvider` | Platform memory queries (RSS, PSS, heap stats) |
| `ProcessManager` | Process creation, monitoring, suspension, termination |
| `Renderer` | WebView lifecycle, navigation, content loading |
| `WindowManager` | Native window creation, focus, fullscreen, minimize |

Each platform provides a concrete implementation of these traits. The Rust core
interacts only with trait objects, never with platform APIs directly.

### C FFI Bridge

Expose the Rust core as a C-compatible shared library (`cdylib` + `staticlib`)
via a flat C FFI bridge. This enables integration from any language that can
call C functions:

- **Desktop shells** (C++, C#, Objective-C) link the Rust `.dll`/`.so`/`.dylib`
  and call exported functions.
- **Android** links the Rust `.so` via JNI and calls exported C functions.
- **iOS** links the Rust static `.a` via a Swift bridging header and calls
  exported C functions.

### Rendering Strategy by Platform

| Platform | Renderer | Integration | Output |
|---|---|---|---|
| Windows | CEF | C FFI → Rust core | `.dll` |
| Linux | CEF | C FFI → Rust core | `.so` |
| macOS | CEF | C FFI → Rust core | `.dylib` |
| Android | Android WebView | JNI → C FFI → Rust core | `.so` (per ABI) |
| iOS | WKWebView | Swift bridging → C FFI → Rust core | `.a` (static) |

### Architecture Diagram

```
┌─────────────────────────────────────────────┐
│           Rust Core (Platform-Agnostic)      │
│  tab-manager, memory-manager, privacy, etc.  │
│  Compiles to: .dll / .so / .dylib / .a       │
└────────────┬───────────────┬────────────────┘
             │               │
    ┌────────▼────────┐  ┌───▼──────────────────┐
    │  C FFI Bridge   │  │  C FFI Bridge        │
    └────────┬────────┘  └───┬──────────────────┘
             │               │
    ┌────────▼────────┐  ┌───▼──────────────────┐
    │ Desktop Shells   │  │ Mobile Shells         │
    │ (CEF-based)      │  │ (Platform WebViews)   │
    │ Win/Linux/macOS  │  │ Android/iOS           │
    └─────────────────┘  └──────────────────────┘
```

**Desktop path**: Native shell (C++/ObjC) → C FFI calls → Rust core manages
tabs/memory → CEF renders web content.

**Android path**: Kotlin/Java shell → JNI → C FFI calls → Rust core manages
tabs/memory → `android.webkit.WebView` renders web content.

**iOS path**: Swift shell → bridging header → C FFI calls → Rust core manages
tabs/memory → `WKWebView` renders web content.

## Consequences

### Positive

- **Single Rust core**: Tab management, memory intelligence, and privacy rules
  are implemented once and reused across all five platforms.
- **Consistent behavior**: Users get the same tab suspension, memory scoring,
  and privacy controls regardless of platform.
- **iOS compliance**: Uses WebKit (WKWebView), satisfying Apple's mandatory
  requirement.
- **Android natural fit**: Android WebView is the standard and expected browser
  component on Android.
- **Testable core**: The Rust core can be unit-tested and integration-tested on
  any platform without needing a full browser shell.

### Negative

- **Two rendering engines**: Desktop uses CEF (Chromium), while iOS uses
  WebKit and Android uses Android WebView. A renderer abstraction is needed
  to normalize behavior across engines. Some web APIs may behave differently.
- **iOS extension limitation**: Chromium extensions are not available on iOS
  due to the WebKit requirement. Extension support on iOS will require a
  separate WebKit-based extension mechanism (if Apple provides one) or will
  be unavailable.
- **Platform shell duplication**: Each platform requires its own native shell
  (Win32/C++, GTK4/Qt, AppKit/ObjC, Android Activity, iOS ViewController).
  This increases UI code maintenance.

### Neutral

- **FFI overhead**: The C FFI bridge adds ~microseconds per call, which is
  negligible for browser operations (navigation, tab switching, memory queries
  happen at human-perceivable timescales).
- **Build matrix complexity**: Five targets increase CI/CD time and require
  platform-specific build environments (NDK for Android, Xcode for iOS).

## Platform-Specific Details

### Windows

- **Shell**: C++ with Win32/COM or Qt
- **Renderer**: CEF (Chromium Embedded Framework)
- **Rust integration**: Dynamic library (`.dll`) linked via C FFI
- **Build target**: `x86_64-pc-windows-msvc`
- **Process monitoring**: Windows API (`QueryWorkingSetEx`, `GetProcessMemoryInfo`)
- **Sandboxing**: CEF sandbox or Windows AppContainer

### Linux

- **Shell**: C++ with GTK4 or Qt
- **Renderer**: CEF
- **Rust integration**: Shared object (`.so`) linked via C FFI
- **Build target**: `x86_64-unknown-linux-gnu`
- **Process monitoring**: `/proc/[pid]/smaps`, `getrusage()`
- **Sandboxing**: CEF sandbox or seccomp-bpf

### macOS

- **Shell**: Objective-C/Swift with AppKit/Cocoa
- **Renderer**: CEF
- **Rust integration**: Dynamic library (`.dylib`) linked via C FFI
- **Build target**: `aarch64-apple-darwin`, `x86_64-apple-darwin`
- **Process monitoring**: `mach_task_basic_info`, `proc_pidinfo()`
- **Sandboxing**: macOS App Sandbox

### Android

- **Shell**: Kotlin with Android Activity
- **Renderer**: `android.webkit.WebView`
- **Rust integration**: Shared object (`.so`) linked via JNI → C FFI
- **Build targets**: `aarch64-linux-android`, `armv7-linux-androideabi`,
  `x86_64-linux-android`
- **Process monitoring**: `/proc/self/smaps`, `Debug.getMemoryInfo()`
- **Minimum SDK**: API 24 (Android 7.0)
- **NDK**: Required for Rust cross-compilation

### iOS

- **Shell**: Swift with UIKit (`UIViewController`)
- **Renderer**: `WKWebView`
- **Rust integration**: Static library (`.a`) linked via Swift bridging header
- **Build targets**: `aarch64-apple-ios`, `x86_64-apple-ios` (simulator)
- **Process monitoring**: `mach_task_basic_info` (limited by iOS sandbox)
- **Minimum deployment**: iOS 15+
- **Xcode**: Required for build and code signing

## Build Strategy

### Cross-Compilation

Use the `cross` tool for Rust cross-compilation, with platform-specific
toolchains:

| Target | Toolchain | Sysroot |
|---|---|---|
| `x86_64-pc-windows-msvc` | MSVC (Visual Studio) | Windows SDK |
| `x86_64-unknown-linux-gnu` | GCC/Clang | Linux sysroot |
| `aarch64-apple-darwin` | Xcode Clang | macOS SDK |
| `aarch64-linux-android` | Android NDK | NDK sysroot |
| `aarch64-apple-ios` | Xcode Clang | iOS SDK |

### Build Scripts

Platform-specific build scripts in `/scripts/`:

```
scripts/
├── build-windows.sh
├── build-linux.sh
├── build-macos.sh
├── build-android.sh
└── build-ios.sh
```

Each script handles:
1. Setting environment variables for the target toolchain
2. Running `cargo build` with the correct `--target` and features
3. Producing the platform-specific output (`.dll`, `.so`, `.dylib`, `.a`)
4. Copying the output to the platform shell's build directory

### CI/CD

GitHub Actions matrix builds:

```yaml
strategy:
  matrix:
    include:
      - target: x86_64-pc-windows-msvc
        os: windows-latest
      - target: x86_64-unknown-linux-gnu
        os: ubuntu-latest
      - target: aarch64-apple-darwin
        os: macos-latest
      - target: aarch64-linux-android
        os: ubuntu-latest
      - target: aarch64-apple-ios
        os: macos-latest
```

Each matrix entry runs the corresponding build script and uploads artifacts.

## Testing

### Rust Core

- **Unit tests**: Run on all targets via `cargo test`. Platform-specific code
  is behind `#[cfg(target_os = "...")]` attributes.
- **Integration tests**: Test FFI exports, trait implementations, and
  cross-module interactions.
- **Property-based tests**: Use `proptest` for memory scoring algorithms and
  tab eviction logic.

### Platform Shells

Each shell is tested independently on its target OS:

| Platform | Test Framework | Scope |
|---|---|---|
| Windows | Google Test / Catch2 | Shell UI, CEF integration, FFI calls |
| Linux | Google Test / Catch2 | Shell UI, CEF integration, FFI calls |
| macOS | XCTest | Shell UI, CEF integration, FFI calls |
| Android | Espresso / JUnit | WebView integration, JNI calls, Activity lifecycle |
| iOS | XCUITest | WKWebView integration, Swift bridging, ViewController lifecycle |

### End-to-End

- **Desktop**: WebDriver protocol via Selenium/Playwright for automated
  browsing tests (navigation, tab switching, memory behavior).
- **Android**: Espresso UI tests for WebView interactions and
  `androidTest` for JNI/Rust integration.
- **iOS**: XCUITest for WKWebView interactions and Swift bridging validation.

## References

- [ADR-001: Chromium Integration Strategy](docs/adr-001-chromium-integration.md)
- [CEF Documentation](https://cef-builds.spotifycdn.com/docs/index.html)
- [Android WebView Documentation](https://developer.android.com/reference/android/webkit/WebView)
- [WKWebView Documentation](https://developer.apple.com/documentation/webkit/wkwebview)
- [Rust Cross-Compilation](https://rust-lang.github.io/rustup/cross-compilation.html)
- [FeatherSurf Architecture](docs/architecture.md)
