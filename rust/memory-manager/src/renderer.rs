// FeatherSurf Renderer Adapter
//
// Commands and state management for freezing, suspending, discarding,
// and restoring browser tabs across different rendering engines.

use std::time::{Duration, SystemTime};

// ── Commands ───────────────────────────────────────────────────────

/// Commands sent to the renderer process.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RendererCommand {
    /// Freeze JavaScript execution and timers.
    /// The renderer stays in memory but stops all background work.
    Freeze,
    /// Unfreeze a previously frozen renderer.
    Unfreeze,
    /// Suspend the renderer process (pause or hibernate).
    /// State is serialized before suspension.
    Suspend,
    /// Discard the renderer and release memory.
    /// Only valid after Suspend with valid snapshot.
    Discard,
    /// Restore a suspended/discarded tab from snapshot.
    Restore {
        url: String,
        scroll_x: i32,
        scroll_y: i32,
    },
    /// Cancel an in-progress reclamation if tab becomes active.
    CancelReclamation,
}

/// Result of a renderer command.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum RendererResult {
    /// Command succeeded.
    Success,
    /// Command failed with reason.
    Failed(String),
    /// Command timed out.
    Timeout,
    /// Tab was already in the requested state.
    AlreadyInState,
}

/// State of a renderer process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum RendererState {
    /// Renderer is active and running.
    Active,
    /// Renderer is frozen (JS paused, in memory).
    Frozen,
    /// Renderer is suspended (process paused/hibernated).
    Suspended,
    /// Renderer was discarded (process killed, needs restore).
    Discarded,
    /// Restore is in progress.
    Restoring,
}

// ── Renderer Adapter ───────────────────────────────────────────────

/// Tracks renderer state and manages command lifecycle.
pub struct RendererAdapter {
    /// Current state per tab.
    tab_states: std::collections::HashMap<u64, RendererState>,
    /// Timestamps of last state change per tab.
    state_timestamps: std::collections::HashMap<u64, u64>,
    /// Pending commands per tab.
    pending_commands: std::collections::HashMap<u64, RendererCommand>,
    /// Command timeout.
    command_timeout: Duration,
    /// Snapshot data for suspended tabs.
    snapshots: std::collections::HashMap<u64, Vec<u8>>,
}

impl Default for RendererAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl RendererAdapter {
    pub fn new() -> Self {
        Self {
            tab_states: std::collections::HashMap::new(),
            state_timestamps: std::collections::HashMap::new(),
            pending_commands: std::collections::HashMap::new(),
            command_timeout: Duration::from_secs(30),
            snapshots: std::collections::HashMap::new(),
        }
    }

    /// Create with custom timeout.
    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            command_timeout: timeout,
            ..Self::new()
        }
    }

    /// Register a tab with initial state.
    pub fn register_tab(&mut self, tab_id: u64) {
        self.tab_states.insert(tab_id, RendererState::Active);
        self.state_timestamps.insert(tab_id, now_secs());
    }

    /// Unregister a tab.
    pub fn unregister_tab(&mut self, tab_id: u64) {
        self.tab_states.remove(&tab_id);
        self.state_timestamps.remove(&tab_id);
        self.pending_commands.remove(&tab_id);
        self.snapshots.remove(&tab_id);
    }

    /// Get current renderer state for a tab.
    pub fn state(&self, tab_id: u64) -> RendererState {
        self.tab_states
            .get(&tab_id)
            .copied()
            .unwrap_or(RendererState::Active)
    }

    /// Check if a command can be executed on a tab.
    pub fn can_execute(&self, tab_id: u64, command: &RendererCommand) -> bool {
        let current = self.state(tab_id);

        // Can always cancel
        if matches!(command, RendererCommand::CancelReclamation) {
            return true;
        }

        // Can't execute if there's a pending command
        if self.pending_commands.contains_key(&tab_id) {
            return false;
        }

        match command {
            RendererCommand::Freeze => current == RendererState::Active,
            RendererCommand::Unfreeze => current == RendererState::Frozen,
            RendererCommand::Suspend => {
                current == RendererState::Frozen || current == RendererState::Active
            }
            RendererCommand::Discard => current == RendererState::Suspended,
            RendererCommand::Restore { .. } => {
                current == RendererState::Suspended || current == RendererState::Discarded
            }
            RendererCommand::CancelReclamation => unreachable!(),
        }
    }

    /// Execute a command (simulated - real implementation calls engine).
    pub fn execute(&mut self, tab_id: u64, command: RendererCommand) -> RendererResult {
        if !self.can_execute(tab_id, &command) {
            return RendererResult::AlreadyInState;
        }

        let now = now_secs();

        match &command {
            RendererCommand::Freeze => {
                self.tab_states.insert(tab_id, RendererState::Frozen);
                self.state_timestamps.insert(tab_id, now);
                RendererResult::Success
            }
            RendererCommand::Unfreeze => {
                self.tab_states.insert(tab_id, RendererState::Active);
                self.state_timestamps.insert(tab_id, now);
                RendererResult::Success
            }
            RendererCommand::Suspend => {
                self.tab_states.insert(tab_id, RendererState::Suspended);
                self.state_timestamps.insert(tab_id, now);
                RendererResult::Success
            }
            RendererCommand::Discard => {
                self.tab_states.insert(tab_id, RendererState::Discarded);
                self.state_timestamps.insert(tab_id, now);
                RendererResult::Success
            }
            RendererCommand::Restore {
                url: _,
                scroll_x: _,
                scroll_y: _,
            } => {
                self.tab_states.insert(tab_id, RendererState::Restoring);
                self.state_timestamps.insert(tab_id, now);
                // In real implementation, this would restore from snapshot
                self.snapshots.remove(&tab_id);
                self.tab_states.insert(tab_id, RendererState::Active);
                RendererResult::Success
            }
            RendererCommand::CancelReclamation => {
                self.pending_commands.remove(&tab_id);
                RendererResult::Success
            }
        }
    }

    /// Store snapshot data for a tab.
    pub fn store_snapshot(&mut self, tab_id: u64, data: Vec<u8>) {
        self.snapshots.insert(tab_id, data);
    }

    /// Get snapshot data for a tab.
    pub fn get_snapshot(&self, tab_id: u64) -> Option<&Vec<u8>> {
        self.snapshots.get(&tab_id)
    }

    /// Check if a tab has a valid snapshot.
    pub fn has_snapshot(&self, tab_id: u64) -> bool {
        self.snapshots.contains_key(&tab_id)
    }

    /// Check if a command has timed out.
    pub fn is_timed_out(&self, tab_id: u64) -> bool {
        if let Some(_cmd) = self.pending_commands.get(&tab_id) {
            if let Some(&ts) = self.state_timestamps.get(&tab_id) {
                let elapsed = now_secs().saturating_sub(ts);
                return elapsed > self.command_timeout.as_secs();
            }
        }
        false
    }

    /// Get time since last state change.
    pub fn time_in_state(&self, tab_id: u64) -> Duration {
        let now = now_secs();
        let ts = self.state_timestamps.get(&tab_id).copied().unwrap_or(now);
        Duration::from_secs(now.saturating_sub(ts))
    }

    /// Get all tabs in a specific state.
    pub fn tabs_in_state(&self, state: RendererState) -> Vec<u64> {
        self.tab_states
            .iter()
            .filter(|(_, &s)| s == state)
            .map(|(&id, _)| id)
            .collect()
    }

    /// Get tab count per state.
    pub fn state_counts(&self) -> std::collections::HashMap<RendererState, usize> {
        let mut counts = std::collections::HashMap::new();
        for &state in self.tab_states.values() {
            *counts.entry(state).or_insert(0) += 1;
        }
        counts
    }

    /// Clear all state.
    pub fn clear(&mut self) {
        self.tab_states.clear();
        self.state_timestamps.clear();
        self.pending_commands.clear();
        self.snapshots.clear();
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
    fn register_and_get_state() {
        let mut adapter = RendererAdapter::new();
        adapter.register_tab(1);
        assert_eq!(adapter.state(1), RendererState::Active);
    }

    #[test]
    fn freeze_unfreeze_cycle() {
        let mut adapter = RendererAdapter::new();
        adapter.register_tab(1);

        assert!(adapter.can_execute(1, &RendererCommand::Freeze));
        assert_eq!(
            adapter.execute(1, RendererCommand::Freeze),
            RendererResult::Success
        );
        assert_eq!(adapter.state(1), RendererState::Frozen);

        assert!(adapter.can_execute(1, &RendererCommand::Unfreeze));
        assert_eq!(
            adapter.execute(1, RendererCommand::Unfreeze),
            RendererResult::Success
        );
        assert_eq!(adapter.state(1), RendererState::Active);
    }

    #[test]
    fn suspend_from_active() {
        let mut adapter = RendererAdapter::new();
        adapter.register_tab(1);

        assert!(adapter.can_execute(1, &RendererCommand::Suspend));
        assert_eq!(
            adapter.execute(1, RendererCommand::Suspend),
            RendererResult::Success
        );
        assert_eq!(adapter.state(1), RendererState::Suspended);
    }

    #[test]
    fn suspend_from_frozen() {
        let mut adapter = RendererAdapter::new();
        adapter.register_tab(1);
        adapter.execute(1, RendererCommand::Freeze);

        assert!(adapter.can_execute(1, &RendererCommand::Suspend));
        assert_eq!(
            adapter.execute(1, RendererCommand::Suspend),
            RendererResult::Success
        );
        assert_eq!(adapter.state(1), RendererState::Suspended);
    }

    #[test]
    fn discard_after_suspend() {
        let mut adapter = RendererAdapter::new();
        adapter.register_tab(1);
        adapter.execute(1, RendererCommand::Suspend);

        assert!(adapter.can_execute(1, &RendererCommand::Discard));
        assert_eq!(
            adapter.execute(1, RendererCommand::Discard),
            RendererResult::Success
        );
        assert_eq!(adapter.state(1), RendererState::Discarded);
    }

    #[test]
    fn cannot_discard_active() {
        let mut adapter = RendererAdapter::new();
        adapter.register_tab(1);

        assert!(!adapter.can_execute(1, &RendererCommand::Discard));
    }

    #[test]
    fn restore_from_suspended() {
        let mut adapter = RendererAdapter::new();
        adapter.register_tab(1);
        adapter.store_snapshot(1, vec![1, 2, 3]);
        adapter.execute(1, RendererCommand::Suspend);

        let cmd = RendererCommand::Restore {
            url: "https://example.com".into(),
            scroll_x: 0,
            scroll_y: 100,
        };
        assert!(adapter.can_execute(1, &cmd));
        assert_eq!(adapter.execute(1, cmd), RendererResult::Success);
        assert_eq!(adapter.state(1), RendererState::Active);
        assert!(!adapter.has_snapshot(1));
    }

    #[test]
    fn restore_from_discarded() {
        let mut adapter = RendererAdapter::new();
        adapter.register_tab(1);
        adapter.execute(1, RendererCommand::Suspend);
        adapter.execute(1, RendererCommand::Discard);

        let cmd = RendererCommand::Restore {
            url: "https://example.com".into(),
            scroll_x: 0,
            scroll_y: 0,
        };
        assert!(adapter.can_execute(1, &cmd));
        assert_eq!(adapter.execute(1, cmd), RendererResult::Success);
        assert_eq!(adapter.state(1), RendererState::Active);
    }

    #[test]
    fn cancel_reclamation() {
        let mut adapter = RendererAdapter::new();
        adapter.register_tab(1);

        assert!(adapter.can_execute(1, &RendererCommand::CancelReclamation));
        assert_eq!(
            adapter.execute(1, RendererCommand::CancelReclamation),
            RendererResult::Success
        );
    }

    #[test]
    fn cannot_freeze_already_frozen() {
        let mut adapter = RendererAdapter::new();
        adapter.register_tab(1);
        adapter.execute(1, RendererCommand::Freeze);

        assert!(!adapter.can_execute(1, &RendererCommand::Freeze));
        assert_eq!(
            adapter.execute(1, RendererCommand::Freeze),
            RendererResult::AlreadyInState
        );
    }

    #[test]
    fn tabs_in_state() {
        let mut adapter = RendererAdapter::new();
        adapter.register_tab(1);
        adapter.register_tab(2);
        adapter.register_tab(3);
        adapter.execute(1, RendererCommand::Freeze);
        adapter.execute(3, RendererCommand::Suspend);

        let frozen = adapter.tabs_in_state(RendererState::Frozen);
        assert_eq!(frozen, vec![1]);

        let suspended = adapter.tabs_in_state(RendererState::Suspended);
        assert_eq!(suspended, vec![3]);
    }

    #[test]
    fn state_counts() {
        let mut adapter = RendererAdapter::new();
        adapter.register_tab(1);
        adapter.register_tab(2);
        adapter.register_tab(3);
        adapter.execute(1, RendererCommand::Freeze);
        adapter.execute(2, RendererCommand::Freeze);
        adapter.execute(3, RendererCommand::Suspend);

        let counts = adapter.state_counts();
        assert_eq!(counts.get(&RendererState::Frozen), Some(&2));
        assert_eq!(counts.get(&RendererState::Suspended), Some(&1));
    }

    #[test]
    fn snapshot_storage() {
        let mut adapter = RendererAdapter::new();
        adapter.register_tab(1);

        assert!(!adapter.has_snapshot(1));
        adapter.store_snapshot(1, vec![1, 2, 3]);
        assert!(adapter.has_snapshot(1));
        assert_eq!(adapter.get_snapshot(1).unwrap(), &vec![1, 2, 3]);
    }

    #[test]
    fn time_in_state() {
        let mut adapter = RendererAdapter::new();
        adapter.register_tab(1);
        let before = adapter.time_in_state(1);
        // Should be very small
        assert!(before.as_millis() < 100);
    }

    #[test]
    fn unregister_tab() {
        let mut adapter = RendererAdapter::new();
        adapter.register_tab(1);
        adapter.store_snapshot(1, vec![1, 2, 3]);
        adapter.unregister_tab(1);

        assert_eq!(adapter.state(1), RendererState::Active); // Default
        assert!(!adapter.has_snapshot(1));
    }
}
