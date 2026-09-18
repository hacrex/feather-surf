# FeatherSurf Efficiency Benchmark

"RAM friendly" is a measurable claim, not a marketing statement. This suite
defines how that claim gets proven and tracked over time.

## Test environment

- Reference machine: 8 GB RAM
- Tab counts: 10, 20, 50, 100
- Comparison browsers: at minimum one Chromium-based browser, one
  Firefox-based browser, and one Safari/WebKit-based browser where
  applicable

## Metrics

- Idle RAM
- RAM at 10 / 20 / 50 / 100 tabs
- CPU usage at idle
- CPU usage while browsing
- Startup time
- Tab restore time (after suspension/discard)
- Battery consumption
- Page-load performance

## Reporting format

```
Browser        RAM       CPU       Startup
-----------------------------------------
Browser A      X GB      X %       X sec
Browser B      X GB      X %       X sec
FeatherSurf    X GB      X %       X sec
```

## Status

No results yet — this suite is defined ahead of implementation so that
every milestone in the roadmap is measured against it from Phase 1 onward.
