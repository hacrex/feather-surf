# Changelog

All notable changes to FeatherSurf will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial project scaffold and repository structure
- Architecture Decision Records (ADRs) for Chromium integration strategy
- Tab lifecycle state machine (Active → RecentlyActive → Background → Frozen → Suspended → Discardable)
- Memory scoring model with keep-resident score and reclaim pressure
- Tab serialization with bincode and schema versioning
- Property-based testing for tab state transitions
- User overrides: keep-awake protection, force-suspend action
- Hysteresis anti-loop protection for tab state transitions
- Session persistence with atomic writes and backup rotation
- Session schema migrations (V0 → V1)
- Navigation history (back/forward) per tab
- Profile isolation with Default, Container, Ephemeral modes
- Container-style isolation rules
- Bookmarks storage with folder hierarchy
- History storage with retention and pruning
- Import/export for bookmarks (HTML, JSON)
- Import/export for history (JSON)
- Process monitoring (RSS, private memory, CPU, zombie detection)
- Resource monitoring with pressure levels (None → Critical)
- Renderer adapter commands (Freeze/Unfreeze/Suspend/Discard/Restore)
- RAM budgets (Unlimited, Fixed, Adaptive)
- BudgetManager with warning/critical thresholds
- Developer Center with browser summary, tab details, diagnostics export
- Structured logging with privacy-safe redaction
- Browser preferences with validation
- Windows browser shell with Win32 API
- Linux browser shell with GTK4
- CEF integration with Rust FFI bridge
- Tab strip, navigation controls, address bar
- Keyboard shortcuts (Ctrl+T, Ctrl+W, Ctrl+L, etc.)
- Error pages for failed navigation

### Changed
- Updated todo.md to reflect completed milestones M1-M5

### Fixed
- None

## [0.0.1] - 2026-09-21

### Added
- Initial repository creation
- Project documentation and architecture
- CI pipeline (formatting, clippy, tests, documentation)
