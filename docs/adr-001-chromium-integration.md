# ADR-001: Chromium Integration Strategy

## Status

Proposed

## Context

FeatherSurf needs to integrate with Chromium/Blink to provide a modern web
browsing experience. The integration strategy determines:

- How the browser shell is built
- How extension compatibility is achieved
- How process control and memory telemetry are obtained
- How sandboxing and security are maintained
- The upgrade burden and maintenance cost
- Binary size and platform coverage
- Licensing constraints

FeatherSurf's primary differentiator is the Resource Intelligence Layer — a
Rust-native memory management system that requires deep visibility into
renderer processes (memory usage, CPU, tab-to-process mapping, freeze/suspend
primitives).

## Decision Options

### Option 1: Chromium Embedded Framework (CEF)

CEF is a C/C++ framework that wraps Chromium with a stable API. It provides
browser controls, DevTools, extensions, and multi-process architecture in a
reusable library.

**Pros:**
- Mature, widely used (Electron is based on it)
- Stable C API with ABI compatibility across versions
- Built-in extension support (Chromium extensions work)
- Process model exposed via C API
- Active community, regular releases
- Supports Windows, macOS, Linux
- Permissive BSD-style license
- Well-documented upgrade path between versions

**Cons:**
- Binary size: ~150-200 MB minimum
- C API requires FFI bindings from Rust
- Limited control over Chromium internals compared to source integration
- Memory telemetry requires platform-specific queries (not fully exposed by CEF)
- CEF version lags behind Chromium stable by weeks

### Option 2: Qt WebEngine

Qt WebEngine embeds Chromium via the Qt framework. It provides a
QWebEngineView widget that can be embedded in Qt applications.

**Pros:**
- Cross-platform (Windows, macOS, Linux)
- Good integration with Qt UI framework
- Extension support via Chromium's extension system
- Qt's signal/slot mechanism for event handling

**Cons:**
- Qt's LGPL/GPL licensing complicates commercial distribution
- Qt is a large dependency (~500 MB+ development, ~100 MB+ runtime)
- Limited process control compared to raw CEF or Chromium source
- Memory telemetry requires Qt-specific workarounds
- Tightly coupled to Qt's event loop and threading model
- Rust integration via C++ FFI is complex

### Option 3: Electron

Electron bundles Chromium and Node.js into a desktop application framework.
Apps are built with HTML/CSS/JavaScript.

**Pros:**
- Simple development model (web technologies)
- Large ecosystem of tools and libraries
- Extension support via Chrome APIs
- Cross-platform

**Cons:**
- Massive memory overhead (Node.js + Chromium + V8 + your app)
- Conflicts with FeatherSurf's core philosophy: "Never solve a software
  problem by simply demanding more RAM"
- Node.js adds ~50-100 MB baseline memory
- Limited native process control
- JavaScript-based shell means the UI itself consumes significant memory
- Not suitable for a RAM-efficient browser

### Option 4: Native Chromium (Source Integration)

Build directly from Chromium source, modifying the browser shell.

**Pros:**
- Maximum control over every aspect of Chromium
- Direct access to renderer process internals
- Can modify Chromium's memory management at the source level
- Full extension API surface
- Best possible memory telemetry

**Cons:**
- Enormous build complexity (Chromium build requires 16+ GB RAM, hours to build)
- Must maintain a fork or track upstream with heavy merge conflicts
- Chromium's own license (BSD) requires careful compliance
- Upgrade burden is extreme (Chromium releases every 4 weeks)
- Requires dedicated build infrastructure
- Binary size: 300+ MB minimum
- Platform-specific build configurations for each target

### Option 5: CEF with Custom Process Bridge

Use CEF as the browser engine but build a custom native bridge to Chromium's
process management APIs for memory telemetry and lifecycle control.

**Pros:**
- CEF's stable API for browser controls and extensions
- Custom bridge provides access to Chromium's process metrics
- Can implement freeze/suspend via Chromium's process API
- Moderate binary size (~150-200 MB)
- CEF's BSD license is permissive
- Upgrade path is well-defined (CEF tracks Chromium)
- Rust FFI bindings are straightforward

**Cons:**
- Requires maintaining the custom bridge code
- Bridge must be updated when CEF/Chromium changes process APIs
- Some process controls may not be available via CEF's API

## Decision

**Recommended: Option 1 — Chromium Embedded Framework (CEF)**

### Rationale

1. **Memory control**: CEF exposes process creation, destruction, and basic
   lifecycle hooks. Combined with platform-specific process monitoring (which
   the Rust `process-manager` and `resource-monitor` crates will handle),
   this provides sufficient visibility for the Resource Intelligence Layer.

2. **Extension compatibility**: CEF supports Chromium extensions natively,
   which is a hard requirement.

3. **Upgrade burden**: CEF releases follow Chromium stable with a 2-4 week
   lag. Upgrading is well-documented and mechanical.

4. **Binary size**: ~150-200 MB is acceptable for a desktop browser. This
   can be reduced with build flags.

5. **Licensing**: CEF uses a BSD-style license, compatible with Apache 2.0.

6. **Platform coverage**: CEF supports Windows, macOS, and Linux — matching
   FeatherSurf's target platforms.

7. **Rust integration**: CEF's C API can be wrapped with `bindgen` or
   manually. The existing Rust crates (`tab-manager`, `memory-manager`,
   etc.) are designed to be engine-agnostic, so they don't need modification.

### Why Not Option 5 (CEF + Custom Process Bridge)?

While Option 5 offers more control, it adds maintenance burden without
significant benefit for Phase 1-2. CEF's process API provides enough
information for the initial Resource Intelligence Layer. If deeper process
control is needed later, the bridge can be added incrementally.

## Implementation Plan

### Phase 1: CEF Shell

1. Integrate CEF as a submodule or prebuilt binary
2. Create a Rust FFI layer for CEF's C API
3. Implement browser window, tabs, address bar, navigation
4. Connect `tab-manager` to CEF's tab lifecycle
5. Implement basic bookmarks, history, downloads

### Phase 2: Process Monitoring

1. Use platform APIs (Windows: `QueryWorkingSetEx`, Linux: `/proc/[pid]/smaps`)
   to monitor renderer process memory
2. Map CEF process IDs to tab IDs
3. Feed observations to `memory-manager`'s scoring engine
4. Implement freeze/suspend via CEF's process API

### Phase 3: Extension Support

1. Enable CEF's extension system
2. Map `chrome.*` APIs to CEF's extension API surface
3. Implement extension permissions and isolation

## Alternatives Considered and Rejected

| Option | Reason for Rejection |
|---|---|
| Qt WebEngine | Licensing complexity, Qt dependency weight |
| Electron | Memory overhead contradicts core philosophy |
| Native Chromium | Build complexity beyond project resources |
| CEF + Custom Bridge | Premature optimization; can be added later |

## References

- [CEF Documentation](https://cef-builds.spotifycdn.com/docs/index.html)
- [CEF GitHub](https://github.com/nicedoc/cef)
- [Chromium Embedded Framework](https://en.wikipedia.org/wiki/Chromium_Embedded_Framework)
- [FeatherSurf Architecture](docs/architecture.md)
- [FeatherSurf Roadmap](docs/roadmap.md)
