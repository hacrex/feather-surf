// FeatherSurf Process Monitor
//
// Cross-platform process monitoring for browser tabs.
// Tracks per-process memory usage, CPU, and lifecycle.

use std::collections::HashMap;
use std::time::SystemTime;

// ── Types ──────────────────────────────────────────────────────────

/// Process identifier (OS-specific).
pub type ProcessId = u32;

/// A tracked process (e.g., a tab's renderer).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TrackedProcess {
    pub pid: ProcessId,
    pub tab_id: u64,
    pub start_time: u64,
    pub last_sample_time: u64,

    // Memory (bytes)
    pub resident_memory: u64,
    pub private_memory: u64,
    pub shared_memory: u64,

    // CPU
    pub cpu_usage: f32, // 0.0 - 1.0

    // Process state
    pub state: ProcessState,
}

/// State of a tracked process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ProcessState {
    Running,
    Suspended,
    Zombie,
    Unknown,
}

/// Memory statistics for the entire system.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SystemMemory {
    pub total_ram: u64,
    pub available_ram: u64,
    pub used_ram: u64,
    pub swap_total: u64,
    pub swap_used: u64,
    pub pressure_level: MemoryPressure,
}

/// Memory pressure level.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MemoryPressure {
    #[default]
    None,
    Moderate,
    Critical,
}

/// A sample of process metrics at a point in time.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProcessSample {
    pub pid: ProcessId,
    pub timestamp: u64,
    pub resident_memory: u64,
    pub private_memory: u64,
    pub cpu_usage: f32,
}

// ── Process Monitor ────────────────────────────────────────────────

/// Monitors browser processes and collects metrics.
pub struct ProcessMonitor {
    tracked: HashMap<ProcessId, TrackedProcess>,
    samples: Vec<ProcessSample>,
    max_samples_per_process: usize,
    system_memory: SystemMemory,
}

impl Default for ProcessMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessMonitor {
    pub fn new() -> Self {
        Self {
            tracked: HashMap::new(),
            samples: Vec::new(),
            max_samples_per_process: 100,
            system_memory: SystemMemory::default(),
        }
    }

    /// Create a monitor with custom sample retention.
    pub fn with_sample_limit(max_samples: usize) -> Self {
        Self {
            max_samples_per_process: max_samples,
            ..Self::new()
        }
    }

    /// Register a process for tracking.
    pub fn track_process(&mut self, pid: ProcessId, tab_id: u64) {
        let now = now_secs();
        self.tracked.insert(
            pid,
            TrackedProcess {
                pid,
                tab_id,
                start_time: now,
                last_sample_time: now,
                resident_memory: 0,
                private_memory: 0,
                shared_memory: 0,
                cpu_usage: 0.0,
                state: ProcessState::Running,
            },
        );
    }

    /// Untrack a process.
    pub fn untrack_process(&mut self, pid: ProcessId) -> Option<TrackedProcess> {
        self.tracked.remove(&pid)
    }

    /// Record a sample for a tracked process.
    pub fn record_sample(&mut self, sample: ProcessSample) {
        if let Some(proc) = self.tracked.get_mut(&sample.pid) {
            proc.resident_memory = sample.resident_memory;
            proc.private_memory = sample.private_memory;
            proc.shared_memory = sample.resident_memory.saturating_sub(sample.private_memory);
            proc.cpu_usage = sample.cpu_usage;
            proc.last_sample_time = sample.timestamp;
        }

        // Add sample and trim old ones
        self.samples.push(sample);
        let pid = self.samples.last().unwrap().pid;
        let count = self.samples.iter().filter(|s| s.pid == pid).count();
        if count > self.max_samples_per_process {
            // Remove oldest sample for this PID
            if let Some(pos) = self.samples.iter().position(|s| s.pid == pid) {
                self.samples.remove(pos);
            }
        }
    }

    /// Get all tracked processes.
    pub fn tracked_processes(&self) -> Vec<&TrackedProcess> {
        self.tracked.values().collect()
    }

    /// Get a tracked process by PID.
    pub fn get_process(&self, pid: ProcessId) -> Option<&TrackedProcess> {
        self.tracked.get(&pid)
    }

    /// Get processes for a specific tab.
    pub fn processes_for_tab(&self, tab_id: u64) -> Vec<&TrackedProcess> {
        self.tracked
            .values()
            .filter(|p| p.tab_id == tab_id)
            .collect()
    }

    /// Get total resident memory across all tracked processes.
    pub fn total_resident_memory(&self) -> u64 {
        self.tracked.values().map(|p| p.resident_memory).sum()
    }

    /// Get total private memory across all tracked processes.
    pub fn total_private_memory(&self) -> u64 {
        self.tracked.values().map(|p| p.private_memory).sum()
    }

    /// Get the process with highest memory usage.
    pub fn largest_process(&self) -> Option<&TrackedProcess> {
        self.tracked.values().max_by_key(|p| p.resident_memory)
    }

    /// Get the process with highest CPU usage.
    pub fn most_cpu_intensive(&self) -> Option<&TrackedProcess> {
        self.tracked.values().max_by(|a, b| {
            a.cpu_usage
                .partial_cmp(&b.cpu_usage)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Update system memory statistics.
    pub fn update_system_memory(&mut self, memory: SystemMemory) {
        self.system_memory = memory;
    }

    /// Get current system memory stats.
    pub fn system_memory(&self) -> &SystemMemory {
        &self.system_memory
    }

    /// Get memory pressure level.
    pub fn pressure_level(&self) -> MemoryPressure {
        self.system_memory.pressure_level
    }

    /// Get recent samples for a process.
    pub fn recent_samples(&self, pid: ProcessId, n: usize) -> Vec<&ProcessSample> {
        self.samples
            .iter()
            .filter(|s| s.pid == pid)
            .rev()
            .take(n)
            .collect()
    }

    /// Get average memory usage for a process over recent samples.
    pub fn average_memory(&self, pid: ProcessId, samples: usize) -> Option<u64> {
        let recent: Vec<u64> = self
            .recent_samples(pid, samples)
            .iter()
            .map(|s| s.resident_memory)
            .collect();
        if recent.is_empty() {
            None
        } else {
            Some(recent.iter().sum::<u64>() / recent.len() as u64)
        }
    }

    /// Detect processes that are likely zombies (no recent samples).
    pub fn detect_zombies(&self, max_age_secs: u64) -> Vec<ProcessId> {
        let now = now_secs();
        self.tracked
            .values()
            .filter(|p| {
                p.state == ProcessState::Zombie
                    || now.saturating_sub(p.last_sample_time) > max_age_secs
            })
            .map(|p| p.pid)
            .collect()
    }

    /// Get process count.
    pub fn process_count(&self) -> usize {
        self.tracked.len()
    }

    /// Clear all tracked processes and samples.
    pub fn clear(&mut self) {
        self.tracked.clear();
        self.samples.clear();
    }
}

// ── Helpers ────────────────────────────────────────────────────────

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(pid: ProcessId, mem: u64, cpu: f32) -> ProcessSample {
        ProcessSample {
            pid,
            timestamp: now_secs(),
            resident_memory: mem,
            private_memory: mem,
            cpu_usage: cpu,
        }
    }

    #[test]
    fn track_and_untrack() {
        let mut monitor = ProcessMonitor::new();
        monitor.track_process(100, 1);
        assert_eq!(monitor.process_count(), 1);
        monitor.untrack_process(100);
        assert_eq!(monitor.process_count(), 0);
    }

    #[test]
    fn record_sample_updates_process() {
        let mut monitor = ProcessMonitor::new();
        monitor.track_process(100, 1);
        monitor.record_sample(sample(100, 1024 * 1024, 0.5));
        let proc = monitor.get_process(100).unwrap();
        assert_eq!(proc.resident_memory, 1024 * 1024);
        assert_eq!(proc.cpu_usage, 0.5);
    }

    #[test]
    fn total_memory_across_processes() {
        let mut monitor = ProcessMonitor::new();
        monitor.track_process(100, 1);
        monitor.track_process(200, 2);
        monitor.record_sample(sample(100, 100, 0.1));
        monitor.record_sample(sample(200, 200, 0.2));
        assert_eq!(monitor.total_resident_memory(), 300);
    }

    #[test]
    fn largest_process() {
        let mut monitor = ProcessMonitor::new();
        monitor.track_process(100, 1);
        monitor.track_process(200, 2);
        monitor.record_sample(sample(100, 100, 0.1));
        monitor.record_sample(sample(200, 500, 0.2));
        assert_eq!(monitor.largest_process().unwrap().pid, 200);
    }

    #[test]
    fn most_cpu_intensive() {
        let mut monitor = ProcessMonitor::new();
        monitor.track_process(100, 1);
        monitor.track_process(200, 2);
        monitor.record_sample(sample(100, 100, 0.3));
        monitor.record_sample(sample(200, 500, 0.9));
        assert_eq!(monitor.most_cpu_intensive().unwrap().pid, 200);
    }

    #[test]
    fn processes_for_tab() {
        let mut monitor = ProcessMonitor::new();
        monitor.track_process(100, 1);
        monitor.track_process(200, 1);
        monitor.track_process(300, 2);
        let tab1 = monitor.processes_for_tab(1);
        assert_eq!(tab1.len(), 2);
        let tab2 = monitor.processes_for_tab(2);
        assert_eq!(tab2.len(), 1);
    }

    #[test]
    fn average_memory() {
        let mut monitor = ProcessMonitor::new();
        monitor.track_process(100, 1);
        monitor.record_sample(sample(100, 100, 0.1));
        monitor.record_sample(sample(100, 200, 0.1));
        monitor.record_sample(sample(100, 300, 0.1));
        assert_eq!(monitor.average_memory(100, 3), Some(200));
    }

    #[test]
    fn sample_retention_limit() {
        let mut monitor = ProcessMonitor::with_sample_limit(5);
        monitor.track_process(100, 1);
        for i in 0..10 {
            monitor.record_sample(sample(100, i * 100, 0.1));
        }
        let samples = monitor.recent_samples(100, 100);
        assert!(samples.len() <= 5);
    }

    #[test]
    fn system_memory() {
        let mut monitor = ProcessMonitor::new();
        monitor.update_system_memory(SystemMemory {
            total_ram: 8_000_000_000,
            available_ram: 4_000_000_000,
            used_ram: 4_000_000_000,
            swap_total: 2_000_000_000,
            swap_used: 500_000_000,
            pressure_level: MemoryPressure::None,
        });
        assert_eq!(monitor.system_memory().total_ram, 8_000_000_000);
        assert_eq!(monitor.pressure_level(), MemoryPressure::None);
    }

    #[test]
    fn detect_zombies() {
        let mut monitor = ProcessMonitor::new();
        monitor.track_process(100, 1);
        monitor.track_process(200, 2);
        // Make process 200 look old by setting a very old last_sample_time
        if let Some(p) = monitor.tracked.get_mut(&200) {
            p.last_sample_time = 0;
        }
        let zombies = monitor.detect_zombies(1000);
        assert!(zombies.contains(&200));
        assert!(!zombies.contains(&100));
    }

    #[test]
    fn clear_all() {
        let mut monitor = ProcessMonitor::new();
        monitor.track_process(100, 1);
        monitor.record_sample(sample(100, 100, 0.1));
        monitor.clear();
        assert_eq!(monitor.process_count(), 0);
    }
}
