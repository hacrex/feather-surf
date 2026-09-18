# Memory-Management Scoring Policy

This document defines the first deterministic policy for the Resource Intelligence
Layer. The policy is intentionally independent of Chromium so it can be tested
against synthetic observations before it is connected to renderer/process metrics.

## 1. Observation window and normalization

The manager evaluates every eligible tab once per policy interval. The default
interval is **5 seconds**. Each input is normalized to the closed interval
`[0, 1]`, where `1` means “more reason to keep the tab resident” unless noted
otherwise.

For tab `i` at sample `t`, define:

| Symbol | Meaning | Normalization |
|---|---|---|
| `Aᵢ` | recent user activity | `exp(-age_since_interaction / 300 s)` |
| `Mᵢ` | media activity | `1` while audible audio/video is active, otherwise `0` |
| `Uᵢ` | unsaved user interaction | `1` when a dirty form or active edit exists, otherwise `0` |
| `Nᵢ` | network activity | `min(bytes_per_second / 1 MiB/s, 1)` over the last interval |
| `Rᵢ` | memory pressure | `min(tab_working_set / target_tab_memory, 1)` |
| `Cᵢ` | CPU activity | `min(cpu_percent / 100, 1)` using the tab’s process group |
| `Pᵢ` | pinned status | `1` when pinned, otherwise `0` |
| `Eᵢ` | foreground status | `1` when the selected tab, otherwise `0` |

`Rᵢ` and `Cᵢ` are resource-cost signals. They increase reclaim pressure because
a resource-heavy tab is a stronger candidate for action **only when it is
already inactive**. This is handled by multiplying them by the inactivity
factor below, rather than allowing an active tab to be reclaimed merely
because it is expensive.

## 2. Formal score

Let the inactivity factor be:

```text
Iᵢ = 1 - max(Eᵢ, Aᵢ, Mᵢ, Uᵢ, Pᵢ)
```

The keep-resident score is:

```text
Kᵢ = clamp(
      0.30Eᵢ
    + 0.20Aᵢ
    + 0.15Mᵢ
    + 0.10Uᵢ
    + 0.05Nᵢ
    + 0.10Pᵢ
    - Iᵢ × (0.06Rᵢ + 0.04Cᵢ),
    0,
    1
)
```

where `clamp(x, 0, 1) = min(max(x, 0), 1)`.

This is a **resident score**, not a direct command. Protection signals are
also enforced as hard guards by the tab state machine: pinned tabs, tabs with
active media, and tabs with dirty forms must not enter a reclaimable state even
if a future scoring implementation supplies bad or stale telemetry.

The complementary reclaim pressure is:

```text
Qᵢ = 1 - Kᵢ
```

A global budget multiplier increases reclaim pressure when the browser exceeds
its configured memory budget:

```text
B = clamp((browser_working_set - memory_budget) / memory_budget, 0, 1)
Q'ᵢ = clamp(Qᵢ + 0.25B × Iᵢ, 0, 1)
```

If the budget is unlimited, use `B = 0`. The multiplier affects only inactive
tabs; foreground or protected tabs remain protected by policy.

## 3. State bands

The manager uses different thresholds for entering and leaving each state.
Thresholds are applied to `Q'ᵢ` and are evaluated only after the minimum dwell
time has elapsed.

| Transition | Enter when reclaim pressure remains at least | Exit when pressure falls below | Minimum dwell |
|---|---:|---:|---:|
| `Active → RecentlyActive` | never automatic; focus change only | — | — |
| `RecentlyActive → Background` | `0.20` | `0.12` | 10 s |
| `Background → Frozen` | `0.45` | `0.32` | 30 s |
| `Frozen → Suspended` | `0.68` | `0.52` | 60 s |
| `Background → Discardable` | `0.86` and budget pressure `B ≥ 0.25` | `0.70` | 120 s |

A transition is allowed only if the condition is true for **three consecutive
samples**. At the default five-second interval this is a 15-second debounce.
The candidate counter resets to zero when the condition fails.

The `Frozen → Suspended` and `Background → Discardable` transitions are
mutually exclusive actions. The manager must prefer suspension first when the
tab has a valid snapshot; discard is an emergency fallback when the snapshot is
not available or when budget pressure is severe.

## 4. Hysteresis and anti-thrashing rules

Hysteresis prevents a tab from oscillating between resident and reclaimed
states near a threshold.

1. **Separate enter/exit thresholds.** A tab that entered `Frozen` at `Q' ≥
   0.45` is not restored because `Q'` merely fell to `0.44`; it must fall below
   `0.32` or become explicitly foregrounded.
2. **Consecutive-sample debounce.** A condition must hold for three samples
   before a transition is committed.
3. **Minimum dwell time.** After a successful transition, the tab remains in
   that state for the minimum dwell period in the table, except for explicit
   user focus, navigation, media start, or a dirty-form event.
4. **Restoration cooldown.** After a tab is restored to `Active`, it cannot be
   automatically reclaimed for 30 seconds. User focus always restores
   immediately.
5. **Budget-pressure override.** If `B ≥ 0.50` for two consecutive samples,
   the manager may bypass the normal dwell time for inactive, unprotected tabs,
   but it still preserves the three-sample debounce and never reclaims a
   protected tab.
6. **Priority ordering.** When several tabs are candidates, reclaim in this
   order: discardable candidates with the lowest `K`, then suspended/frozen
   candidates with the lowest `K`, while respecting pinned, media, dirty-form,
   and active-tab guards.
7. **Manual override.** “Keep awake” and “Suspend now” user actions override
   automatic scoring until the user clears the override or the tab is closed.

## 5. Memory modes

Modes adjust thresholds, not the underlying score, so benchmark results remain
comparable:

| Mode | Frozen enter | Suspended enter | Discard enter | Dwell multiplier |
|---|---:|---:|---:|---:|
| Lite | `0.35` | `0.58` | `0.78` | `0.75` |
| Balanced | `0.45` | `0.68` | `0.86` | `1.00` |
| Performance | `0.58` | `0.78` | `0.93` | `1.50` |

Exit thresholds remain 13 percentage points below the corresponding entry
threshold unless an explicit user action requests restoration.

## 6. Required test vectors

The implementation should include deterministic tests for:

- Active tabs never becoming reclaimable through scoring alone.
- Pinned, media-playing, and dirty-form tabs being hard-protected.
- A tab hovering around a threshold not oscillating because of hysteresis.
- Three consecutive qualifying samples being required.
- Minimum dwell times being respected.
- A tab restoring immediately when focused.
- Budget pressure increasing reclaim priority only for inactive tabs.
- Lite, Balanced, and Performance modes producing different transition timing
  while using the same score.
