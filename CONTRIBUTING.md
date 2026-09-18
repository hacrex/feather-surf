# Contributing to FeatherSurf

Thanks for considering a contribution to FeatherSurf — an open-source,
privacy-first, RAM-efficient browser.

## Ground rules

- Read [`docs/architecture.md`](docs/architecture.md) first. The Resource
  Intelligence Layer is the heart of the project — most meaningful
  contributions touch it directly or support it.
- **Never solve a problem by simply demanding more RAM.** Every PR that
  adds memory usage should explain why it can't be optimized, reduced, or
  made optional first.
- Prefer optional, gateway-routed integrations (see `ai/`) over anything
  that adds baseline memory when a feature is unused.
- No mandatory telemetry, ever. See `docs/architecture.md` for defaults.

## Where to start

- Check open issues labeled `good first issue`.
- Phase 0/1 work (see `docs/roadmap.md`) is the current priority: Chromium
  integration research and the Browser MVP.
- Rust components live in `rust/`; browser shell/UI work lives in
  `browser/`.

## Development setup

This repository is currently a scaffold. Build instructions will be added
here as Phase 0 research and Phase 1 MVP work lands.

## Pull requests

1. Fork the repo and create a feature branch.
2. Keep PRs scoped to one concern (one manager, one UI surface, one doc).
3. Include benchmark results (see `benchmarks/`) for any change that could
   affect memory or CPU usage.
4. Describe what you tested and on what hardware tier (4 GB / 8 GB / 16 GB).

## Code of Conduct

This project follows the [Code of Conduct](CODE_OF_CONDUCT.md). By
participating, you agree to uphold it.
