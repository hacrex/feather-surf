# FeatherSurf Windows Shell

Native Win32 browser shell for FeatherSurf on Windows.

## Architecture

```
shells/windows/
├── CMakeLists.txt          # Build configuration (requires CEF_ROOT)
├── README.md
└── src/
    ├── main.cpp            # Win32 entry point, CEF initialization
    ├── browser_window.h    # Window management, address bar, tabs
    ├── browser_window.cpp  # Win32 UI implementation
    ├── cef_handler.h       # CEF client callbacks
    └── cef_handler.cpp     # CEF lifecycle, navigation, error handling
```

## Prerequisites

- Windows 10+ (x86_64)
- Visual Studio 2022+ (MSVC)
- CMake 3.20+
- [CEF Binary Distribution](https://cef-builds.spotifycdn.com/index.html)

## Build

```powershell
# 1. Download and extract CEF binary distribution
#    https://cef-builds.spotifycdn.com/index.html
#    Choose "Standard Distribution" for Windows 64-bit

# 2. Build CEF wrapper
cd <CEF_ROOT>/build
cmake -G "Visual Studio 17 2022" -A x64 ..
cmake --build . --config Release

# 3. Build FeatherSurf
cd shells/windows
mkdir build && cd build
cmake -G "Visual Studio 17 2022" -A x64 -DCEF_ROOT=<CEF_ROOT> ..
cmake --build . --config Release

# 4. Run
.\build\Release\FeatherSurf.exe
```

## Status

- [x] Win32 window creation with DPI awareness
- [x] Address bar with keyboard shortcuts (Ctrl+L)
- [x] Tab strip (Ctrl+T, Ctrl+W, Ctrl+Tab)
- [x] Navigation buttons (back, forward, reload, home)
- [x] Status bar
- [x] Keyboard shortcuts
- [ ] CEF browser integration (requires CEF binary)
- [ ] Rust FFI bridge connection
- [ ] Tab lifecycle management via memory-manager
- [ ] Privacy engine integration

## Keyboard Shortcuts

| Shortcut | Action |
|---|---|
| Ctrl+T | New tab |
| Ctrl+W | Close tab |
| Ctrl+L | Focus address bar |
| Ctrl+R | Reload |
| Ctrl+Tab | Next tab |
| Ctrl+Shift+Tab | Previous tab |
| Alt+Left | Back |
| Alt+Right | Forward |
