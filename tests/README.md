# Tests

FeatherSurf keeps component tests beside their Rust crates and places cross-
component workflow tests under the relevant crate's `tests/` directory.

The memory-management integration suite is located at
`rust/memory-manager/tests/eviction_workflow.rs`. It exercises the scoring
equation, budget pressure, debounce, dwell time, hysteresis reset, memory modes,
protected tabs, freeze/suspend/restore behavior, and emergency discard.

Run the complete Rust workspace suite with:

```bash
cd rust
cargo test --workspace --all-targets
```

Run only the memory-management workflow suite with:

```bash
cd rust
cargo test -p feathersurf-memory-manager --test eviction_workflow
```

Renderer, browser-shell, privacy, profile, and platform sandbox integration
tests will be added as those adapters are implemented. Security-specific test
requirements are defined in
[`docs/tab-execution-security.md`](../docs/tab-execution-security.md).
