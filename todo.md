# FeatherSurf Project TODO

This document is the execution plan for turning FeatherSurf from an architecture scaffold into a usable, testable, privacy-focused browser. Work should proceed from the top of the document downward unless a task explicitly depends on a later platform decision.

## How to use this file

- Mark a task `[x]` only after its acceptance criteria are met.
- Keep one milestone in progress at a time where possible.
- Every implementation task should include tests and documentation before it is marked complete.
- Any task that changes memory behavior must include benchmark results.
- Any task that changes security, privacy, permissions, synchronization, or updates must include a threat-model or security review.

## Project-wide definition of done

A milestone is complete only when the code is implemented, automated tests pass, documentation is updated, CI is green, and the behavior is reproducible on the supported platforms. A release is complete only when the packaged artifacts are signed, verifiable, documented, and tested from a clean installation.

---

## Milestone 0 — Repository foundation

### 0.1 Make the repository status accurate

- [ ] Add a clear project-status section to `README.md`.
- [ ] Mark planned and scaffolded features as planned or experimental instead of claiming them as available.
- [ ] Add a prominent warning that FeatherSurf is not yet suitable as a daily browser.
- [ ] Add a supported-platform policy.
- [ ] Add a minimum supported Rust version policy.
- [ ] Add `CHANGELOG.md`.
- [ ] Add `SECURITY.md` with private vulnerability-reporting instructions.
- [ ] Add `SUPPORT.md` explaining where questions and bug reports belong.
- [ ] Add `CODEOWNERS` for core, privacy, packaging, and documentation areas.

**Acceptance criteria:** A new contributor can determine the current implementation status, supported platforms, security-reporting path, and expected development workflow from the repository front page.

### 0.2 Normalize Rust workspace metadata

- [ ] Rename crate packages to standard lowercase kebab-case names.
- [ ] Correct the repository URL in `rust/Cargo.toml`.
- [ ] Add workspace descriptions, authors, repository metadata, and `rust-version`.
- [ ] Add descriptions and documentation links to each crate.
- [ ] Add a root `rust-toolchain.toml` with the supported toolchain.
- [ ] Add formatting and lint configuration where needed.
- [ ] Decide whether all Rust crates should remain independent libraries or whether a shared domain-model crate is needed.

**Acceptance criteria:** `cargo metadata`, `cargo fmt --check`, and workspace discovery work from a clean checkout.

### 0.3 Add continuous integration

- [ ] Add GitHub Actions for formatting, Clippy, tests, and documentation builds.
- [ ] Add Markdown link checking.
- [ ] Add dependency license and vulnerability checks.
- [ ] Run CI on pull requests and pushes to `main`.
- [ ] Cache Rust dependencies safely.
- [ ] Upload test and benchmark artifacts from CI.
- [ ] Add a protected `main` branch requiring green CI.

**Acceptance criteria:** A pull request cannot merge when formatting, linting, tests, dependency checks, or documentation checks fail.

### 0.4 Improve contributor workflow

- [ ] Add issue templates for bugs, feature requests, benchmark regressions, architecture decisions, and good-first issues.
- [ ] Add a pull-request template containing scope, testing, security, and benchmark sections.
- [ ] Create GitHub milestones for each project phase.
- [ ] Label issues by component, platform, difficulty, and security impact.
- [ ] Add at least ten small, actionable starter issues.

**Acceptance criteria:** A contributor can select a scoped task, understand its acceptance criteria, and submit a consistent pull request.

---

## Milestone 1 — Architecture decisions and risk reduction

### 1.1 Select the Chromium integration strategy

- [ ] Compare Chromium Embedded Framework, Qt WebEngine, Electron, native Chromium, and WebView alternatives.
- [ ] Evaluate extension compatibility, process control, memory telemetry, sandboxing, upgrade burden, licensing, binary size, and platform coverage.
- [ ] Record the decision in an architecture decision record.
- [ ] Define the Chromium version upgrade policy.
- [ ] Define the boundary between the browser shell and Chromium.

**Acceptance criteria:** The repository contains a decision record naming the chosen integration strategy, rejected alternatives, trade-offs, and upgrade plan.

### 1.2 Define system interfaces

- [ ] Define the browser-shell interface to the tab manager.
- [ ] Define the process-monitor interface.
- [ ] Define the renderer adapter interface for freeze, suspend, discard, and restore.
- [ ] Define the session snapshot format and versioning rules.
- [ ] Define the resource telemetry schema.
- [ ] Define error-handling and cancellation rules for asynchronous operations.
- [ ] Define which operations must run on the UI thread and which may run in worker tasks.

**Acceptance criteria:** Each planned component has a documented input/output contract and does not depend on undocumented global state.

### 1.3 Create the security model

- [ ] Document the threat model for websites, extensions, renderer compromise, local profile theft, network observers, sync-server compromise, malicious updates, and AI providers.
- [ ] Document Chromium sandbox assumptions.
- [ ] Document profile and credential-storage boundaries.
- [ ] Document privacy trade-offs for fingerprint protection.
- [ ] Document crash-reporting and telemetry data flows.
- [ ] Define security review requirements for high-risk changes.

**Acceptance criteria:** Every security-sensitive subsystem has identified threats, mitigations, limitations, and testing requirements.

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
- [ ] Add serialization and deserialization for `TabSnapshot`.
- [ ] Add snapshot schema versioning and migration tests.
- [ ] Add explicit user overrides for keep-awake and suspend-now behavior.

**Acceptance criteria:** The state machine is independent of Chromium, fully unit-tested, serializable where required, and safe against protected-tab reclamation.

### 2.2 Implement the scoring model

- [ ] Define a Rust `TabObservation` type matching `docs/memory-management-policy.md`.
- [ ] Implement normalization functions for activity age, network rate, memory, and CPU.
- [ ] Implement the keep-resident score `Kᵢ`.
- [ ] Implement reclaim pressure `Qᵢ` and budget-adjusted pressure `Q'ᵢ`.
- [ ] Clamp every input and output to `[0, 1]`.
- [ ] Define behavior for missing, stale, or invalid telemetry.
- [ ] Add deterministic score tests for active, idle, media, pinned, dirty-form, and memory-pressure cases.
- [ ] Add configuration for Lite, Balanced, and Performance modes.

**Acceptance criteria:** The same observations always produce the same score, score behavior is documented, and all policy examples have automated tests.

### 2.3 Implement hysteresis and scheduling policy

- [ ] Implement three-sample debounce.
- [ ] Implement minimum dwell times.
- [ ] Implement separate entry and exit thresholds.
- [ ] Implement restoration cooldown.
- [ ] Implement budget-pressure override.
- [ ] Implement candidate priority ordering.
- [ ] Implement manual overrides.
- [ ] Add a deterministic fake clock for policy tests.
- [ ] Add tests for threshold oscillation.
- [ ] Add tests for consecutive-sample requirements.
- [ ] Add tests for dwell-time enforcement.
- [ ] Add tests for immediate explicit focus restoration.

**Acceptance criteria:** Policy tests demonstrate that a tab does not thrash around thresholds and that user actions override automatic decisions safely.

### 2.4 Add model-level property testing

- [ ] Add property tests for legal transition invariants.
- [ ] Verify that protected tabs never enter reclaimable states.
- [ ] Verify that failed transitions never mutate state or revision.
- [ ] Verify that restoration preserves snapshot data.
- [ ] Verify that score outputs remain within `[0, 1]` for arbitrary inputs.
- [ ] Verify that hysteresis cannot produce an immediate reclaim/restore loop.

**Acceptance criteria:** Property tests cover the invariants that ordinary example-based tests cannot exhaustively cover.

---

## Milestone 3 — Minimal browser shell

### 3.1 Create the executable shell

- [ ] Create the selected platform window.
- [ ] Add application startup and shutdown handling.
- [ ] Add structured logging with privacy-safe defaults.
- [ ] Add crash-safe startup recovery.
- [ ] Add a basic settings directory and profile directory.
- [ ] Add a single browser window.

**Acceptance criteria:** A clean build opens a native browser window and exits without data corruption.

### 3.2 Implement navigation

- [ ] Add an address bar.
- [ ] Parse and validate URLs.
- [ ] Add navigation start, commit, finish, and failure events.
- [ ] Add back, forward, reload, and stop controls.
- [ ] Display navigation errors without exposing sensitive data in logs.
- [ ] Add basic keyboard shortcuts.

**Acceptance criteria:** A user can navigate between HTTPS pages, use history controls, and recover from failed navigation.

### 3.3 Connect the tab manager

- [ ] Create a tab from the browser shell.
- [ ] Associate renderer instances with tab IDs.
- [ ] Update snapshots on navigation and relevant page events.
- [ ] Send focus, media, form, and activity events to the tab manager.
- [ ] Close tabs and release renderer resources.
- [ ] Restore tabs from a saved session.

**Acceptance criteria:** The shell can create, select, navigate, close, save, and restore tabs using the tab state-machine API.

### 3.4 Add basic browser surfaces

- [ ] Add a tab strip.
- [ ] Add new-tab and close-tab controls.
- [ ] Add a loading indicator.
- [ ] Add page-title updates.
- [ ] Add window-level error reporting.
- [ ] Add a minimal preferences screen.

**Acceptance criteria:** The browser is usable for basic multi-tab navigation without requiring developer tools or manual configuration.

---

## Milestone 4 — Session and profile management

### 4.1 Implement session persistence

- [ ] Persist open windows and tabs.
- [ ] Persist active tab selection.
- [ ] Persist navigation history needed for restoration.
- [ ] Persist scroll positions where available.
- [ ] Persist form state only when safe and supported.
- [ ] Write atomically using temporary files and rename.
- [ ] Recover from interrupted writes.
- [ ] Add session schema migrations.
- [ ] Add corruption recovery and backup rotation.

**Acceptance criteria:** A forced shutdown does not lose the last valid session, and an incompatible session file is migrated or safely rejected.

### 4.2 Implement profiles and containers

- [ ] Define profile directory layout.
- [ ] Add profile creation, selection, and deletion safeguards.
- [ ] Separate cookies, storage, history, permissions, and extensions by profile.
- [ ] Define container-style isolation rules.
- [ ] Add profile lock handling for concurrent processes.
- [ ] Add tests for cross-profile data isolation.

**Acceptance criteria:** Data from one profile or container cannot be read by another through normal browser APIs.

### 4.3 Add bookmarks and history

- [ ] Define storage schema.
- [ ] Implement create, edit, delete, and search operations.
- [ ] Add import and export formats.
- [ ] Add retention settings.
- [ ] Add privacy-safe database migrations.
- [ ] Add UI for bookmarks and history.

**Acceptance criteria:** Bookmarks and history survive restarts, support migration, and respect profile boundaries.

---

## Milestone 5 — Resource monitoring and memory management

### 5.1 Implement process monitoring

- [ ] Identify browser, renderer, GPU, utility, and extension processes.
- [ ] Map process groups to tabs and profiles.
- [ ] Collect RSS, private memory, CPU, and process lifetime.
- [ ] Handle processes that disappear during sampling.
- [ ] Use platform-specific adapters behind a shared interface.
- [ ] Add permission and unsupported-platform handling.

**Acceptance criteria:** The monitor reports stable process identities and does not crash when processes start, stop, or restart during sampling.

### 5.2 Implement resource telemetry

- [ ] Collect network activity per tab where supported.
- [ ] Collect media-playing state.
- [ ] Collect active-form and user-interaction signals.
- [ ] Add timestamp and freshness metadata to every observation.
- [ ] Reject stale observations according to documented limits.
- [ ] Add sampling backoff when the browser is idle.

**Acceptance criteria:** The scoring engine receives typed, timestamped observations and behaves predictably when data is missing.

### 5.3 Implement freeze, suspend, discard, and restore

- [ ] Define renderer adapter commands.
- [ ] Freeze background execution where supported.
- [ ] Serialize state before suspension.
- [ ] Suspend inactive tabs safely.
- [ ] Discard only tabs with valid restoration data or explicit emergency policy.
- [ ] Restore renderer state and navigation metadata.
- [ ] Handle command timeouts and partial failures.
- [ ] Add cancellation for tabs that become active during reclamation.

**Acceptance criteria:** A tab can move through the policy states and return to active browsing without losing its URL or supported restoration state.

### 5.4 Add RAM budgets and user controls

- [ ] Support unlimited, fixed, and adaptive memory budgets.
- [ ] Validate budgets against system memory.
- [ ] Add Lite, Balanced, and Performance modes.
- [ ] Add per-tab keep-awake controls.
- [ ] Add a manual suspend action.
- [ ] Explain every automatic state change in the UI.
- [ ] Add an emergency low-memory mode.

**Acceptance criteria:** Users can understand and control memory behavior, and automatic reclamation respects configured limits and protections.

### 5.5 Build the Developer Center

- [ ] Add browser-wide memory and CPU summaries.
- [ ] Add per-tab and per-process resource views.
- [ ] Add state-transition history.
- [ ] Add current score and policy reason.
- [ ] Add exportable diagnostics.
- [ ] Redact URLs and content where privacy settings require it.

**Acceptance criteria:** A developer can identify which tabs and processes consume resources and why a tab changed state.

---

## Milestone 6 — Privacy and permissions

### 6.1 Implement request blocking

- [ ] Select and document filter-list sources.
- [ ] Define update, caching, and rollback behavior for lists.
- [ ] Implement tracker and known-malicious-domain blocking.
- [ ] Add per-site allowlists and blocklists.
- [ ] Add request-blocking diagnostics.
- [ ] Add filter-list license compliance checks.
- [ ] Add performance benchmarks for large lists.

**Acceptance criteria:** Blocking is enabled by default, explainable to users, configurable per site, and does not silently fail on corrupt updates.

### 6.2 Implement cookie and storage controls

- [ ] Restrict third-party cookies by default.
- [ ] Add site-specific exceptions.
- [ ] Add storage clearing controls.
- [ ] Add partitioning behavior where supported.
- [ ] Add tests for cross-site storage isolation.
- [ ] Document compatibility limitations.

**Acceptance criteria:** Users can inspect and control site storage, and defaults match the published privacy policy.

### 6.3 Implement fingerprinting protections

- [ ] Define the fingerprinting threat model.
- [ ] Select protections that minimize site breakage.
- [ ] Add per-site compatibility exceptions.
- [ ] Add a diagnostics page showing active protections.
- [ ] Add tests for stable, privacy-preserving behavior.
- [ ] Document what the browser does not protect against.

**Acceptance criteria:** The feature is described accurately and users can disable it per site when necessary.

### 6.4 Implement permission management

- [ ] Handle camera, microphone, location, notifications, MIDI, USB, and clipboard permissions.
- [ ] Add one-time, session, and persistent permission choices.
- [ ] Add per-profile permission storage.
- [ ] Add permission reset controls.
- [ ] Add tests for denial, expiration, and profile isolation.

**Acceptance criteria:** No sensitive permission is granted without an explicit user decision and visible origin information.

---

## Milestone 7 — Extensions and downloads

### 7.1 Implement extension support

- [ ] Define supported manifest versions.
- [ ] Implement extension installation and removal.
- [ ] Enforce extension permissions.
- [ ] Implement content scripts and background workers supported by the chosen engine.
- [ ] Isolate extension storage by profile.
- [ ] Add extension update verification.
- [ ] Add extension resource accounting.
- [ ] Add an extension management screen.

**Acceptance criteria:** A documented subset of Chromium extensions installs, runs, and is permission-controlled without weakening browser isolation.

### 7.2 Implement downloads

- [ ] Add download job state and persistence.
- [ ] Add destination selection.
- [ ] Add pause, resume, cancel, and retry.
- [ ] Validate filenames and prevent path traversal.
- [ ] Add quarantine or malware-scanning integration where available.
- [ ] Add download history and cleanup controls.
- [ ] Add progress and error UI.

**Acceptance criteria:** Downloads are resumable, safely stored, visible to the user, and cannot escape the configured destination through malicious filenames.

---

## Milestone 8 — Developer tools and diagnostics

### 8.1 Resource inspector

- [ ] Add process tree visualization.
- [ ] Add tab-to-process mapping.
- [ ] Add live memory and CPU charts.
- [ ] Add state-transition inspection.
- [ ] Add snapshot and diagnostic export.
- [ ] Add sampling controls for development builds.

### 8.2 Network and page diagnostics

- [ ] Expose request-blocking decisions.
- [ ] Add navigation timing data.
- [ ] Add cache status.
- [ ] Add permission and storage inspection.
- [ ] Add safe redaction for exported diagnostics.

### 8.3 Crash and support diagnostics

- [ ] Add opt-in crash reporting.
- [ ] Add local crash dump collection.
- [ ] Add privacy review for all diagnostic fields.
- [ ] Add a user-facing diagnostic bundle generator.
- [ ] Add a support workflow for reproducible reports.

**Acceptance criteria:** Diagnostic tools help developers reproduce resource, privacy, and navigation issues without collecting unnecessary browsing data.

---

## Milestone 9 — Sync and credential security

### 9.1 Define the synchronization model

- [ ] Decide which data types can sync.
- [ ] Define conflict-resolution rules.
- [ ] Define device registration and revocation.
- [ ] Define offline behavior.
- [ ] Define server-side metadata minimization.
- [ ] Record the sync architecture decision.

### 9.2 Implement end-to-end encrypted sync

- [ ] Choose audited cryptographic primitives and libraries.
- [ ] Define key generation, storage, recovery, and rotation.
- [ ] Encrypt data before it leaves the device.
- [ ] Authenticate devices and sync records.
- [ ] Protect against replay and rollback.
- [ ] Add recovery and device-revocation flows.
- [ ] Add interoperability and migration tests.

**Acceptance criteria:** The sync service cannot read synchronized secrets, compromised devices can be revoked, and conflict behavior is deterministic.

### 9.3 Implement password and passkey storage

- [ ] Use the operating-system credential store.
- [ ] Define profile and device key boundaries.
- [ ] Add explicit user confirmation before autofill or save.
- [ ] Add passkey/WebAuthn integration through the selected engine.
- [ ] Add export and deletion safeguards.
- [ ] Add security review and threat-model updates.

**Acceptance criteria:** Credentials are never stored in plaintext application files and all sensitive operations are visible to the user.

---

## Milestone 10 — Optional AI gateway

### 10.1 Define AI privacy and product boundaries

- [ ] Keep AI disabled and non-resident by default.
- [ ] Define exactly what page data may leave the device.
- [ ] Add provider-specific data-use disclosures.
- [ ] Add per-site and per-action consent controls.
- [ ] Define local-model resource limits.
- [ ] Add prompt-injection and untrusted-page-content warnings.

### 10.2 Implement the AI gateway

- [ ] Define a provider-neutral request interface.
- [ ] Implement cloud-provider adapters.
- [ ] Implement local-provider adapters for supported runtimes.
- [ ] Add timeout, cancellation, retry, and rate-limit behavior.
- [ ] Add model availability and capability reporting.
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

- [ ] Define exact hardware tiers.
- [ ] Define supported operating systems and versions.
- [ ] Define browser versions and launch flags.
- [ ] Define test pages and scripted workloads.
- [ ] Define cold-cache and warm-cache procedures.
- [ ] Define RSS, PSS, private memory, CPU, startup, restore, and battery metrics.
- [ ] Define warm-up, repetition, and statistical reporting rules.
- [ ] Store machine-readable benchmark results.

### 11.2 Build the benchmark harness

- [ ] Automate browser launch and shutdown.
- [ ] Automate tab creation and workload execution.
- [ ] Automate state transitions.
- [ ] Collect process and browser metrics.
- [ ] Collect startup and restore timings.
- [ ] Generate Markdown and machine-readable reports.
- [ ] Detect regressions against a baseline.

### 11.3 Establish performance budgets

- [ ] Set startup-time targets.
- [ ] Set idle-memory targets.
- [ ] Set per-tab memory targets by workload.
- [ ] Set restore-time targets.
- [ ] Set background CPU targets.
- [ ] Set privacy-filter latency targets.
- [ ] Set acceptable overhead for diagnostics and AI-disabled mode.

**Acceptance criteria:** Every performance claim in the README links to reproducible results and no release exceeds the declared budgets without an explicit decision record.

---

## Milestone 12 — Packaging, updates, and release security

### 12.1 Build platform packages

- [ ] Define supported Linux distributions.
- [ ] Define supported Windows versions.
- [ ] Add platform build scripts.
- [ ] Add application icons and metadata.
- [ ] Add file associations and URL handlers.
- [ ] Add clean-install and uninstall tests.
- [ ] Add upgrade and downgrade behavior tests.

### 12.2 Implement secure updates

- [ ] Define signed update manifests.
- [ ] Use separate package and update-signing keys.
- [ ] Verify signatures before installation.
- [ ] Prevent unauthorized downgrade and rollback.
- [ ] Define key rotation and emergency revocation.
- [ ] Add staged rollout support.
- [ ] Add update failure recovery.
- [ ] Add offline update verification.

**Acceptance criteria:** An attacker who controls the download URL cannot install an unsigned or modified package.

### 12.3 Establish release provenance

- [ ] Add reproducible-build documentation.
- [ ] Generate SBOMs.
- [ ] Publish checksums and signatures.
- [ ] Record source commit and toolchain versions.
- [ ] Run release builds in isolated CI environments.
- [ ] Publish release notes and known limitations.
- [ ] Maintain a security advisory process.

**Acceptance criteria:** Users can verify what source and toolchain produced each release and can validate downloaded artifacts independently.

---

## Milestone 13 — Beta readiness

### 13.1 Reliability testing

- [ ] Add long-running soak tests.
- [ ] Add repeated suspend/restore tests.
- [ ] Add renderer-crash recovery tests.
- [ ] Add interrupted-session-write tests.
- [ ] Add low-disk-space tests.
- [ ] Add low-memory tests.
- [ ] Add network-loss and captive-portal tests.
- [ ] Add concurrent profile and multi-window tests.

### 13.2 Compatibility testing

- [ ] Test common websites.
- [ ] Test WebAuthn and passkeys.
- [ ] Test video and audio playback.
- [ ] Test file uploads and downloads.
- [ ] Test notifications and permissions.
- [ ] Test extensions from the documented compatibility set.
- [ ] Test accessibility features.
- [ ] Test keyboard-only navigation.
- [ ] Test high-DPI and multi-monitor setups.

### 13.3 Privacy and security review

- [ ] Conduct an independent review of profile storage.
- [ ] Review network requests made at startup.
- [ ] Review telemetry and crash-report defaults.
- [ ] Review update verification.
- [ ] Review extension isolation.
- [ ] Review sync and credential handling.
- [ ] Review AI data flows.
- [ ] Resolve high-severity findings before beta release.

**Acceptance criteria:** The beta has a documented support matrix, known limitations, reproducible diagnostics, and no unresolved high-severity security issues.

---

## Milestone 14 — Public release and ongoing maintenance

### 14.1 Release the first stable scope

- [ ] Freeze the stable feature set.
- [ ] Remove or clearly label experimental features.
- [ ] Publish installation instructions.
- [ ] Publish privacy policy and security model.
- [ ] Publish benchmark results.
- [ ] Publish support and reporting procedures.
- [ ] Tag and sign the release.
- [ ] Verify installation from a clean machine.

### 14.2 Operate the project

- [ ] Monitor crash reports and security reports.
- [ ] Track memory and startup regressions.
- [ ] Update Chromium according to the defined cadence.
- [ ] Update filter lists and review licenses.
- [ ] Patch dependencies promptly.
- [ ] Maintain release branches where necessary.
- [ ] Publish monthly project-status updates.
- [ ] Review roadmap priorities using user feedback and benchmark evidence.

### 14.3 Plan future capabilities

- [ ] Add tab groups and vertical tabs after the core lifecycle is stable.
- [ ] Add picture-in-picture and PWA support.
- [ ] Add reader mode.
- [ ] Add optional VPN integration only after the trust and operational model is defined.
- [ ] Expand developer tooling.
- [ ] Expand local AI support only when resource budgets remain measurable.

**Acceptance criteria:** New features do not weaken the core guarantees: predictable resource behavior, privacy by default, explicit user control, and verifiable releases.

---

## Immediate next actions

These are the recommended next five implementation tasks:

1. [ ] Install the pinned Rust toolchain and run the existing tab-manager tests.
2. [ ] Add `TabObservation`, score calculation, and deterministic scoring tests.
3. [ ] Add the hysteresis scheduler with a fake clock and policy tests.
4. [ ] Add CI for formatting, Clippy, workspace tests, and Markdown validation.
5. [ ] Decide and document the Chromium integration strategy before building the browser shell.

## References

[1]: README.md "FeatherSurf project overview"
[2]: docs/architecture.md "FeatherSurf architecture"
[3]: docs/roadmap.md "FeatherSurf roadmap"
[4]: docs/memory-management-policy.md "FeatherSurf memory-management scoring policy"
[5]: CONTRIBUTING.md "FeatherSurf contribution guidelines"
