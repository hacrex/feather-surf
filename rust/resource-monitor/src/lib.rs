// FeatherSurf Resource Monitor
//
// System-level resource monitoring that feeds into the eviction policy.
// Tracks overall memory usage, per-profile memory, and memory pressure.

use std::collections::HashMap;
use std::time::{Duration, SystemTime};

// ── Types ──────────────────────────────────────────────────────────

/// System memory snapshot.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemorySnapshot {
    pub timestamp: u64,
    pub total_ram: u64,
    pub available_ram: u64,
    pub used_ram: u64,
    pub swap_total: u64,
    pub swap_used: u64,
    pub cached: u64,
    pub buffers: u64,
}

/// Memory pressure level derived from system state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub enum PressureLevel {
    /// >50% RAM free.
    None,
    /// 30-50% RAM free.
    Low,
    /// 15-30% RAM free.
    Moderate,
    /// <15% RAM free or swap heavily used.
    High,
    /// <5% RAM free and swap full.
    Critical,
}

impl PressureLevel {
    pub fn from_memory(total: u64, available: u64, swap_total: u64, swap_used: u64) -> Self {
        if total == 0 {
            return PressureLevel::None;
        }
        let free_pct = (available as f64 / total as f64) * 100.0;
        let swap_pct = if swap_total > 0 {
            (swap_used as f64 / swap_total as f64) * 100.0
        } else {
            0.0
        };

        if free_pct < 5.0 || (swap_total > 0 && swap_pct > 90.0) {
            PressureLevel::Critical
        } else if free_pct < 15.0 || (swap_total > 0 && swap_pct > 70.0) {
            PressureLevel::High
        } else if free_pct < 30.0 {
            PressureLevel::Moderate
        } else if free_pct < 50.0 {
            PressureLevel::Low
        } else {
            PressureLevel::None
        }
    }
}

/// Per-profile memory breakdown.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProfileMemory {
    pub profile_id: u64,
    pub resident_memory: u64,
    pub tab_count: usize,
    pub frozen_count: usize,
    pub suspended_count: usize,
}

/// Browser-level resource summary.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResourceSummary {
    pub timestamp: u64,
    pub system: MemorySnapshot,
    pub pressure: PressureLevel,
    pub browser_total_memory: u64,
    pub tab_count: usize,
    pub frozen_count: usize,
    pub suspended_count: usize,
    pub profiles: Vec<ProfileMemory>,
}

/// Resource event for monitoring.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ResourceEvent {
    /// Memory pressure changed level.
    PressureChanged {
        from: PressureLevel,
        to: PressureLevel,
    },
    /// Browser memory exceeded threshold.
    MemoryThresholdExceeded {
        used: u64,
        threshold: u64,
    },
    /// System low memory warning.
    SystemLowMemory,
    /// Eviction recommended.
    EvictionRecommended {
        target_reclaim: u64,
        pressure: PressureLevel,
    },
}

// ── Resource Monitor ───────────────────────────────────────────────

/// Monitors system and browser resource usage.
pub struct ResourceMonitor {
    snapshots: Vec<MemorySnapshot>,
    max_snapshots: usize,
    pressure_thresholds: PressureThresholds,
    last_pressure: PressureLevel,
    events: Vec<ResourceEvent>,
    profile_memory: HashMap<u64, ProfileMemory>,
}

/// Thresholds for memory pressure levels.
#[derive(Debug, Clone)]
pub struct PressureThresholds {
    pub low_pct: f64,
    pub moderate_pct: f64,
    pub high_pct: f64,
    pub critical_pct: f64,
    pub swap_warning_pct: f64,
}

impl Default for PressureThresholds {
    fn default() -> Self {
        Self {
            low_pct: 50.0,
            moderate_pct: 30.0,
            high_pct: 15.0,
            critical_pct: 5.0,
            swap_warning_pct: 70.0,
        }
    }
}

impl Default for ResourceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceMonitor {
    pub fn new() -> Self {
        Self {
            snapshots: Vec::new(),
            max_snapshots: 360, // 1 hour at 1 per 10 seconds
            pressure_thresholds: PressureThresholds::default(),
            last_pressure: PressureLevel::None,
            events: Vec::new(),
            profile_memory: HashMap::new(),
        }
    }

    /// Create with custom snapshot retention.
    pub fn with_retention(max_snapshots: usize) -> Self {
        Self {
            max_snapshots,
            ..Self::new()
        }
    }

    /// Record a memory snapshot.
    pub fn record_snapshot(&mut self, snapshot: MemorySnapshot) {
        let new_pressure = PressureLevel::from_memory(
            snapshot.total_ram,
            snapshot.available_ram,
            snapshot.swap_total,
            snapshot.swap_used,
        );

        if new_pressure != self.last_pressure {
            self.events.push(ResourceEvent::PressureChanged {
                from: self.last_pressure,
                to: new_pressure,
            });
            self.last_pressure = new_pressure;
        }

        self.snapshots.push(snapshot);
        if self.snapshots.len() > self.max_snapshots {
            self.snapshots.remove(0);
        }
    }

    /// Get current pressure level.
    pub fn pressure_level(&self) -> PressureLevel {
        self.last_pressure
    }

    /// Get the latest snapshot.
    pub fn latest_snapshot(&self) -> Option<&MemorySnapshot> {
        self.snapshots.last()
    }

    /// Get recent snapshots.
    pub fn recent_snapshots(&self, n: usize) -> Vec<&MemorySnapshot> {
        self.snapshots.iter().rev().take(n).collect()
    }

    /// Check if eviction is recommended.
    pub fn should_evict(&self, browser_memory: u64, max_browser_memory: u64) -> bool {
        if self.last_pressure >= PressureLevel::High {
            return true;
        }
        if browser_memory > max_browser_memory {
            return true;
        }
        false
    }

    /// Calculate how much memory to reclaim.
    pub fn recommended_reclaim(
        &self,
        browser_memory: u64,
        max_browser_memory: u64,
    ) -> u64 {
        let target = match self.last_pressure {
            PressureLevel::None => return 0,
            PressureLevel::Low => max_browser_memory * 10 / 100, // 10%
            PressureLevel::Moderate => max_browser_memory * 25 / 100, // 25%
            PressureLevel::High => max_browser_memory * 50 / 100, // 50%
            PressureLevel::Critical => max_browser_memory * 75 / 100, // 75%
        };

        if browser_memory > max_browser_memory {
            let over = browser_memory - max_browser_memory;
            return over + target;
        }

        target
    }

    /// Update per-profile memory.
    pub fn update_profile_memory(
        &mut self,
        profile_id: u64,
        resident_memory: u64,
        tab_count: usize,
        frozen_count: usize,
        suspended_count: usize,
    ) {
        self.profile_memory.insert(
            profile_id,
            ProfileMemory {
                profile_id,
                resident_memory,
                tab_count,
                frozen_count,
                suspended_count,
            },
        );
    }

    /// Get per-profile memory breakdown.
    pub fn profile_memory(&self) -> Vec<&ProfileMemory> {
        self.profile_memory.values().collect()
    }

    /// Get total browser memory across all profiles.
    pub fn total_browser_memory(&self) -> u64 {
        self.profile_memory
            .values()
            .map(|p| p.resident_memory)
            .sum()
    }

    /// Get total tab count.
    pub fn total_tab_count(&self) -> usize {
        self.profile_memory.values().map(|p| p.tab_count).sum()
    }

    /// Consume pending events.
    pub fn drain_events(&mut self) -> Vec<ResourceEvent> {
        std::mem::take(&mut self.events)
    }

    /// Get memory trend (positive = increasing, negative = decreasing).
    pub fn memory_trend(&self, window: usize) -> f64 {
        let recent = self.recent_snapshots(window);
        if recent.len() < 2 {
            return 0.0;
        }
        let first = recent.last().unwrap().used_ram as f64;
        let last = recent.first().unwrap().used_ram as f64;
        last - first
    }

    /// Estimate time until critical pressure based on trend.
    pub fn estimated_time_to_critical(&self) -> Option<Duration> {
        let trend = self.memory_trend(30);
        if trend <= 0.0 {
            return None; // Stable or decreasing
        }

        let latest = self.latest_snapshot()?;
        let total = latest.total_ram as f64;
        let available = latest.available_ram as f64;
        let critical_threshold = total * 0.05;
        let remaining = available - critical_threshold;

        if remaining <= 0.0 {
            return Some(Duration::ZERO);
        }

        let seconds = remaining / trend;
        Some(Duration::from_secs_f64(seconds))
    }

    /// Clear all data.
    pub fn clear(&mut self) {
        self.snapshots.clear();
        self.events.clear();
        self.profile_memory.clear();
        self.last_pressure = PressureLevel::None;
    }
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot(total: u64, available: u64) -> MemorySnapshot {
        MemorySnapshot {
            timestamp: now_secs(),
            total_ram: total,
            available_ram: available,
            used_ram: total - available,
            swap_total: 0,
            swap_used: 0,
            cached: 0,
            buffers: 0,
        }
    }

    #[test]
    fn pressure_levels() {
        assert_eq!(
            PressureLevel::from_memory(100, 60, 0, 0),
            PressureLevel::None
        );
        assert_eq!(
            PressureLevel::from_memory(100, 40, 0, 0),
            PressureLevel::Low
        );
        assert_eq!(
            PressureLevel::from_memory(100, 20, 0, 0),
            PressureLevel::Moderate
        );
        assert_eq!(
            PressureLevel::from_memory(100, 10, 0, 0),
            PressureLevel::High
        );
        assert_eq!(
            PressureLevel::from_memory(100, 3, 0, 0),
            PressureLevel::Critical
        );
    }

    #[test]
    fn swap_pressure() {
        // 50% free RAM but 80% swap used should be Moderate
        assert_eq!(
            PressureLevel::from_memory(100, 50, 100, 80),
            PressureLevel::Moderate
        );
        // 20% free RAM but 95% swap used should be Critical
        assert_eq!(
            PressureLevel::from_memory(100, 20, 100, 95),
            PressureLevel::Critical
        );
    }

    #[test]
    fn record_and_detect_pressure_change() {
        let mut monitor = ResourceMonitor::new();
        monitor.record_snapshot(snapshot(100, 60));
        assert_eq!(monitor.pressure_level(), PressureLevel::None);

        monitor.record_snapshot(snapshot(100, 10));
        assert_eq!(monitor.pressure_level(), PressureLevel::High);

        let events = monitor.drain_events();
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0],
            ResourceEvent::PressureChanged {
                from: PressureLevel::None,
                to: PressureLevel::High
            }
        ));
    }

    #[test]
    fn should_evict_based_on_pressure() {
        let mut monitor = ResourceMonitor::new();
        monitor.record_snapshot(snapshot(100, 60)); // None
        assert!(!monitor.should_evict(1000, 2000));

        monitor.record_snapshot(snapshot(100, 10)); // High
        assert!(monitor.should_evict(1000, 2000));
    }

    #[test]
    fn recommended_reclaim_scales_with_pressure() {
        let mut monitor = ResourceMonitor::new();
        monitor.record_snapshot(snapshot(100, 40)); // Low
        let reclaim = monitor.recommended_reclaim(1000, 2000);
        assert_eq!(reclaim, 200); // 10%

        monitor.record_snapshot(snapshot(100, 10)); // High
        let reclaim = monitor.recommended_reclaim(1000, 2000);
        assert_eq!(reclaim, 1000); // 50%
    }

    #[test]
    fn profile_memory_tracking() {
        let mut monitor = ResourceMonitor::new();
        monitor.update_profile_memory(1, 1000, 5, 2, 1);
        monitor.update_profile_memory(2, 500, 3, 1, 0);
        assert_eq!(monitor.total_browser_memory(), 1500);
        assert_eq!(monitor.total_tab_count(), 8);
    }

    #[test]
    fn snapshot_retention() {
        let mut monitor = ResourceMonitor::with_retention(5);
        for i in 0..10 {
            monitor.record_snapshot(snapshot(100, 60));
        }
        assert!(monitor.recent_snapshots(100).len() <= 5);
    }

    #[test]
    fn memory_trend() {
        let mut monitor = ResourceMonitor::new();
        monitor.record_snapshot(MemorySnapshot {
            timestamp: 100,
            total_ram: 100,
            available_ram: 50,
            used_ram: 50,
            swap_total: 0,
            swap_used: 0,
            cached: 0,
            buffers: 0,
        });
        monitor.record_snapshot(MemorySnapshot {
            timestamp: 200,
            total_ram: 100,
            available_ram: 30,
            used_ram: 70,
            swap_total: 0,
            swap_used: 0,
            cached: 0,
            buffers: 0,
        });
        let trend = monitor.memory_trend(10);
        assert!(trend > 0.0); // Memory is increasing
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
