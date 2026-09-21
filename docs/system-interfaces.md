# System Interfaces

This document defines the contracts between FeatherSurf's Rust core and the
browser shell. Every interface is designed to be platform-agnostic — the
platform-specific code lives in the shell and implements these contracts.

## 1. FFI Bridge Contract

The Rust FFI bridge (`rust/ffi`) exposes a C ABI that the browser shell calls.
All functions use opaque handles and return error codes.

### 1.1 Tab Lifecycle

```c
// Create a new tab. Returns NULL on failure.
FfiTab* feathersurf_tab_create(uint64_t id, const char* url);

// Destroy a tab and free its memory.
void feathersurf_tab_destroy(FfiTab* tab);

// Get the current state (0=Active, 1=RecentlyActive, ..., 5=Discardable).
uint32_t feathersurf_tab_state(const FfiTab* tab);

// Transition to a new state. Returns 0 on success.
uint32_t feathersurf_tab_transition(FfiTab* tab, uint32_t target_state);

// Set protection flags.
void feathersurf_tab_set_protection(
    FfiTab* tab,
    bool pinned,
    bool media_playing,
    bool dirty_form
);

// Get the tab's URL. Caller must free with feathersurf_string_free.
const char* feathersurf_tab_url(const FfiTab* tab);

// Get the tab's title. Caller must free with feathersurf_string_free.
char* feathersurf_tab_title(const FfiTab* tab);

// Free a string returned by FFI.
void feathersurf_string_free(char* s);
```

### 1.2 Eviction Manager

```c
// Create a new eviction manager. mode: 0=Lite, 1=Balanced, 2=Performance.
FfiEvictionManager* feathersurf_eviction_create(uint32_t mode);

// Destroy an eviction manager.
void feathersurf_eviction_destroy(FfiEvictionManager* mgr);

// Add a tab to the manager. now = monotonic seconds.
uint32_t feathersurf_eviction_add_tab(
    FfiEvictionManager* mgr,
    FfiTab* tab,
    uint64_t now
);

// Update observations for a tab.
uint32_t feathersurf_eviction_update_observation(
    FfiEvictionManager* mgr,
    uint64_t tab_id,
    float activity,
    float media,
    float user_interaction,
    float network,
    float memory,
    float cpu,
    float pinned,
    float foreground
);

// Run a tick of the eviction engine. Returns number of events.
uint32_t feathersurf_eviction_tick(
    const FfiEvictionManager* mgr,
    uint64_t now
);
```

### 1.3 Shell → Rust Event Flow

The shell must call these FFI functions at specific times:

```
Tab created:
  1. feathersurf_tab_create(id, url)
  2. feathersurf_eviction_add_tab(mgr, tab, now)

Tab focused:
  1. feathersurf_tab_transition(tab, 0)  // Active
  2. feathersurf_eviction_focus_tab(mgr, tab_id, now)

Tab backgrounded:
  1. feathersurf_tab_transition(tab, 1)  // RecentlyActive
  2. (eviction manager handles further transitions)

Tab closed:
  1. feathersurf_tab_destroy(tab)

Periodic tick (every 5 seconds):
  1. feathersurf_eviction_update_observation(mgr, tab_id, ...)
  2. feathersurf_eviction_tick(mgr, now)
  3. Process returned events (transitions, blocks)
```

## 2. Renderer Adapter Interface

The renderer adapter is implemented by each platform shell. It provides
the commands to freeze, suspend, discard, and restore renderer processes.

### 2.1 Trait Definition (Rust)

```rust
pub trait Renderer {
    /// Create a new renderer instance for a tab.
    fn create_renderer(&mut self, tab_id: RendererId, url: &str)
        -> Result<(), RendererError>;

    /// Navigate an existing renderer to a new URL.
    fn navigate(&mut self, id: RendererId, url: &str)
        -> Result<(), RendererError>;

    /// Get the current URL of a renderer.
    fn current_url(&self, id: RendererId) -> Option<String>;

    /// Get the page title of a renderer.
    fn current_title(&self, id: RendererId) -> Option<String>;

    /// Capture the current state for tab suspension/restoration.
    fn capture_state(&self, id: RendererId) -> Option<TabSnapshot>;

    /// Restore a renderer from a captured state.
    fn restore_state(&mut self, id: RendererId, snapshot: &TabSnapshot)
        -> Result<(), RendererError>;

    /// Notify the renderer of a lifecycle state change.
    fn notify_state_change(&mut self, id: RendererId, state: TabState)
        -> Result<(), RendererError>;

    /// Destroy a renderer and release its resources.
    fn destroy_renderer(&mut self, id: RendererId)
        -> Result<(), RendererError>;

    /// Get the OS PID of the renderer process.
    fn renderer_pid(&self, id: RendererId) -> Option<u32>;
}
```

### 2.2 State Change Behavior

Each platform renderer must implement these behaviors:

| State | Renderer Behavior |
|---|---|
| `Active` | Full rendering, network access, JS execution |
| `RecentlyActive` | Keep alive, deprioritize animations/timers |
| `Background` | Throttle animations, reduce JS timer frequency |
| `Frozen` | Pause JavaScript execution, stop network |
| `Suspended` | Serialize state, release renderer process |
| `Discardable` | Mark as discardable under memory pressure |

### 2.3 Platform Implementations

| Platform | Renderer | Freeze Method | Suspend Method |
|---|---|---|---|
| Windows | CEF | `CefBrowserHost::WasHidden(true)` | Process termination + snapshot |
| Linux | CEF | `CefBrowserHost::WasHidden(true)` | Process termination + snapshot |
| macOS | CEF | `CefBrowserHost::WasHidden(true)` | Process termination + snapshot |
| Android | WebView | `WebView.freeze()` (API 33+) | `WebView.onPause()` |
| iOS | WKWebView | `WKWebView.pauseAllMediaPlayback()` | Process pool snapshot |

## 3. Process Monitor Interface

The process monitor tracks renderer processes and their resource usage.

### 3.1 Trait Definition (Rust)

```rust
pub trait ProcessMonitor {
    /// Find all processes matching a name pattern.
    fn find_processes(&self, name_pattern: &str) -> Vec<ProcessInfo>;

    /// Get detailed info for a specific PID.
    fn process_info(&self, pid: u32) -> Option<ProcessInfo>;

    /// Check if a process is still alive.
    fn is_alive(&self, pid: u32) -> bool;

    /// Kill a process.
    fn kill_process(&self, pid: u32) -> Result<(), ProcessError>;

    /// Suspend a process (pause execution).
    /// Windows: SuspendThread; Linux/macOS: SIGSTOP
    fn suspend_process(&self, pid: u32) -> Result<(), ProcessError>;

    /// Resume a previously suspended process.
    /// Windows: ResumeThread; Linux/macOS: SIGCONT
    fn resume_process(&self, pid: u32) -> Result<(), ProcessError>;
}
```

### 3.2 Platform APIs

| Platform | RSS/Private Memory | CPU | Suspend/Resume |
|---|---|---|---|
| Windows | `QueryWorkingSetEx` | `GetProcessTimes` | `SuspendThread`/`ResumeThread` |
| Linux | `/proc/[pid]/smaps` | `/proc/[pid]/stat` | `SIGSTOP`/`SIGCONT` |
| macOS | `task_info` | `proc_pidinfo` | `SIGSTOP`/`SIGCONT` |
| Android | `/proc/[pid]/smaps` | `/proc/[pid]/stat` | `SIGSTOP`/`SIGCONT` |
| iOS | `task_info` | `proc_pidinfo` | Limited (sandboxed) |

## 4. Memory Provider Interface

The memory provider reports system and process memory usage.

### 4.1 Trait Definition (Rust)

```rust
pub trait MemoryProvider {
    /// Get system-wide memory statistics.
    fn system_memory(&self) -> SystemMemory;

    /// Get memory usage for a specific process.
    fn process_memory(&self, pid: u32) -> Option<ProcessMemory>;

    /// Get memory usage for multiple processes at once.
    fn batch_process_memory(&self, pids: &[u32]) -> HashMap<u32, ProcessMemory>;

    /// Get the browser's total working set across all renderer processes.
    fn browser_working_set(&self, pids: &[u32]) -> u64;
}
```

### 4.2 Platform APIs

| Platform | System Memory | Process Memory |
|---|---|---|
| Windows | `GlobalMemoryStatusEx` | `QueryWorkingSetEx` |
| Linux | `/proc/meminfo` | `/proc/[pid]/smaps` |
| macOS | `sysctl hw.memsize` + `host_statistics64` | `task_info` |
| Android | `ActivityManager.getMemoryInfo()` | `/proc/[pid]/smaps` |
| iOS | `NSProcessInfo.physicalMemory` | `task_info` |

## 5. Window Interface

The window interface manages native OS windows.

### 5.1 Trait Definition (Rust)

```rust
pub trait Window {
    /// Create a new browser window.
    fn create_window(&mut self, rect: WindowRect, title: &str)
        -> Result<WindowId, WindowError>;

    /// Show or hide a window.
    fn set_visible(&mut self, id: WindowId, visible: bool)
        -> Result<(), WindowError>;

    /// Set the window title.
    fn set_title(&mut self, id: WindowId, title: &str)
        -> Result<(), WindowError>;

    /// Set the window bounds.
    fn set_rect(&mut self, id: WindowId, rect: WindowRect)
        -> Result<(), WindowError>;

    /// Get the current window bounds.
    fn get_rect(&self, id: WindowId) -> Option<WindowRect>;

    /// Minimize the window.
    fn minimize(&mut self, id: WindowId) -> Result<(), WindowError>;

    /// Maximize or restore the window.
    fn maximize_or_restore(&mut self, id: WindowId) -> Result<(), WindowError>;

    /// Close and destroy a window.
    fn close_window(&mut self, id: WindowId) -> Result<(), WindowError>;

    /// Enter fullscreen mode.
    fn enter_fullscreen(&mut self, id: WindowId) -> Result<(), WindowError>;

    /// Exit fullscreen mode.
    fn exit_fullscreen(&mut self, id: WindowId) -> Result<(), WindowError>;
}
```

## 6. Telemetry Schema

The telemetry schema defines what observations are sent to the scoring engine.

### 6.1 TabObservation

```rust
pub struct TabObservation {
    /// Recent user activity (0.0 = none, 1.0 = active interaction).
    pub activity: f64,

    /// Media playing state (0.0 = silent, 1.0 = audible audio/video).
    pub media: f64,

    /// Unsaved user input (0.0 = none, 1.0 = dirty form/active edit).
    pub user_interaction: f64,

    /// Network activity (0.0 = idle, 1.0 = 1+ MiB/s).
    pub network: f64,

    /// Memory pressure (0.0 = minimal, 1.0 = at target limit).
    pub memory: f64,

    /// CPU activity (0.0 = idle, 1.0 = 100% CPU).
    pub cpu: f64,

    /// Pinned status (0.0 = unpinned, 1.0 = pinned).
    pub pinned: f64,

    /// Foreground status (0.0 = background, 1.0 = selected/active tab).
    pub foreground: f64,
}
```

### 6.2 Observation Sources

| Field | Source | Sampling Rate |
|---|---|---|
| `activity` | Last user input timestamp | On change |
| `media` | Audio/video element state | 1 Hz |
| `user_interaction` | Form/input element state | On change |
| `network` | Network bytes transferred | 1 Hz |
| `memory` | Process RSS/private bytes | 5 s |
| `cpu` | Process CPU time | 5 s |
| `pinned` | Tab pinned state | On change |
| `foreground` | Tab focus state | On change |

## 7. Error Handling Rules

### 7.1 FFI Error Codes

| Code | Meaning |
|---|---|
| 0 | Success |
| 1 | Null pointer argument |
| 2 | Invalid enum value |
| 3 | Operation failed (transition rejected, etc.) |
| 4 | Tab not found |
| 5 | Permission denied |

### 7.2 Async Operation Rules

- All FFI calls are synchronous and must complete quickly (< 1 ms).
- Long-running operations (navigation, downloads) use callbacks.
- The eviction tick must complete within 100 ms for 100 tabs.
- Cancellation is cooperative: check `tab.state()` after async work.

### 7.3 Thread Safety

- FFI functions are safe to call from any thread.
- The `EvictionManager` uses `Mutex` internally.
- The shell must not call FFI functions concurrently from multiple threads
  without external synchronization.
- The eviction tick should run on a dedicated background thread.

## 8. Platform Bridge Summary

| Component | Windows | Linux | macOS | Android | iOS |
|---|---|---|---|---|---|
| Window | Win32 `CreateWindowExW` | GTK4 / Wayland | `NSWindow` | Activity | UIViewController |
| Renderer | CEF | CEF | CEF | WebView | WKWebView |
| Process | Job Objects | `/proc` | `sysctl` | `/proc` | `proc_pidinfo` |
| Memory | `QueryWorkingSetEx` | `/proc/[pid]/smaps` | `task_info` | `/proc/[pid]/smaps` | `task_info` |
| FFI | C ABI (cdylib) | C ABI (cdylib) | C ABI (cdylib) | JNI | Swift bridging |
| Key Storage | DPAPI | libsecret | Keychain | Android Keystore | iOS Keychain |
