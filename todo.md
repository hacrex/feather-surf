# FeatherSurf Project TODO

This document is the execution plan for turning FeatherSurf from an architecture
scaffold into a usable, testable, privacy-focused browser across **Windows, Linux,
macOS, iOS, and Android**. Work proceeds top-down unless a task explicitly depends
on a later platform decision.

## How to use this file

- Mark a task `[x]` only after its acceptance criteria are met.
- Keep one milestone in progress at a time where possible.
- Every implementation task should include tests and documentation before it is marked complete.
- Any task that changes memory behavior must include benchmark results.
- Any task that changes security, privacy, permissions, synchronization, or updates must include a threat-model or security review.
- Platform-specific tasks are tagged: `[win]` `[linux]` `[mac]` `[ios]` `[android]`.

## Platform tier definitions

| Tier | Platforms | Packaging | First-class target |
|------|-----------|-----------|-------------------|
| Tier 1 | Windows 10+, Ubuntu 22.04+, Fedora 38+ | .msi/.exe, .deb/.rpm | Yes |
| Tier 2 | macOS 13+, Debian 12+, Arch | .dmg, .AppImage | Yes |
| Tier 3 | Android 10+, iOS 16+ | .apk, App Store | Yes (later phase) |

## Project-wide definition of done

A milestone is complete only when the code is implemented, automated tests pass,
documentation is updated, CI is green, and the behavior is reproducible on all
tier-1 and tier-2 platforms. Tier-3 platforms must pass CI on supported versions.
A release is complete only when the packaged artifacts are signed, verifiable,
documented, and tested from a clean installation on every target platform.

---

## Milestone 0 — Repository foundation

### 0.1 Make the repository status accurate

- [x] Add a clear project-status section to `README.md`.
- [x] Mark planned and scaffolded features as planned or experimental instead of claiming them as available.
- [x] Add a prominent warning that FeatherSurf is not yet suitable as a daily browser.
- [x] Add a supported-platform policy with tier definitions.
- [x] Add a minimum supported Rust version policy.
- [x] Add `CHANGELOG.md`.
- [x] Add `SECURITY.md` with private vulnerability-reporting instructions.
- [x] Add `SUPPORT.md` explaining where questions and bug reports belong.
- [x] Add `CODEOWNERS` for core, privacy, packaging, and documentation areas.

**Acceptance criteria:** A new contributor can determine the current implementation status, supported platforms, security-reporting path, and expected development workflow from the repository front page.

### 0.2 Normalize Rust workspace metadata

- [x] Rename crate packages to standard lowercase kebab-case names.
- [x] Correct the repository URL in `rust/Cargo.toml`.
- [x] Add workspace descriptions, authors, repository metadata, and `rust-version`.
- [x] Add descriptions and documentation links to each crate.
- [x] Add a root `rust-toolchain.toml` with the supported toolchain.
- [ ] Add formatting and lint configuration where needed.
- [ ] Decide whether all Rust crates should remain independent libraries or whether a shared domain-model crate is needed.
- [ ] Add `target` triples for cross-compilation: `x86_64-pc-windows-msvc`, `x86_64-unknown-linux-gnu`, `aarch64-apple-darwin`, `aarch64-linux-android`, `aarch64-apple-ios`.

**Acceptance criteria:** `cargo metadata`, `cargo fmt --check`, and workspace discovery work from a clean checkout for all target triples.

### 0.3 Add continuous integration

- [x] Add GitHub Actions for formatting, Clippy, tests, and documentation builds.
- [x] `[win]` Add Windows CI runner with MSVC toolchain.
- [x] `[linux]` Add Linux CI runner with GCC/glibc.
- [x] `[mac]` Add macOS CI runner with Xcode toolchain.
- [x] `[android]` Add Android NDK cross-compilation CI job.
- [x] `[ios]` Add iOS cross-compilation CI job (no simulator, build-only).
- [x] Add Markdown link checking.
- [ ] Add dependency license and vulnerability checks (`cargo-audit`, `cargo-deny`).
- [x] Run CI on pull requests and pushes to `main`.
- [x] Cache Rust dependencies per-platform.
- [ ] Upload test and benchmark artifacts from CI.
- [ ] Add a protected `main` branch requiring green CI.

**Acceptance criteria:** A pull request cannot merge when formatting, linting, tests, dependency checks, or documentation checks fail on any tier-1 or tier-2 platform.

### 0.4 Improve contributor workflow

- [x] Add issue templates for bugs, feature requests, benchmark regressions, architecture decisions, and good-first issues.
- [x] Add a pull-request template containing scope, testing, security, and benchmark sections.
- [ ] Create GitHub milestones for each project phase.
- [ ] Label issues by component, platform, difficulty, and security impact.
- [ ] Add at least ten small, actionable starter issues.

**Acceptance criteria:** A contributor can select a scoped task, understand its acceptance criteria, and submit a consistent pull request.

---

## Milestone 1 — Architecture decisions and risk reduction

### 1.1 Select the Chromium integration strategy

- [x] Compare Chromium Embedded Framework, Qt WebEngine, Electron, native Chromium, and WebView alternatives.
- [x] `[win][linux][mac]` Evaluate CEF for desktop platforms.
- [x] `[android]` Evaluate Android WebView or Chromium WebView.
- [x] `[ios]` Evaluate WKWebView (mandatory WebKit on iOS).
- [x] Evaluate extension compatibility, process control, memory telemetry, sandboxing, upgrade burden, licensing, binary size, and platform coverage.
- [x] Record the decision in an architecture decision record (`docs/adr-001-chromium-integration.md`).
- [ ] Define the Chromium version upgrade policy.
- [ ] Define the boundary between the browser shell and Chromium.
- [ ] Define how the Rust FFI bridge connects to each platform's engine.

**Acceptance criteria:** The repository contains a decision record naming the chosen integration strategy per platform, rejected alternatives, trade-offs, and upgrade plan.

### 1.2 Define system interfaces

- [x] Define the browser-shell interface to the tab manager.
- [x] Define the process-monitor interface.
- [x] Define the renderer adapter interface for freeze, suspend, discard, and restore.
- [x] Define the session snapshot format and versioning rules.
- [x] Define the resource telemetry schema.
- [x] Define error-handling and cancellation rules for asynchronous operations.
- [x] Define which operations must run on the UI thread and which may run in worker tasks.
- [x] `[win]` Define COM/WinRT interface boundaries for Windows shell integration.
- [x] `[linux]` Define D-Bus or socket interface for Linux desktop integration.
- [x] `[mac]` Define Objective-C bridging for macOS shell integration.
- [x] `[android]` Define JNI interface for Android Activity ↔ Rust communication.
- [x] `[ios]` Define Swift-ObjC-C++ bridging for iOS shell ↔ Rust communication.

**Acceptance criteria:** Each planned component has a documented input/output contract and does not depend on undocumented global state. Platform bridges are specified.

### 1.3 Create the security model

- [x] Document the threat model for websites, extensions, renderer compromise, local profile theft, network observers, sync-server compromise, malicious updates, and AI providers.
- [x] Document Chromium sandbox assumptions per platform.
- [x] `[win]` Document AppContainer / restricted-token sandbox requirements.
- [x] `[linux]` Document namespace / seccomp / user-namespace sandbox requirements.
- [x] `[mac]` Document seatbelt / hardened-runtime sandbox requirements.
- [x] `[android]` Document Android sandbox and permission model.
- [x] `[ios]` Document iOS WKWebView sandbox and App Store constraints.
- [x] Document profile and credential-storage boundaries.
- [x] Document privacy trade-offs for fingerprint protection.
- [x] Document crash-reporting and telemetry data flows.
- [x] Define security review requirements for high-risk changes.

**Acceptance criteria:** Every security-sensitive subsystem has identified threats, mitigations, limitations, and testing requirements for each supported platform.

---

## Milestone 2 — Domain model and tab lifecycle core

### 2.1 Stabilize the tab state machine

- [x] Define `Active`, `RecentlyActive`, `Background`, `Frozen`, `Suspended`, and `Discardable` states.
- [x] Define legal state transitions.
- [x] Reject illegal transitions without mutating state.
- [x] Add hard protection for pinned tabs.
- [x] Add hard protection for media-playing tabs.
- [x] Add hard protection for dirty forms.
- [x] Add revision tracking.
- [x] Add restorable tab snapshots.
- [ ] Decide whether `Discardable` is a state or a candidate flag in the final design.
- [x] Add serialization and deserialization for `TabSnapshot` (serde + bincode + schema versioning).
- [x] Add snapshot schema versioning and migration tests.
- [ ] Add explicit user overrides for keep-awake and suspend-now behavior.

**Acceptance criteria:** The state machine is independent of Chromium, fully unit-tested, serializable where required, and safe against protected-tab reclamation.

### 2.2 Implement the scoring model

- [x] Implement the keep-resident score `Kᵢ`.
- [x] Implement reclaim pressure `Qᵢ` and budget-adjusted pressure `Q'ᵢ`.
- [x] Clamp every input and output to `[0, 1]`.
- [x] Add deterministic score tests for active, idle, media, pinned, dirty-form, and memory-pressure cases.
- [x] Add configuration for Lite, Balanced, and Performance modes.
- [ ] Define a Rust `TabObservation` type matching `docs/memory-management-policy.md`.
- [ ] Implement normalization functions for activity age, network rate, memory, and CPU.
- [ ] Define behavior for missing, stale, or invalid telemetry.
- [ ] Add normalization tests per platform (different memory units: RSS vs private bytes).

**Acceptance criteria:** The same observations always produce the same score, score behavior is documented, and all policy examples have automated tests.

### 2.3 Implement hysteresis and scheduling policy

- [x] Implement three-sample debounce.
- [x] Implement minimum dwell times.
- [x] Implement separate entry and exit thresholds.
- [x] Implement restoration cooldown.
- [x] Implement budget-pressure override.
- [x] Implement candidate priority ordering.
- [x] Implement manual overrides.
- [x] Add a deterministic fake clock for policy tests.
- [x] Add tests for threshold oscillation.
- [x] Add tests for consecutive-sample requirements.
- [x] Add tests for dwell-time enforcement.
- [x] Add tests for immediate explicit focus restoration.

**Acceptance criteria:** Policy tests demonstrate that a tab does not thrash around thresholds and that user actions override automatic decisions safely.

### 2.4 Add model-level property testing

- [x] Add property tests for legal transition invariants.
- [x] Verify that protected tabs never enter reclaimable states.
- [x] Verify that failed transitions never mutate state or revision.
- [x] Verify that restoration preserves snapshot data.
- [x] Verify that score outputs remain within `[0, 1]` for arbitrary inputs.
- [ ] Verify that hysteresis cannot produce an immediate reclaim/restore loop.

**Acceptance criteria:** Property tests cover the invariants that ordinary example-based tests cannot exhaustively cover.

---

## Milestone 3 — Minimal browser shell

### 3.1 Create the executable shell

- [x] `[win]` Create Win32 window with `CreateWindowExW`, message loop, and DPI awareness.
- [x] `[linux]` Create GTK4 or raw Wayland/X11 window with `xdg-shell`.
- [ ] `[mac]` Create `NSWindow` with `NSApplication` run loop.
- [ ] `[android]` Create `Activity` with `SurfaceView` or `WebView`.
- [ ] `[ios]` Create `UIWindow` with `UIViewController`.
- [x] Add application startup and shutdown handling per platform.
- [x] Add structured logging with privacy-safe defaults (`tracing` crate).
- [ ] Add crash-safe startup recovery (session file validation).
- [x] Add a basic settings directory and profile directory per platform conventions:
  - `[win]` `%LOCALAPPDATA%\FeatherSurf\`
  - `[linux]` `~/.config/feathersurf/`
  - `[mac]` `~/Library/Application Support/FeatherSurf/`
  - `[android]` App internal storage via `Context.getFilesDir()`
  - `[ios]` App sandbox `Documents/` or `Library/`
- [x] Add a single browser window.

**Acceptance criteria:** A clean build opens a native browser window and exits without data corruption on every target platform.

### 3.2 Implement navigation

- [x] Add an address bar.
- [x] Parse and validate URLs.
- [x] Add navigation start, commit, finish, and failure events.
- [x] Add back, forward, reload, and stop controls.
- [x] Display navigation errors without exposing sensitive data in logs.
- [x] Add basic keyboard shortcuts per platform conventions:
  - `[win][linux]` `Ctrl+T`, `Ctrl+W`, `Ctrl+L`, `Ctrl+R`, `Alt+Left`, `Alt+Right`
  - `[mac]` `Cmd+T`, `Cmd+W`, `Cmd+L`, `Cmd+R`, `Cmd+[`, `Cmd+]`
  - `[android][ios]` Gesture-based or toolbar buttons (no physical keyboard assumed).

**Acceptance criteria:** A user can navigate between HTTPS pages, use history controls, and recover from failed navigation.

### 3.3 Connect the tab manager

- [x] Create a tab from the browser shell.
- [x] Associate renderer instances with tab IDs.
- [x] Update snapshots on navigation and relevant page events.
- [x] Send focus, media, form, and activity events to the tab manager via FFI.
- [x] Close tabs and release renderer resources.
- [x] Restore tabs from a saved session.
- [ ] `[android][ios]` Handle back-button / swipe gestures as tab navigation.

**Acceptance criteria:** The shell can create, select, navigate, close, save, and restore tabs using the tab state-machine API.

### 3.4 Add basic browser surfaces

- [x] `[win][linux][mac]` Add a tab strip (horizontal).
- `[android]` Add tab strip or tab overview grid.
- `[ios]` Add tab overview (Safari-style grid).
- [x] Add new-tab and close-tab controls.
- [x] Add a loading indicator.
- [x] Add page-title updates.
- [x] Add window-level error reporting.
- [x] Add a minimal preferences screen.

**Acceptance criteria:** The browser is usable for basic multi-tab navigation without requiring developer tools or manual configuration.

---

## Milestone 4 — Session and profile management

### 4.1 Implement session persistence

- [x] Persist open windows and tabs.
- [x] Persist active tab selection.
- [x] Persist navigation history needed for restoration.
- [x] Persist scroll positions where available.
- [x] Persist form state only when safe and supported.
- [x] Write atomically using temporary files and rename.
- [x] Recover from interrupted writes.
- [x] Add session schema migrations.
- [x] Add corruption recovery and backup rotation.
- [ ] `[android]` Persist across `Activity` recreation (config changes, process death).
- [ ] `[ios]` Persist across app suspension/termination.

**Acceptance criteria:** A forced shutdown does not lose the last valid session, and an incompatible session file is migrated or safely rejected.

### 4.2 Implement profiles and containers

- [x] Define profile directory layout per platform.
- [x] Add profile creation, selection, and deletion safeguards.
- [x] Separate cookies, storage, history, permissions, and extensions by profile.
- [x] Define container-style isolation rules.
- [x] Add profile lock handling for concurrent processes.
- [x] Add tests for cross-profile data isolation.
- [ ] `[android]` Support Android multi-user (work profile, guest).
- [ ] `[ios]` Support iOS app groups for share extension data (if applicable).

**Acceptance criteria:** Data from one profile or container cannot be read by another through normal browser APIs.

### 4.3 Add bookmarks and history

- [x] Define storage schema (in-memory with bincode serialization, ready for SQLite).
- [x] Implement create, edit, delete, and search operations.
- [x] Add import and export formats (HTML, JSON).
- [x] Add retention settings (configurable max history entries).
- [x] Add privacy-safe database migrations.
- [ ] `[win][linux][mac]` Add sidebar or menu UI for bookmarks and history.
- `[android][ios]` Add bookmarks/history screens in settings or toolbar.

**Acceptance criteria:** Bookmarks and history survive restarts, support migration, and respect profile boundaries.

---

## Milestone 5 — Resource monitoring and memory management

### 5.1 Implement process monitoring

- [x] `[win]` Use `OpenProcess`, `QueryFullProcessImageNameW`, job objects for process tracking.
- [x] `[linux]` Parse `/proc/[pid]/stat`, `/proc/[pid]/status`, `/proc/[pid]/smaps`.
- [x] `[mac]` Use `sysctl`, `proc_pidinfo`, `task_info`.
- [x] `[android]` Use `/proc/[pid]/stat` (same as Linux kernel).
- [x] `[ios]` Use `proc_pidinfo` (limited, same kernel as macOS).
- [x] Identify browser, renderer, GPU, utility, and extension processes.
- [x] Map process groups to tabs and profiles.
- [x] Collect RSS, private memory, CPU, and process lifetime.
- [x] Handle processes that disappear during sampling.
- [x] Use platform-specific adapters behind the PAL `ProcessMonitor` trait.
- [x] Add permission and unsupported-platform handling.

**Acceptance criteria:** The monitor reports stable process identities and does not crash when processes start, stop, or restart during sampling.

### 5.2 Implement resource telemetry

- [x] Collect network activity per tab where supported.
- [x] Collect media-playing state.
- [x] Collect active-form and user-interaction signals.
- [x] Add timestamp and freshness metadata to every observation.
- [x] Reject stale observations according to documented limits.
- [x] Add sampling backoff when the browser is idle.
- [x] `[win]` Use `GetNetworkAdapterStatistics` or ETW for network telemetry.
- [x] `[linux]` Parse `/proc/net/dev` or use `getifaddrs`.
- [x] `[mac]` Use `SystemConfiguration.framework` for network stats.
- `[android][ios]` Use engine-reported network stats from WebView/CEF.

**Acceptance criteria:** The scoring engine receives typed, timestamped observations and behaves predictably when data is missing.

### 5.3 Implement freeze, suspend, discard, and restore

- [x] Define renderer adapter commands.
- [x] `[win][linux][mac]` Freeze background execution via CEF/devtools protocol.
- [x] `[android]` Use `WebView.freeze()` (API 33+) or process suspension.
- [x] `[ios]` Use `WKWebView` page pause / process pool snapshot.
- [x] Serialize state before suspension.
- [x] Suspend inactive tabs safely.
- [x] Discard only tabs with valid restoration data or explicit emergency policy.
- [x] Restore renderer state and navigation metadata.
- [x] Handle command timeouts and partial failures.
- [x] Add cancellation for tabs that become active during reclamation.
- [x] `[android]` Handle `onTrimMemory()` signals for system-initiated reclamation.
- [x] `[ios]` Handle `didReceiveMemoryWarning` for system-initiated reclamation.

**Acceptance criteria:** A tab can move through the policy states and return to active browsing without losing its URL or supported restoration state.

### 5.4 Add RAM budgets and user controls

- [x] Support unlimited, fixed, and adaptive memory budgets.
- [x] Validate budgets against system memory.
- [x] `[win][linux][mac]` Query system RAM via `GlobalMemoryStatusEx` / `sysinfo` / `sysctl`.
- [x] `[android]` Query via `ActivityManager.getMemoryInfo()`.
- [x] `[ios]` Query via `NSProcessInfo.physicalMemory`.
- [x] Add Lite, Balanced, and Performance modes.
- [x] Add per-tab keep-awake controls.
- [x] Add a manual suspend action.
- [x] Explain every automatic state change in the UI.
- [x] Add an emergency low-memory mode.
- [ ] `[android]` React to `ComponentCallbacks2.onTrimMemory(TRIM_MEMORY_COMPLETE)`.
- [ ] `[ios]` React to `UIApplication.didReceiveMemoryWarningNotification`.

**Acceptance criteria:** Users can understand and control memory behavior, and automatic reclamation respects configured limits and protections.

### 5.5 Build the Developer Center

- [x] Add browser-wide memory and CPU summaries.
- [x] Add per-tab and per-process resource views.
- [x] Add state-transition history.
- [x] Add current score and policy reason.
- [x] Add exportable diagnostics.
- [x] Redact URLs and content where privacy settings require it.
- [x] `[win][linux][mac]` Show Developer Center in a dedicated tab or window.
- `[android][ios]` Show Developer Center in settings or as a debug overlay.

**Acceptance criteria:** A developer can identify which tabs and processes consume resources and why a tab changed state.

---

## Milestone 6 — Privacy and permissions

### 6.1 Implement request blocking

- [x] Select and document filter-list sources (EasyList, EasyPrivacy, uBlock lists).
- [x] Define update, caching, and rollback behavior for lists.
- [x] Implement tracker and known-malicious-domain blocking in the Rust `privacy-engine`.
- [x] Add per-site allowlists and blocklists.
- [x] Add request-blocking diagnostics.
- [ ] Add filter-list license compliance checks.
- [x] Add performance benchmarks for large lists (100k+ rules).
- [ ] Implement blocking at the network layer for each platform engine.

**Acceptance criteria:** Blocking is enabled by default, explainable to users, configurable per site, and does not silently fail on corrupt updates.

### 6.2 Implement cookie and storage controls

- [x] Restrict third-party cookies by default.
- [x] Add site-specific exceptions.
- [x] Add storage clearing controls.
- [x] Add partitioning behavior where supported by the engine.
- [x] Add tests for cross-site storage isolation.
- [ ] Document compatibility limitations.
- [ ] `[ios]` Handle ITP (Intelligent Tracking Prevention) interactions.

**Acceptance criteria:** Users can inspect and control site storage, and defaults match the published privacy policy.

### 6.3 Implement fingerprinting protections

- [x] Define the fingerprinting threat model.
- [x] Select protections that minimize site breakage.
- [x] Add per-site compatibility exceptions.
- [x] Add a diagnostics page showing active protections.
- [x] Add tests for stable, privacy-preserving behavior.
- [ ] Document what the browser does not protect against.

**Acceptance criteria:** The feature is described accurately and users can disable it per site when necessary.

### 6.4 Implement permission management

- [x] Handle camera, microphone, location, notifications, MIDI, USB, and clipboard permissions.
- [x] Add one-time, session, and persistent permission choices.
- [x] Add per-profile permission storage.
- [x] Add permission reset controls.
- [x] Add tests for denial, expiration, and profile isolation.
- [ ] `[android]` Map to Android runtime permissions (`CAMERA`, `RECORD_AUDIO`, `ACCESS_FINE_LOCATION`, etc.).
- [ ] `[ios]` Map to iOS `Info.plist` usage descriptions and `AVCaptureDevice.requestAccess`.

**Acceptance criteria:** No sensitive permission is granted without an explicit user decision and visible origin information.

---

## Milestone 7 — Extensions and downloads

### 7.1 Implement extension support

- [ ] Define supported manifest versions (Manifest V3).
- [ ] `[win][linux][mac]` Implement Chromium extension API compatibility via CEF.
- [ ] Implement extension installation and removal.
- [ ] Enforce extension permissions.
- [ ] Implement content scripts and background workers supported by the chosen engine.
- [ ] Isolate extension storage by profile.
- [ ] Add extension update verification.
- [ ] Add extension resource accounting.
- [ ] Add an extension management screen.
- [ ] `[android]` Evaluate Chromium extension support on Android WebView (limited).
- [ ] `[ios]` No extension support (iOS WebKit limitations). Document as unsupported.

**Acceptance criteria:** A documented subset of Chromium extensions installs, runs, and is permission-controlled without weakening browser isolation.

### 7.2 Implement downloads

- [x] Add download job state and persistence.
- [x] Add destination selection.
- [x] `[win]` Use Windows Shell `SHGetKnownFolderPath` for Downloads directory.
- `[linux]` Use XDG `xdg-user-dir DOWNLOADS`.
- `[mac]` Use `NSSavePanel` or `~/Downloads`.
- `[android]` Use `Environment.getExternalStoragePublicDirectory(DIRECTORY_DOWNLOADS)` or MediaStore.
- `[ios]` Use `UIDocumentPickerViewController` for save location.
- [x] Add pause, resume, cancel, and retry.
- [x] Validate filenames and prevent path traversal.
- [ ] Add quarantine or malware-scanning integration where available.
- [ ] `[win]` Integrate with Windows Defender SmartScreen.
- `[mac]` Integrate with Gatekeeper / XProtect.
- [x] Add download history and cleanup controls.
- [ ] Add progress and error UI.

**Acceptance criteria:** Downloads are resumable, safely stored, visible to the user, and cannot escape the configured destination through malicious filenames.

---

## Milestone 8 — Developer tools and diagnostics

### 8.1 Resource inspector

- [x] Add process tree visualization.
- [x] Add tab-to-process mapping.
- [x] Add live memory and CPU charts.
- [x] Add state-transition inspection.
- [x] Add snapshot and diagnostic export.
- [ ] Add sampling controls for development builds.

### 8.2 Network and page diagnostics

- [x] Expose request-blocking decisions.
- [x] Add navigation timing data.
- [x] Add cache status.
- [x] Add permission and storage inspection.
- [x] Add safe redaction for exported diagnostics.

### 8.3 Crash and support diagnostics

- [x] Add opt-in crash reporting.
- [ ] `[win]` Collect Windows Error Reporting (WER) minidumps.
- `[linux]` Collect core dumps with `coredumpctl` or `apport`.
- `[mac]` Collect macOS crash reports from `~/Library/Logs/DiagnosticReports`.
- `[android]` Use `android.util.Log` and `Application.getProcessName()` for logcat.
- `[ios]` Use `PLCrashReporter` or equivalent.
- [x] Add local crash dump collection.
- [x] Add privacy review for all diagnostic fields.
- [x] Add a user-facing diagnostic bundle generator.
- [ ] Add a support workflow for reproducible reports.

**Acceptance criteria:** Diagnostic tools help developers reproduce resource, privacy, and navigation issues without collecting unnecessary browsing data.

---

## Milestone 9 — Sync and credential security

### 9.1 Define the synchronization model

- [x] Decide which data types can sync (bookmarks, history, settings, passwords, tabs).
- [x] Define conflict-resolution rules.
- [x] Define device registration and revocation.
- [x] Define offline behavior.
- [x] Define server-side metadata minimization.
- [x] Record the sync architecture decision.
- [ ] `[android][ios]` Define sync behavior under background execution limits.

### 9.2 Implement end-to-end encrypted sync

- [x] Choose audited cryptographic primitives and libraries (`ring`, `rustls`, `age`).
- [x] Define key generation, storage, recovery, and rotation.
- [ ] `[win]` Use Windows Credential Manager or DPAPI for key storage.
- `[linux]` Use `libsecret` / `gnome-keyring` or `kwallet`.
- `[mac]` Use Keychain Services.
- `[android]` Use Android Keystore (`android.security.keystore`).
- `[ios]` Use iOS Keychain (`kSecClassGenericPassword`).
- [x] Encrypt data before it leaves the device.
- [x] Authenticate devices and sync records.
- [ ] Protect against replay and rollback.
- [ ] Add recovery and device-revocation flows.
- [ ] Add interoperability and migration tests.

**Acceptance criteria:** The sync service cannot read synchronized secrets, compromised devices can be revoked, and conflict behavior is deterministic.

### 9.3 Implement password and passkey storage

- [ ] Use the operating-system credential store per platform.
- [ ] Define profile and device key boundaries.
- [ ] Add explicit user confirmation before autofill or save.
- [ ] Add passkey/WebAuthn integration through the selected engine.
- [ ] `[win]` Integrate with Windows Hello / Hello for Business.
- `[mac]` Integrate with Touch ID / Face ID via Keychain.
- `[android]` Integrate with Android BiometricPrompt.
- `[ios]` Integrate with LAContext (Face ID / Touch ID).
- [ ] Add export and deletion safeguards.
- [ ] Add security review and threat-model updates.

**Acceptance criteria:** Credentials are never stored in plaintext application files and all sensitive operations are visible to the user.

---

## Milestone 10 — Optional AI gateway

### 10.1 Define AI privacy and product boundaries

- [x] Keep AI disabled and non-resident by default.
- [x] Define exactly what page data may leave the device.
- [ ] Add provider-specific data-use disclosures.
- [x] Add per-site and per-action consent controls.
- [x] Define local-model resource limits.
- [ ] Add prompt-injection and untrusted-page-content warnings.

### 10.2 Implement the AI gateway

- [x] Define a provider-neutral request interface (Rust trait).
- [x] Implement cloud-provider adapters (OpenAI, Anthropic, local).
- [ ] `[win][linux][mac]` Implement local-provider adapters for Ollama / llama.cpp.
- [ ] `[android]` Implement local-provider via llama.cpp Android bindings or JNI.
- [ ] `[ios]` Implement local-provider via llama.cpp Metal acceleration (if feasible).
- [ ] Add timeout, cancellation, retry, and rate-limit behavior.
- [x] Add model availability and capability reporting.
- [ ] Add resource accounting for local models.
- [ ] Add audit-safe request logging with content disabled by default.

### 10.3 Implement user-facing AI features

- [ ] Add optional page summarization.
- [ ] Add optional selection summarization.
- [ ] Add optional developer assistance.
- [ ] Add clear source-page context boundaries.
- [ ] Add tests preventing unintended page-data transmission.

**Acceptance criteria:** AI is visibly optional, its data flow is understandable, and enabling it does not silently add resident model memory when unused.

---

## Milestone 11 — Benchmarking and performance

### 11.1 Make benchmarks reproducible

- [x] Define exact hardware tiers (4 GB, 8 GB, 16 GB RAM).
- [x] Define supported operating systems and versions per platform.
- [ ] Define browser versions and launch flags.
- [ ] Define test pages and scripted workloads.
- [ ] Define cold-cache and warm-cache procedures.
- [x] Define RSS, PSS, private memory, CPU, startup, restore, and battery metrics.
- [ ] Define warm-up, repetition, and statistical reporting rules.
- [x] Store machine-readable benchmark results (JSON + Markdown).
- [ ] `[android]` Define benchmark procedures for ARM64 devices (Pixel, Samsung, etc.).
- [ ] `[ios]` Define benchmark procedures for A-series / M-series devices.

### 11.2 Build the benchmark harness

- [x] `[win][linux][mac]` Automate browser launch and shutdown via CLI.
- [ ] `[android]` Automate via `adb shell am start` + `adb shell dumpsys meminfo`.
- [ ] `[ios]` Automate via `xcodebuild test` + Instruments.
- [x] Automate tab creation and workload execution.
- [x] Automate state transitions.
- [x] Collect process and browser metrics.
- [x] Collect startup and restore timings.
- [x] Generate Markdown and machine-readable reports.
- [x] Detect regressions against a baseline.

### 11.3 Establish performance budgets

- [x] Set startup-time targets per platform.
- [x] Set idle-memory targets per platform.
- [x] Set per-tab memory targets by workload.
- [x] Set restore-time targets.
- [x] Set background CPU targets.
- [x] Set privacy-filter latency targets.
- [x] Set acceptable overhead for diagnostics and AI-disabled mode.

**Acceptance criteria:** Every performance claim in the README links to reproducible results and no release exceeds the declared budgets without an explicit decision record.

---

## Milestone 12 — Packaging, updates, and release security

### 12.1 Build platform packages

- [x] `[win]` Build `.msi` (WiX) and `.exe` (NSIS or Inno Setup) installers.
- [ ] `[win]` Add file associations and `feathersurf://` URL handler.
- [ ] `[win]` Add Start Menu entries and desktop shortcut options.
- [x] `[linux]` Build `.deb` (Debian/Ubuntu), `.rpm` (Fedora/RHEL), `.AppImage`, `.tar.zst` (Arch).
- [ ] `[linux]` Add `.desktop` file, MIME type associations, icon themes.
- [ ] `[linux]` Add Flatpak and Snap manifests (if desired).
- [x] `[mac]` Build `.dmg` and `.pkg` installers.
- [ ] `[mac]` Add code signing with Apple Developer ID.
- [ ] `[mac]` Add notarization via `notarytool`.
- [x] `[mac]` Add `.app` bundle with `Info.plist`.
- [x] `[android]` Build `.apk` and `.aab` (Android App Bundle).
- [ ] `[android]` Add Play Store listing metadata.
- [x] `[ios]` Build via Xcode, submit to App Store Connect.
- [ ] `[ios]` Add entitlements for Keychain, Background Modes, Associated Domains.
- [x] Add clean-install and uninstall tests.
- [x] Add upgrade and downgrade behavior tests.

### 12.2 Implement secure updates

- [ ] `[win]` Implement auto-update via Sparkle or custom signed-manifest updater.
- [ ] `[linux]` Implement auto-update via package manager (apt/yum) or built-in updater.
- [ ] `[mac]` Implement auto-update via Sparkle framework.
- [ ] `[android]` Implement in-app update via Google Play In-App Updates API.
- [ ] `[ios]` No auto-update mechanism (App Store handles updates).
- [x] Define signed update manifests.
- [x] Use separate package and update-signing keys.
- [ ] Verify signatures before installation.
- [ ] Prevent unauthorized downgrade and rollback.
- [ ] Define key rotation and emergency revocation.
- [x] Add staged rollout support.
- [x] Add update failure recovery.
- [x] Add offline update verification.

**Acceptance criteria:** An attacker who controls the download URL cannot install an unsigned or modified package.

### 12.3 Establish release provenance

- [x] Add reproducible-build documentation.
- [x] Generate SBOMs (Software Bill of Materials).
- [x] Publish checksums and signatures (SHA-256 + GPG/Sigstore).
- [x] Record source commit and toolchain versions.
- [x] Run release builds in isolated CI environments.
- [x] Publish release notes and known limitations.
- [ ] Maintain a security advisory process.

**Acceptance criteria:** Users can verify what source and toolchain produced each release and can validate downloaded artifacts independently.

---

## Milestone 13 — Beta readiness

### 13.1 Reliability testing

- [x] Add long-running soak tests (24h+ idle, 100 tabs).
- [x] Add repeated suspend/restore tests (1000 cycles).
- [x] Add renderer-crash recovery tests.
- [x] Add interrupted-session-write tests (kill -9 / `TerminateProcess`).
- [x] Add low-disk-space tests.
- [x] Add low-memory tests.
- [ ] `[win]` Test under Windows Memory Diagnostic conditions.
- [ ] `[linux]` Test with `cgroups` memory limits and `oom-killer`.
- [ ] `[mac]` Test under macOS memory pressure (memorystatus).
- [ ] `[android]` Test under `onTrimMemory()` escalation levels.
- [ ] `[ios]` Test under `didReceiveMemoryWarning`.
- [x] Add network-loss and captive-portal tests.
- [x] Add concurrent profile and multi-window tests.
- [ ] `[android]` Test across `Activity` lifecycle (rotation, process death, multi-window).
- [ ] `[ios]` Test across app lifecycle (background, suspension, termination).

### 13.2 Compatibility testing

- [x] Test common websites (Google, YouTube, Twitter, GitHub, banking sites, news sites).
- [ ] Test WebAuthn and passkeys.
- [ ] Test video and audio playback.
- [x] Test file uploads and downloads.
- [x] Test notifications and permissions.
- [ ] Test extensions from the documented compatibility set.
- [x] Test accessibility features (screen readers, high contrast, reduced motion).
- [x] Test keyboard-only navigation.
- [ ] `[win][linux][mac]` Test high-DPI and multi-monitor setups.
- `[android]` Test across screen densities and orientations.
- `[ios]` Test across device sizes (iPhone SE → iPhone 15 Pro Max, iPad).
- [ ] `[android]` Test on ARM64 (primary) and x86_64 (emulator).
- [ ] `[ios]` Test on A-series (iPhone) and M-series (iPad) simulators + devices.

### 13.3 Privacy and security review

- [x] Conduct an independent review of profile storage.
- [x] Review network requests made at startup.
- [x] Review telemetry and crash-report defaults.
- [x] Review update verification.
- [x] Review extension isolation.
- [x] Review sync and credential handling.
- [x] Review AI data flows.
- [x] Resolve high-severity findings before beta release.
- [ ] `[android]` Review Android permissions manifest and data-access audit.
- [ ] `[ios]` Review App Transport Security, data protection, and privacy manifest (`NSPrivacyTrackedEntities`).

**Acceptance criteria:** The beta has a documented support matrix, known limitations, reproducible diagnostics, and no unresolved high-severity security issues.

---

## Milestone 14 — Public release and ongoing maintenance

### 14.1 Release the first stable scope

- [ ] Freeze the stable feature set.
- [ ] Remove or clearly label experimental features.
- [ ] Publish installation instructions per platform.
- [ ] Publish privacy policy and security model.
- [ ] Publish benchmark results.
- [ ] Publish support and reporting procedures.
- [ ] Tag and sign the release.
- [ ] Verify installation from a clean machine on every platform.
- [ ] `[win]` Verify on Windows 10 21H2+ and Windows 11.
- [ ] `[linux]` Verify on Ubuntu 22.04, Fedora 38, Debian 12.
- [ ] `[mac]` Verify on macOS 13 Ventura, 14 Sonoma, 15 Sequoia.
- [ ] `[android]` Verify on Android 10–14 (API 29–34).
- [ ] `[ios]` Verify on iOS 16–17.

### 14.2 Operate the project

- [ ] Monitor crash reports and security reports.
- [ ] Track memory and startup regressions.
- [ ] Update Chromium according to the defined cadence.
- [ ] Update filter lists and review licenses.
- [ ] Patch dependencies promptly.
- [ ] Maintain release branches where necessary.
- [ ] Publish monthly project-status updates.
- [ ] Review roadmap priorities using user feedback and benchmark evidence.
- [ ] Maintain platform-specific issue triage (label `platform:win`, `platform:linux`, etc.).

### 14.3 Plan future capabilities

- [ ] Add tab groups and vertical tabs after the core lifecycle is stable.
- [ ] Add picture-in-picture and PWA support.
- [ ] Add reader mode.
- [ ] Add optional VPN integration only after the trust and operational model is defined.
- [ ] Expand developer tooling.
- [ ] Expand local AI support only when resource budgets remain measurable.
- [ ] `[android]` Evaluate Android tablet / foldable optimizations.
- [ ] `[ios]` Evaluate iPad multitasking (Split View, Stage Manager).
- [ ] Evaluate ChromeOS support (Linux binary or Android app).

**Acceptance criteria:** New features do not weaken the core guarantees: predictable resource behavior, privacy by default, explicit user control, and verifiable releases.

---

## Immediate next actions

These are the recommended next five implementation tasks:

1. [ ] Install the pinned Rust toolchain and run the existing tab-manager and memory-manager tests on Windows, Linux, and macOS.
2. [ ] Add `TabSnapshot` serialization (serde + JSON + bincode) with schema versioning.
3. [ ] Add normalization functions for `TabObservation` inputs and property tests via `proptest`.
4. [ ] Add CI for formatting, Clippy, workspace tests, and Markdown validation on all tier-1 platforms.
5. [ ] Decide and document the Chromium integration strategy (CEF vs WebView vs native) before building the browser shell.

---

## References

[1]: README.md "FeatherSurf project overview"
[2]: docs/architecture.md "FeatherSurf architecture"
[3]: docs/roadmap.md "FeatherSurf roadmap"
[4]: docs/memory-management-policy.md "FeatherSurf memory-management scoring policy"
[5]: docs/tab-execution-security.md "FeatherSurf tab execution security and sandboxing"
[6]: CONTRIBUTING.md "FeatherSurf contribution guidelines"
