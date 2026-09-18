# 🪶 FeatherSurf

**The browser that respects your hardware.**
*More Web. Less RAM.*

FeatherSurf is an open-source, privacy-first, RAM-efficient desktop browser
designed to deliver a modern web experience without demanding high-end
hardware.

---

## Why FeatherSurf

Most modern browsers assume you have RAM to spare. FeatherSurf assumes the
opposite: it treats memory as the scarcest, most valuable resource on the
machine, and makes **RAM efficiency the primary product principle** rather
than an afterthought.

Feature parity with Chrome, Brave, Firefox, and Safari is the target — not
the differentiator. The differentiator is a **Resource Intelligence Layer**
that decides, continuously and transparently, what deserves memory and what
doesn't.

Target hardware tiers:

| RAM   | Use case                          |
|-------|------------------------------------|
| 4 GB  | Basic browsing                     |
| 8 GB  | Comfortable development / student  |
| 16 GB | Advanced workloads                 |

Initial platforms: **Linux + Windows**. macOS is a later target.

---

## Core Philosophy

> **Modern browser capabilities without requiring a modern high-end PC.**

One non-negotiable rule guides every design decision:

**Never solve a software problem by simply demanding more RAM.**

```
First    → Optimize
Second   → Reduce
Third    → Suspend
Fourth   → Restore intelligently
Finally  → Use more resources
```

---

## Architecture at a Glance

FeatherSurf does not attempt to merge four browser engines into one. It
builds around **Chromium/Blink** (V8, modern web standards, the extensions
ecosystem, DevTools, multi-process architecture, and a huge upstream
community) and layers its own shell and resource-management system on top.

```
                 Browser UI
                     │
                     ▼
              Browser Controller
                     │
        ┌────────────┼────────────┐
        ▼            ▼            ▼
   Tab Manager   RAM Manager   Privacy
        │            │            │
        └────────────┼────────────┘
                     ▼
             Chromium Engine
              Blink + V8
```

The resource-management components (`/rust`) are built in **Rust** for
memory safety, low overhead, and predictable long-running concurrency —
without rewriting Chromium itself.

See [`docs/architecture.md`](docs/architecture.md) for the full breakdown
and [`docs/repo-layout.md`](docs/repo-layout.md) for what lives where.

---

## What Makes FeatherSurf Different

FeatherSurf borrows philosophy, not code or branding, from each major browser:

| Inspiration | Taken from |
|---|---|
| Compatibility, DevTools, extensions, PWA, profiles, sync | Chromium / Chrome |
| Privacy-by-default, tracker blocking, fingerprint protection | Brave |
| User control, container-style isolation, customization, open governance | Firefox |
| Energy efficiency, simplicity, minimal background activity | Safari |

Result:

> **Chromium compatibility + Firefox-style control + Brave-style privacy + Safari-style efficiency** — without literally containing any of them.

---

## Signature Features

- **Resource Intelligence Layer** — every tab is scored (activity, media,
  interaction, network, memory, CPU, pinned state) and moved through
  `Active → Recently Active → Background → Frozen → Suspended → Discardable`.
- **Adaptive Memory Modes** — 🪶 Lite, ⚖️ Balanced, 🚀 Performance.
- **RAM Budget** — set a hard ceiling; the browser manages itself to stay
  under it.
- **Graceful suspension** — suspended tabs preserve URL, scroll position,
  form state, and history so restoring feels instant. No "where did my tab
  go?" moments.
- **Developer Center** — a live, per-component memory/CPU/network dashboard
  far more granular than a single "Chrome is using 1.2 GB" number.
- **Privacy opt-out, not opt-in** — trackers and known-malicious domains
  blocked by default, fingerprinting reduced, third-party cookies
  restricted, telemetry off and opt-in only.
- **AI is optional and never resident by default** — routed through an AI
  Gateway to cloud or local (Ollama/llama.cpp) providers. `AI = OFF` means
  zero extra model RAM.

Full feature matrix in [`docs/features.md`](docs/features.md).

---

## Proving It: The Efficiency Benchmark

"RAM friendly" is a measurable claim here, not a marketing line. See
[`benchmarks/README.md`](benchmarks/README.md) for the methodology used to
compare FeatherSurf against other browsers at 10 / 20 / 50 / 100 tabs, on
idle RAM, CPU, startup time, tab-restore time, and battery draw.

---

## Roadmap

| Phase | Focus |
|---|---|
| 0 | Research — Chromium architecture, Blink, V8, CEF, licensing, extension model |
| 1 | Browser MVP — window, tabs, address bar, navigation, bookmarks, history, downloads, private mode |
| 2 | RAM Engine — process monitoring, tab scoring, freeze/suspend/discard, RAM budget, dashboard |
| 3 | Privacy — tracker/ad blocking, cookie controls, fingerprint protection, permissions |
| 4 | Developer Edition — resource inspector, network inspector, process tree, memory profiler |
| 5 | Sync — bookmarks, history, settings, passwords, tabs, end-to-end encrypted |
| 6 | AI — assistant, summarization, local + cloud providers, developer assistant |

Full detail in [`docs/roadmap.md`](docs/roadmap.md).

---

## Repository Layout

```
featherSurf/
├── browser/        # UI, windows, tabs, profiles
├── engine/          # Chromium integration
├── core/           # tab/process/memory/cache/session managers
├── privacy/        # blocker, fingerprint, cookie-policy, permissions
├── extensions/      # Chromium extension compatibility layer
├── downloads/
├── sync/
├── ai/             # gateway, local, providers — all optional
├── devtools/
├── rust/           # Rust-native resource-management components
├── updater/
├── installer/
├── packaging/       # windows/, linux/
├── docs/
├── tests/
├── benchmarks/
└── .github/workflows/
```

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md) and our
[`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md). Issues and discussions are the
best place to start — especially around the Resource Intelligence Layer,
which is the heart of the project.

## License

Licensed under the [Apache License 2.0](LICENSE).
