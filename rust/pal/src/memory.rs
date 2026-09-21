//! Platform-specific memory monitoring abstraction.
//!
//! Each platform implements this trait to report working set sizes,
//! available memory, and process-level memory usage.

use std::collections::HashMap;

/// Memory statistics for a single process.
#[derive(Debug, Clone, PartialEq)]
pub struct ProcessMemory {
    pub pid: u32,
    /// Private working set in bytes.
    pub private_bytes: u64,
    /// Shared working set in bytes.
    pub shared_bytes: u64,
    /// Peak private working set in bytes.
    pub peak_private_bytes: u64,
}

/// System-wide memory statistics.
#[derive(Debug, Clone, PartialEq)]
pub struct SystemMemory {
    /// Total physical RAM in bytes.
    pub total_bytes: u64,
    /// Currently available in bytes.
    pub available_bytes: u64,
    /// Total committed (used) in bytes.
    pub committed_bytes: u64,
}

/// Trait for platform-specific memory monitoring.
///
/// # Platform Implementations
///
/// - **Windows:** Uses `QueryWorkingSetEx`, `GlobalMemoryStatusEx`
/// - **Linux:** Parses `/proc/[pid]/smaps`, `/proc/meminfo`
/// - **macOS:** Uses `mach_task_basic_info`, `host_statistics64`
/// - **Android:** Uses `/proc/[pid]/smaps` (same as Linux)
/// - **iOS:** Uses `mach_task_basic_info` (same as macOS kernel)
pub trait MemoryProvider {
    /// Get system-wide memory statistics.
    fn system_memory(&self) -> SystemMemory;

    /// Get memory usage for a specific process.
    fn process_memory(&self, pid: u32) -> Option<ProcessMemory>;

    /// Get memory usage for multiple processes at once.
    fn batch_process_memory(&self, pids: &[u32]) -> HashMap<u32, ProcessMemory>;

    /// Get the browser's total working set across all renderer processes.
    fn browser_working_set(&self, pids: &[u32]) -> u64;
}
