// FeatherSurf Developer Tools
//
// Resource inspector, network diagnostics, and crash reporting.

use std::collections::HashMap;
use std::time::SystemTime;

// ── Resource Inspector ─────────────────────────────────────────────

/// Process information for the resource inspector.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub tab_id: Option<u64>,
    pub process_type: ProcessType,
    pub memory_bytes: u64,
    pub cpu_percent: f32,
    pub state: String,
    pub uptime_secs: u64,
}

/// Type of browser process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ProcessType {
    Browser,
    Renderer,
    Gpu,
    Utility,
    Extension,
    Plugin,
    Other,
}

/// Tab-to-process mapping.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TabProcessMapping {
    pub tab_id: u64,
    pub url: String,
    pub title: String,
    pub pid: u32,
    pub process_type: ProcessType,
    pub state: String,
    pub memory_bytes: u64,
}

/// Memory chart data point.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryDataPoint {
    pub timestamp: u64,
    pub total_bytes: u64,
    pub browser_bytes: u64,
    pub renderer_bytes: u64,
    pub gpu_bytes: u64,
    pub other_bytes: u64,
}

/// CPU chart data point.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CpuDataPoint {
    pub timestamp: u64,
    pub browser_percent: f32,
    pub renderer_percent: f32,
    pub gpu_percent: f32,
    pub total_percent: f32,
}

/// State transition record.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StateTransition {
    pub tab_id: u64,
    pub from_state: String,
    pub to_state: String,
    pub timestamp: u64,
    pub reason: String,
    pub trigger: TransitionTrigger,
}

/// What triggered a state transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TransitionTrigger {
    User,
    Automatic,
    System,
    Manual,
}

/// Resource inspector.
pub struct ResourceInspector {
    /// Process information.
    processes: Vec<ProcessInfo>,
    /// Tab-to-process mappings.
    tab_mappings: Vec<TabProcessMapping>,
    /// Memory history for charts.
    memory_history: Vec<MemoryDataPoint>,
    /// CPU history for charts.
    cpu_history: Vec<CpuDataPoint>,
    /// State transition history.
    transitions: Vec<StateTransition>,
    /// Maximum history size.
    max_history: usize,
}

impl ResourceInspector {
    pub fn new() -> Self {
        Self {
            processes: Vec::new(),
            tab_mappings: Vec::new(),
            memory_history: Vec::new(),
            cpu_history: Vec::new(),
            transitions: Vec::new(),
            max_history: 1000,
        }
    }

    /// Update process information.
    pub fn update_processes(&mut self, processes: Vec<ProcessInfo>) {
        self.processes = processes;
    }

    /// Update tab-to-process mappings.
    pub fn update_tab_mappings(&mut self, mappings: Vec<TabProcessMapping>) {
        self.tab_mappings = mappings;
    }

    /// Record memory usage.
    pub fn record_memory(&mut self, data_point: MemoryDataPoint) {
        self.memory_history.push(data_point);
        if self.memory_history.len() > self.max_history {
            self.memory_history.remove(0);
        }
    }

    /// Record CPU usage.
    pub fn record_cpu(&mut self, data_point: CpuDataPoint) {
        self.cpu_history.push(data_point);
        if self.cpu_history.len() > self.max_history {
            self.cpu_history.remove(0);
        }
    }

    /// Record a state transition.
    pub fn record_transition(&mut self, transition: StateTransition) {
        self.transitions.push(transition);
        if self.transitions.len() > self.max_history {
            self.transitions.remove(0);
        }
    }

    /// Get all processes.
    pub fn processes(&self) -> &[ProcessInfo] {
        &self.processes
    }

    /// Get tab-to-process mappings.
    pub fn tab_mappings(&self) -> &[TabProcessMapping] {
        &self.tab_mappings
    }

    /// Get memory history.
    pub fn memory_history(&self) -> &[MemoryDataPoint] {
        &self.memory_history
    }

    /// Get CPU history.
    pub fn cpu_history(&self) -> &[CpuDataPoint] {
        &self.cpu_history
    }

    /// Get recent transitions.
    pub fn recent_transitions(&self, n: usize) -> Vec<&StateTransition> {
        self.transitions.iter().rev().take(n).collect()
    }

    /// Get total memory across all processes.
    pub fn total_memory(&self) -> u64 {
        self.processes.iter().map(|p| p.memory_bytes).sum()
    }

    /// Get total CPU across all processes.
    pub fn total_cpu(&self) -> f32 {
        self.processes.iter().map(|p| p.cpu_percent).sum()
    }

    /// Get process count.
    pub fn process_count(&self) -> usize {
        self.processes.len()
    }

    /// Get tab count.
    pub fn tab_count(&self) -> usize {
        self.tab_mappings.len()
    }

    /// Export snapshot for diagnostics.
    pub fn export_snapshot(&self) -> DiagnosticSnapshot {
        DiagnosticSnapshot {
            timestamp: now_secs(),
            processes: self.processes.clone(),
            tab_mappings: self.tab_mappings.clone(),
            total_memory: self.total_memory(),
            total_cpu: self.total_cpu(),
            recent_transitions: self.recent_transitions(100).into_iter().cloned().collect(),
        }
    }
}

impl Default for ResourceInspector {
    fn default() -> Self {
        Self::new()
    }
}

/// Diagnostic snapshot for export.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiagnosticSnapshot {
    pub timestamp: u64,
    pub processes: Vec<ProcessInfo>,
    pub tab_mappings: Vec<TabProcessMapping>,
    pub total_memory: u64,
    pub total_cpu: f32,
    pub recent_transitions: Vec<StateTransition>,
}

// ── Network Diagnostics ────────────────────────────────────────────

/// Request blocking decision for diagnostics.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RequestDecision {
    pub url: String,
    pub source_url: String,
    pub blocked: bool,
    pub reason: Option<String>,
    pub list_name: Option<String>,
    pub timestamp: u64,
}

/// Navigation timing data.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NavigationTiming {
    pub url: String,
    pub dns_lookup_ms: u64,
    pub tcp_connect_ms: u64,
    pub tls_handshake_ms: u64,
    pub time_to_first_byte_ms: u64,
    pub content_download_ms: u64,
    pub dom_interactive_ms: u64,
    pub dom_complete_ms: u64,
    pub load_event_ms: u64,
    pub total_ms: u64,
}

/// Cache status for a request.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CacheStatus {
    Hit,
    Miss,
    Revalidate,
    Bypass,
    Error,
}

/// Network diagnostics.
pub struct NetworkDiagnostics {
    /// Recent request decisions.
    request_decisions: Vec<RequestDecision>,
    /// Navigation timings.
    navigation_timings: Vec<NavigationTiming>,
    /// Cache statuses.
    cache_statuses: HashMap<String, CacheStatus>,
    /// Maximum history size.
    max_history: usize,
}

impl NetworkDiagnostics {
    pub fn new() -> Self {
        Self {
            request_decisions: Vec::new(),
            navigation_timings: Vec::new(),
            cache_statuses: HashMap::new(),
            max_history: 500,
        }
    }

    /// Record a request decision.
    pub fn record_decision(&mut self, decision: RequestDecision) {
        self.request_decisions.push(decision);
        if self.request_decisions.len() > self.max_history {
            self.request_decisions.remove(0);
        }
    }

    /// Record navigation timing.
    pub fn record_timing(&mut self, timing: NavigationTiming) {
        self.navigation_timings.push(timing);
        if self.navigation_timings.len() > self.max_history {
            self.navigation_timings.remove(0);
        }
    }

    /// Update cache status.
    pub fn update_cache(&mut self, url: impl Into<String>, status: CacheStatus) {
        self.cache_statuses.insert(url.into(), status);
    }

    /// Get recent request decisions.
    pub fn recent_decisions(&self, n: usize) -> Vec<&RequestDecision> {
        self.request_decisions.iter().rev().take(n).collect()
    }

    /// Get blocked requests count.
    pub fn blocked_count(&self) -> usize {
        self.request_decisions.iter().filter(|d| d.blocked).count()
    }

    /// Get allowed requests count.
    pub fn allowed_count(&self) -> usize {
        self.request_decisions.iter().filter(|d| !d.blocked).count()
    }

    /// Get average navigation time.
    pub fn avg_navigation_time(&self) -> u64 {
        if self.navigation_timings.is_empty() {
            return 0;
        }
        let total: u64 = self.navigation_timings.iter().map(|t| t.total_ms).sum();
        total / self.navigation_timings.len() as u64
    }

    /// Clear history.
    pub fn clear(&mut self) {
        self.request_decisions.clear();
        self.navigation_timings.clear();
        self.cache_statuses.clear();
    }
}

impl Default for NetworkDiagnostics {
    fn default() -> Self {
        Self::new()
    }
}

// ── Crash Diagnostics ──────────────────────────────────────────────

/// Crash report.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CrashReport {
    pub id: String,
    pub timestamp: u64,
    pub platform: String,
    pub version: String,
    pub reason: String,
    pub backtrace: Option<String>,
    pub memory_at_crash: u64,
    pub open_tabs: usize,
    pub additional_info: HashMap<String, String>,
}

/// Crash diagnostics collector.
pub struct CrashDiagnostics {
    /// Crash reports.
    reports: Vec<CrashReport>,
    /// Maximum reports to keep.
    max_reports: usize,
}

impl CrashDiagnostics {
    pub fn new() -> Self {
        Self {
            reports: Vec::new(),
            max_reports: 100,
        }
    }

    /// Record a crash report.
    pub fn record_crash(&mut self, report: CrashReport) {
        self.reports.push(report);
        if self.reports.len() > self.max_reports {
            self.reports.remove(0);
        }
    }

    /// Get all crash reports.
    pub fn reports(&self) -> &[CrashReport] {
        &self.reports
    }

    /// Get recent crashes.
    pub fn recent_crashes(&self, n: usize) -> Vec<&CrashReport> {
        self.reports.iter().rev().take(n).collect()
    }

    /// Generate diagnostic bundle.
    pub fn generate_bundle(&self) -> DiagnosticBundle {
        DiagnosticBundle {
            timestamp: now_secs(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            crashes: self.reports.clone(),
            system_info: SystemInfo::collect(),
        }
    }
}

impl Default for CrashDiagnostics {
    fn default() -> Self {
        Self::new()
    }
}

/// Diagnostic bundle for support.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiagnosticBundle {
    pub timestamp: u64,
    pub version: String,
    pub crashes: Vec<CrashReport>,
    pub system_info: SystemInfo,
}

/// System information for diagnostics.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub cpu_cores: usize,
    pub total_memory: u64,
    pub gpu: Option<String>,
}

impl SystemInfo {
    pub fn collect() -> Self {
        Self {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            cpu_cores: num_cpus(),
            total_memory: 0,
            gpu: None,
        }
    }
}

/// Get number of CPU cores.
fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
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

    #[test]
    fn resource_inspector() {
        let mut inspector = ResourceInspector::new();

        inspector.update_processes(vec![ProcessInfo {
            pid: 1,
            tab_id: Some(1),
            process_type: ProcessType::Renderer,
            memory_bytes: 100 * 1024 * 1024,
            cpu_percent: 5.0,
            state: "running".to_string(),
            uptime_secs: 3600,
        }]);

        assert_eq!(inspector.process_count(), 1);
        assert_eq!(inspector.total_memory(), 100 * 1024 * 1024);
    }

    #[test]
    fn network_diagnostics() {
        let mut net = NetworkDiagnostics::new();

        net.record_decision(RequestDecision {
            url: "https://ads.example.com".to_string(),
            source_url: "https://example.com".to_string(),
            blocked: true,
            reason: Some("tracker".to_string()),
            list_name: Some("EasyList".to_string()),
            timestamp: now_secs(),
        });

        assert_eq!(net.blocked_count(), 1);
        assert_eq!(net.allowed_count(), 0);
    }

    #[test]
    fn crash_diagnostics() {
        let mut crash = CrashDiagnostics::new();

        crash.record_crash(CrashReport {
            id: "crash-001".to_string(),
            timestamp: now_secs(),
            platform: "windows".to_string(),
            version: "0.0.1".to_string(),
            reason: "SEGFAULT".to_string(),
            backtrace: None,
            memory_at_crash: 500 * 1024 * 1024,
            open_tabs: 5,
            additional_info: HashMap::new(),
        });

        assert_eq!(crash.reports().len(), 1);

        let bundle = crash.generate_bundle();
        assert_eq!(bundle.crashes.len(), 1);
    }
}
