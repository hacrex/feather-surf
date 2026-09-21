// FeatherSurf Developer Center
//
// Aggregates resource data from all subsystems for display in the browser UI.
// Provides browser-wide summaries, per-tab details, and diagnostics export.

use std::collections::HashMap;
use std::time::{Duration, SystemTime};

use crate::renderer::{RendererAdapter, RendererState};
use crate::{BudgetManager, MemoryBudget, MemoryMode, TabObservation};

// ── Types ──────────────────────────────────────────────────────────

/// Browser-wide resource summary.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BrowserSummary {
    pub timestamp: u64,
    pub total_tabs: usize,
    pub active_tabs: usize,
    pub frozen_tabs: usize,
    pub suspended_tabs: usize,
    pub discarded_tabs: usize,
    pub total_memory_bytes: u64,
    pub budget: Option<BudgetInfo>,
    pub memory_pressure: String,
    pub mode: String,
}

/// Budget information for display.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BudgetInfo {
    pub budget_type: String,
    pub used_bytes: u64,
    pub limit_bytes: Option<u64>,
    pub warning: bool,
    pub critical: bool,
}

/// Per-tab resource detail.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TabDetail {
    pub tab_id: u64,
    pub url: String,
    pub title: String,
    pub state: String,
    pub memory_bytes: u64,
    pub cpu_percent: f32,
    pub score: f64,
    pub last_active_secs: u64,
    pub pinned: bool,
    pub keep_awake: bool,
}

/// State transition history entry.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TransitionRecord {
    pub tab_id: u64,
    pub from_state: String,
    pub to_state: String,
    pub timestamp: u64,
    pub reason: String,
}

/// Exported diagnostics.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Diagnostics {
    pub timestamp: u64,
    pub version: String,
    pub summary: BrowserSummary,
    pub tabs: Vec<TabDetail>,
    pub recent_transitions: Vec<TransitionRecord>,
    pub system_memory: SystemMemoryInfo,
}

/// System memory info for diagnostics.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemMemoryInfo {
    pub total_ram: u64,
    pub available_ram: u64,
    pub used_ram: u64,
}

// ── Developer Center ───────────────────────────────────────────────

/// Central resource visibility engine.
pub struct DeveloperCenter {
    /// Per-tab observations.
    observations: HashMap<u64, TabObservation>,
    /// Per-tab memory usage.
    memory_usage: HashMap<u64, u64>,
    /// Per-tab URLs and titles.
    tab_info: HashMap<u64, (String, String)>,
    /// State transition history.
    transitions: Vec<TransitionRecord>,
    /// Max history entries.
    max_transitions: usize,
    /// System memory.
    system_memory: SystemMemoryInfo,
}

impl Default for DeveloperCenter {
    fn default() -> Self {
        Self::new()
    }
}

impl DeveloperCenter {
    pub fn new() -> Self {
        Self {
            observations: HashMap::new(),
            memory_usage: HashMap::new(),
            tab_info: HashMap::new(),
            transitions: Vec::new(),
            max_transitions: 500,
            system_memory: SystemMemoryInfo {
                total_ram: 0,
                available_ram: 0,
                used_ram: 0,
            },
        }
    }

    /// Create with custom history limit.
    pub fn with_history_limit(max_transitions: usize) -> Self {
        Self {
            max_transitions,
            ..Self::new()
        }
    }

    /// Register a tab.
    pub fn register_tab(&mut self, tab_id: u64, url: String, title: String) {
        self.tab_info.insert(tab_id, (url, title));
        self.observations
            .insert(tab_id, TabObservation::default());
        self.memory_usage.insert(tab_id, 0);
    }

    /// Unregister a tab.
    pub fn unregister_tab(&mut self, tab_id: u64) {
        self.tab_info.remove(&tab_id);
        self.observations.remove(&tab_id);
        self.memory_usage.remove(&tab_id);
    }

    /// Update tab observation.
    pub fn update_observation(&mut self, tab_id: u64, observation: TabObservation) {
        self.observations.insert(tab_id, observation);
    }

    /// Update tab memory usage.
    pub fn update_memory(&mut self, tab_id: u64, bytes: u64) {
        self.memory_usage.insert(tab_id, bytes);
    }

    /// Update tab URL/title.
    pub fn update_tab_info(&mut self, tab_id: u64, url: String, title: String) {
        self.tab_info.insert(tab_id, (url, title));
    }

    /// Record a state transition.
    pub fn record_transition(
        &mut self,
        tab_id: u64,
        from: RendererState,
        to: RendererState,
        reason: String,
    ) {
        self.transitions.push(TransitionRecord {
            tab_id,
            from_state: format!("{:?}", from),
            to_state: format!("{:?}", to),
            timestamp: now_secs(),
            reason,
        });

        // Trim old entries
        if self.transitions.len() > self.max_transitions {
            let excess = self.transitions.len() - self.max_transitions;
            self.transitions.drain(..excess);
        }
    }

    /// Update system memory info.
    pub fn update_system_memory(&mut self, info: SystemMemoryInfo) {
        self.system_memory = info;
    }

    /// Get browser-wide summary.
    pub fn summary(
        &self,
        renderer: &RendererAdapter,
        budget_manager: &BudgetManager,
        mode: MemoryMode,
    ) -> BrowserSummary {
        let state_counts = renderer.state_counts();
        let total_memory: u64 = self.memory_usage.values().sum();

        let budget = budget_manager.effective_budget().map(|limit| BudgetInfo {
            budget_type: format!("{:?}", budget_manager.budget()),
            used_bytes: total_memory,
            limit_bytes: Some(limit),
            warning: budget_manager.is_warning(total_memory),
            critical: budget_manager.is_critical(total_memory),
        });

        BrowserSummary {
            timestamp: now_secs(),
            total_tabs: self.tab_info.len(),
            active_tabs: state_counts.get(&RendererState::Active).copied().unwrap_or(0),
            frozen_tabs: state_counts.get(&RendererState::Frozen).copied().unwrap_or(0),
            suspended_tabs: state_counts.get(&RendererState::Suspended).copied().unwrap_or(0),
            discarded_tabs: state_counts.get(&RendererState::Discarded).copied().unwrap_or(0),
            total_memory_bytes: total_memory,
            budget,
            memory_pressure: format!("{:?}", budget_manager.is_critical(total_memory)),
            mode: format!("{:?}", mode),
        }
    }

    /// Get per-tab details.
    pub fn tab_details(&self, renderer: &RendererAdapter) -> Vec<TabDetail> {
        self.tab_info
            .iter()
            .map(|(&tab_id, (url, title))| {
                let observation = self.observations.get(&tab_id).cloned().unwrap_or_default();
                let score = observation.keep_resident_score();
                let state = renderer.state(tab_id);
                let time_in_state = renderer.time_in_state(tab_id);

                TabDetail {
                    tab_id,
                    url: url.clone(),
                    title: title.clone(),
                    state: format!("{:?}", state),
                    memory_bytes: self.memory_usage.get(&tab_id).copied().unwrap_or(0),
                    cpu_percent: observation.cpu * 100.0,
                    score,
                    last_active_secs: time_in_state.as_secs(),
                    pinned: observation.pinned > 0.5,
                    keep_awake: false, // Would come from TabProtection
                }
            })
            .collect()
    }

    /// Get recent state transitions.
    pub fn recent_transitions(&self, n: usize) -> Vec<&TransitionRecord> {
        self.transitions.iter().rev().take(n).collect()
    }

    /// Export diagnostics for debugging.
    pub fn export_diagnostics(
        &self,
        renderer: &RendererAdapter,
        budget_manager: &BudgetManager,
        mode: MemoryMode,
    ) -> Diagnostics {
        Diagnostics {
            timestamp: now_secs(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            summary: self.summary(renderer, budget_manager, mode),
            tabs: self.tab_details(renderer),
            recent_transitions: self.recent_transitions(50).into_iter().cloned().collect(),
            system_memory: self.system_memory.clone(),
        }
    }

    /// Get total browser memory.
    pub fn total_memory(&self) -> u64 {
        self.memory_usage.values().sum()
    }

    /// Get memory for a specific tab.
    pub fn tab_memory(&self, tab_id: u64) -> u64 {
        self.memory_usage.get(&tab_id).copied().unwrap_or(0)
    }

    /// Get tab count.
    pub fn tab_count(&self) -> usize {
        self.tab_info.len()
    }

    /// Clear all data.
    pub fn clear(&mut self) {
        self.observations.clear();
        self.memory_usage.clear();
        self.tab_info.clear();
        self.transitions.clear();
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

    #[test]
    fn register_and_get_summary() {
        let mut center = DeveloperCenter::new();
        let renderer = RendererAdapter::new();
        let budget = BudgetManager::new();

        center.register_tab(1, "https://example.com".into(), "Example".into());
        center.register_tab(2, "https://rust-lang.org".into(), "Rust".into());

        let summary = center.summary(&renderer, &budget, MemoryMode::Balanced);
        assert_eq!(summary.total_tabs, 2);
        assert_eq!(summary.active_tabs, 2);
    }

    #[test]
    fn tab_details() {
        let mut center = DeveloperCenter::new();
        let renderer = RendererAdapter::new();
        let budget = BudgetManager::new();

        center.register_tab(1, "https://example.com".into(), "Example".into());
        center.update_memory(1, 1024 * 1024);

        let details = center.tab_details(&renderer);
        assert_eq!(details.len(), 1);
        assert_eq!(details[0].memory_bytes, 1024 * 1024);
    }

    #[test]
    fn record_transition() {
        let mut center = DeveloperCenter::new();
        let renderer = RendererAdapter::new();
        let budget = BudgetManager::new();

        center.register_tab(1, "https://example.com".into(), "Example".into());
        center.record_transition(
            1,
            RendererState::Active,
            RendererState::Frozen,
            "background tab".into(),
        );

        let transitions = center.recent_transitions(10);
        assert_eq!(transitions.len(), 1);
        assert_eq!(transitions[0].to_state, "Frozen");
    }

    #[test]
    fn export_diagnostics() {
        let mut center = DeveloperCenter::new();
        let mut renderer = RendererAdapter::new();
        let budget = BudgetManager::new();

        renderer.register_tab(1);
        center.register_tab(1, "https://example.com".into(), "Example".into());

        let diag = center.export_diagnostics(&renderer, &budget, MemoryMode::Balanced);
        assert_eq!(diag.tabs.len(), 1);
        assert!(!diag.version.is_empty());
    }

    #[test]
    fn transition_history_limit() {
        let mut center = DeveloperCenter::with_history_limit(5);
        let renderer = RendererAdapter::new();
        let budget = BudgetManager::new();

        center.register_tab(1, "https://example.com".into(), "Example".into());
        for i in 0..10 {
            center.record_transition(
                1,
                RendererState::Active,
                RendererState::Frozen,
                format!("reason {}", i),
            );
        }
        assert!(center.recent_transitions(100).len() <= 5);
    }

    #[test]
    fn system_memory_update() {
        let mut center = DeveloperCenter::new();
        center.update_system_memory(SystemMemoryInfo {
            total_ram: 8_000_000_000,
            available_ram: 4_000_000_000,
            used_ram: 4_000_000_000,
        });
        let diag = center.export_diagnostics(&RendererAdapter::new(), &BudgetManager::new(), MemoryMode::Balanced);
        assert_eq!(diag.system_memory.total_ram, 8_000_000_000);
    }

    #[test]
    fn unregister_tab() {
        let mut center = DeveloperCenter::new();
        center.register_tab(1, "https://example.com".into(), "Example".into());
        center.unregister_tab(1);
        assert_eq!(center.tab_count(), 0);
    }
}
