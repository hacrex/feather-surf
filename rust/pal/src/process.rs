//! Platform-specific process monitoring abstraction.
//!
//! Each platform implements this trait to track renderer processes,
//! their lifecycle, and resource consumption.

use std::time::Duration;

/// Information about a single OS process.
#[derive(Debug, Clone, PartialEq)]
pub struct ProcessInfo {
    pub pid: u32,
    pub parent_pid: u32,
    pub name: String,
    /// RSS (Resident Set Size) in bytes.
    pub rss_bytes: u64,
    /// CPU usage as a percentage (0.0 - 100.0+).
    pub cpu_percent: f64,
    /// Wall-clock time since process start.
    pub uptime: Duration,
}

/// Trait for platform-specific process monitoring.
///
/// # Platform Implementations
///
/// - **Windows:** Uses `OpenProcess`, `QueryFullProcessImageNameW`, job objects
/// - **Linux:** Uses `/proc/[pid]/stat`, `/proc/[pid]/status`
/// - **macOS:** Uses `sysctl`, `proc_pidinfo`
/// - **Android:** Same as Linux (`/proc/[pid]/stat`)
/// - **iOS:** Limited process info via `proc_pidinfo`
pub trait ProcessMonitor {
    /// Find all processes matching a name pattern (e.g., "feathersurf" or "chrome").
    fn find_processes(&self, name_pattern: &str) -> Vec<ProcessInfo>;

    /// Get detailed info for a specific PID.
    fn process_info(&self, pid: u32) -> Option<ProcessInfo>;

    /// Check if a process is still alive.
    fn is_alive(&self, pid: u32) -> bool;

    /// Kill a process (platform-appropriate signal).
    fn kill_process(&self, pid: u32) -> Result<(), ProcessError>;

    /// Suspend a process (pause execution).
    ///
    /// Used for the "Suspended" tab state.
    /// - **Windows:** `SuspendThread` on all threads
    /// - **Linux/macOS:** `SIGSTOP`
    /// - **Android:** Same as Linux
    /// - **iOS:** Same as macOS
    fn suspend_process(&self, pid: u32) -> Result<(), ProcessError>;

    /// Resume a previously suspended process.
    ///
    /// - **Windows:** `ResumeThread`
    /// - **Linux/macOS:** `SIGCONT`
    fn resume_process(&self, pid: u32) -> Result<(), ProcessError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessError {
    NotFound(u32),
    AccessDenied(u32),
    PlatformError(String),
}

impl std::fmt::Display for ProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(pid) => write!(f, "process {pid} not found"),
            Self::AccessDenied(pid) => write!(f, "access denied to process {pid}"),
            Self::PlatformError(msg) => write!(f, "platform error: {msg}"),
        }
    }
}

impl std::error::Error for ProcessError {}
