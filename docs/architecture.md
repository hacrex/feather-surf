# Architecture

## 1. Engine strategy

FeatherSurf does not try to combine Blink, Gecko, and WebKit into one
engine. That is not a realistic scope for an open-source project, and
Safari in particular contains proprietary Apple components that cannot be
reproduced.

Instead, FeatherSurf builds on **Chromium/Blink**, which already provides:

- Blink rendering engine + V8 JavaScript engine
- Modern web standards support
- The Chrome extensions ecosystem
- DevTools
- Site compatibility
- A multi-process architecture
- A large upstream community

FeatherSurf's job is the **shell and resource-management layer** around it.

## 2. Resource Intelligence Layer

This is the core of the project — the piece that does not exist in
mainstream browsers in this form.

Every tab carries a state:

```
Tab
 ├── Active
 ├── Recently Active
 ├── Background
 ├── Frozen
 ├── Suspended
 └── Discardable
```

A continuously recalculated score decides transitions:

```
Tab Score =
    activity
  + audio/video
  + user interaction
  + network activity
  + memory usage
  + CPU usage
  + pinned status
```

```
High score      → Keep alive
Medium score    → Freeze
Low score       → Suspend
Very low score  → Discard
```

All of this is transparent and user-configurable — never a silent black box.

## 3. Adaptive memory modes

| Mode | Target | Behavior |
|---|---|---|
| 🪶 Lite | 4–8 GB | Aggressive tab freezing, aggressive image optimization, limited background JS, discard inactive tabs, reduced preloading/cache |
| ⚖️ Balanced | 8–16 GB | Normal browsing with intelligent memory management |
| 🚀 Performance | 16 GB+ | More caching, more preloading, less aggressive suspension |

## 4. RAM budget

Users can cap total browser memory (e.g. 2/4/6/8 GB, or unlimited), and the
resource manager actively works to stay within that ceiling. Particularly
useful for students, developers, older laptops, VMs, and cloud desktops.

## 5. State preservation

Suspending a tab must never feel like losing it. FeatherSurf preserves:

- URL
- Scroll position
- Form state where possible
- Session state
- Tab history

so a restored tab feels close to instant.

## 6. Rust-native components

Rust is used for the resource-management layer, not to rewrite Chromium:

```
rust/
├── memory-manager
├── tab-manager
├── process-manager
├── resource-monitor
├── privacy-engine
├── cache-manager
├── download-manager
└── telemetry
```

Rust fits because these components need low overhead, memory safety,
concurrency, predictable resource management, and long process lifetimes.

## 7. AI gateway (optional, non-resident by default)

```
                    Browser
                       │
                 AI Gateway
                       │
             ┌─────────┴─────────┐
             ▼                   ▼
        Cloud AI             Local AI
                               │
                         Ollama / llama.cpp
```

`AI = OFF` means zero extra model RAM. Local AI, when enabled, runs a small
quantized model behind a Local AI Gateway rather than being embedded
directly in the browser process.

## 8. Extension compatibility

Chromium extension compatibility is the starting point:

```
Extension
    │
    ▼
Extension API Layer
    │
    ├── Manifest
    ├── Content Scripts
    ├── Background Workers
    ├── Storage
    └── Permissions
```

`chrome.*` / `browser.*` APIs are exposed where licensing and implementation
allow.

## 9. Privacy defaults (opt-out, not opt-in)

| Setting | Default |
|---|---|
| Third-party trackers | Block |
| Known malicious domains | Block |
| Fingerprinting | Reduce |
| Third-party cookies | Restrict |
| Telemetry | Off |
| Crash reporting | Opt-in |
| AI data collection | Off |

No mandatory telemetry, ever.

## 10. Developer Center

A live breakdown, not a single aggregate number:

```
Browser Memory

Browser Core       210 MB
Tab: GitHub        180 MB
Tab: YouTube       310 MB
Extension           42 MB
Renderer            95 MB
GPU                 120 MB
-------------------------
Total              957 MB
```
