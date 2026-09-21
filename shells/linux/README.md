# FeatherSurf Linux Shell

GTK4-based browser shell for FeatherSurf on Linux.

## Architecture

```
shells/linux/
├── CMakeLists.txt          # Build configuration (requires GTK4)
├── README.md
└── src/
    ├── main.c              # GTK4 entry point
    ├── browser_window.h    # Window management, address bar, tabs
    ├── browser_window.c    # GTK4 UI implementation
    └── cef_handler.c       # CEF integration skeleton
```

## Prerequisites

- Linux (X11 or Wayland)
- GCC or Clang
- CMake 3.20+
- GTK4 development libraries
- Pango, GDK-Pixbuf, graphene (GTK4 dependencies)

### Install dependencies

**Ubuntu/Debian:**
```bash
sudo apt install build-essential cmake libgtk-4-dev
```

**Fedora:**
```bash
sudo dnf install gcc cmake gtk4-devel
```

**Arch:**
```bash
sudo pacman -S base-devel cmake gtk4
```

## Build

```bash
cd shells/linux
mkdir build && cd build
cmake -DCMAKE_BUILD_TYPE=Release ..
make -j$(nproc)

# Run
./feathersurf
```

### With CEF support

```bash
# Download CEF binary distribution for Linux 64-bit
# https://cef-builds.spotifycdn.com/index.html

cmake -DUSE_CEF=ON -DCEF_ROOT=/path/to/cef ..
make -j$(nproc)
```

## Status

- [x] GTK4 window creation with CSS styling
- [x] Address bar with keyboard shortcuts (Ctrl+L)
- [x] Tab strip (Ctrl+T, Ctrl+W, Ctrl+Tab)
- [x] Navigation buttons (back, forward, reload, home)
- [x] Status bar
- [x] Keyboard shortcuts
- [x] CSS-based tab styling (active/inactive states)
- [ ] CEF/WebKit browser integration
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
| Alt+Left | Back |
| Alt+Right | Forward |

## Supported Distributions

| Distribution | Status |
|---|---|
| Ubuntu 22.04+ | Primary target |
| Fedora 38+ | Primary target |
| Debian 12+ | Primary target |
| Arch Linux | Community tested |
| openSUSE Tumbleweed | Community tested |
