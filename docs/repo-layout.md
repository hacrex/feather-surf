# Repository Layout

```
featherSurf/
│
├── browser/
│   ├── ui/            # Shell UI: address bar, tab strip, menus
│   ├── windows/        # Window management, multi-window state
│   ├── tabs/           # Tab lifecycle, grouping, vertical tabs
│   └── profiles/       # User profiles, containers
│
├── engine/
│   └── chromium/       # Chromium/Blink integration layer
│
├── core/
│   ├── tab-manager/     # Tab state machine (active → discardable)
│   ├── process-manager/ # Process lifecycle and isolation
│   ├── memory-manager/  # RAM budget enforcement, scoring
│   ├── cache-manager/   # Cache sizing per memory mode
│   └── session-manager/ # State preservation for suspended tabs
│
├── privacy/
│   ├── blocker/         # Tracker/ad blocking
│   ├── fingerprint/     # Fingerprint protection
│   ├── cookie-policy/   # Third-party cookie restriction
│   └── permissions/     # Site permission management
│
├── extensions/          # Chromium extension API compatibility layer
│
├── downloads/           # Download manager
│
├── sync/                # End-to-end encrypted sync
│
├── ai/
│   ├── gateway/         # Routes requests to cloud or local providers
│   ├── local/           # Ollama / llama.cpp integration
│   └── providers/       # Cloud AI provider adapters
│
├── devtools/            # Developer Center: resource inspector, profiler
│
├── rust/                # Rust-native resource-management components
│   ├── memory-manager/
│   ├── tab-manager/
│   ├── process-manager/
│   ├── resource-monitor/
│   ├── privacy-engine/
│   ├── cache-manager/
│   ├── download-manager/
│   └── telemetry/
│
├── updater/             # Auto-update mechanism
│
├── installer/           # Platform installers
│
├── packaging/
│   ├── windows/
│   └── linux/
│
├── docs/                # Project documentation (this folder)
│
├── tests/               # Test suites
│
├── benchmarks/           # Efficiency Benchmark suite + results
│
└── .github/
    └── workflows/        # CI/CD pipelines
```

Each top-level directory ships with its own `README.md` stub describing its
scope and current status — fill these in as implementation begins.
