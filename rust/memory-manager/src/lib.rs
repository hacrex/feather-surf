//! Deterministic memory scoring and tab eviction orchestration.
//!
//! This crate owns policy decisions. Renderer/process adapters should provide
//! observations and execute the resulting state changes outside this crate.

use std::collections::HashMap;
use tab_manager::{Tab, TabProtection, TabState};

const SAMPLE_DEBOUNCE: u8 = 3;
const RESTORE_COOLDOWN_SECS: u64 = 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryMode {
    Lite,
    Balanced,
    Performance,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TabObservation {
    pub activity: f64,
    pub media: f64,
    pub user_interaction: f64,
    pub network: f64,
    pub memory: f64,
    pub cpu: f64,
    pub pinned: f64,
    pub foreground: f64,
}

impl Default for TabObservation {
    fn default() -> Self {
        Self {
            activity: 0.0,
            media: 0.0,
            user_interaction: 0.0,
            network: 0.0,
            memory: 0.0,
            cpu: 0.0,
            pinned: 0.0,
            foreground: 0.0,
        }
    }
}

impl TabObservation {
    pub fn idle() -> Self {
        Self::default()
    }

    pub fn active() -> Self {
        Self {
            activity: 1.0,
            foreground: 1.0,
            ..Self::default()
        }
    }

    pub fn keep_resident_score(self) -> f64 {
        let activity = clamp(self.activity);
        let media = clamp(self.media);
        let user_interaction = clamp(self.user_interaction);
        let network = clamp(self.network);
        let memory = clamp(self.memory);
        let cpu = clamp(self.cpu);
        let pinned = clamp(self.pinned);
        let foreground = clamp(self.foreground);
        let inactivity = 1.0
            - foreground
                .max(activity)
                .max(media)
                .max(user_interaction)
                .max(pinned);

        clamp(
            0.30 * foreground
                + 0.20 * activity
                + 0.15 * media
                + 0.10 * user_interaction
                + 0.05 * network
                + 0.10 * pinned
                - inactivity * (0.06 * memory + 0.04 * cpu),
        )
    }

    pub fn reclaim_pressure(self, budget_pressure: f64) -> f64 {
        let inactivity = 1.0
            - clamp(
                clamp(self.foreground)
                    .max(clamp(self.activity))
                    .max(clamp(self.media))
                    .max(clamp(self.user_interaction))
                    .max(clamp(self.pinned)),
            );
        clamp(1.0 - self.keep_resident_score() + 0.25 * clamp(budget_pressure) * inactivity)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PolicyConfig {
    pub mode: MemoryMode,
    pub sample_debounce: u8,
    pub recently_active_dwell_secs: u64,
    pub frozen_dwell_secs: u64,
    pub suspended_dwell_secs: u64,
    pub discardable_dwell_secs: u64,
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            mode: MemoryMode::Balanced,
            sample_debounce: SAMPLE_DEBOUNCE,
            recently_active_dwell_secs: 10,
            frozen_dwell_secs: 30,
            suspended_dwell_secs: 60,
            discardable_dwell_secs: 120,
        }
    }
}

impl PolicyConfig {
    fn frozen_entry(self) -> f64 {
        match self.mode {
            MemoryMode::Lite => 0.35,
            MemoryMode::Balanced => 0.45,
            MemoryMode::Performance => 0.58,
        }
    }

    fn suspended_entry(self) -> f64 {
        match self.mode {
            MemoryMode::Lite => 0.58,
            MemoryMode::Balanced => 0.68,
            MemoryMode::Performance => 0.78,
        }
    }

    fn discard_entry(self) -> f64 {
        match self.mode {
            MemoryMode::Lite => 0.78,
            MemoryMode::Balanced => 0.86,
            MemoryMode::Performance => 0.93,
        }
    }

    fn dwell(self, seconds: u64) -> u64 {
        let multiplier = match self.mode {
            MemoryMode::Lite => 0.75,
            MemoryMode::Balanced => 1.0,
            MemoryMode::Performance => 1.5,
        };
        (seconds as f64 * multiplier) as u64
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EvictionEvent {
    Transition {
        tab_id: u64,
        from: TabState,
        to: TabState,
        reclaim_pressure: f64,
    },
    Blocked {
        tab_id: u64,
        target: TabState,
        protection: TabProtection,
    },
}

#[derive(Debug, Clone)]
struct ManagedTab {
    tab: Tab,
    observation: TabObservation,
    entered_at: u64,
    candidate: Option<(TabState, u8)>,
    restore_cooldown_until: u64,
}

#[derive(Debug)]
pub struct EvictionManager {
    config: PolicyConfig,
    tabs: HashMap<u64, ManagedTab>,
    browser_working_set: f64,
    memory_budget: Option<f64>,
}

impl EvictionManager {
    pub fn new(config: PolicyConfig, memory_budget: Option<f64>) -> Self {
        Self {
            config,
            tabs: HashMap::new(),
            browser_working_set: 0.0,
            memory_budget,
        }
    }

    pub fn add_tab(&mut self, tab: Tab, observation: TabObservation, now: u64) {
        self.tabs.insert(
            tab.id,
            ManagedTab {
                tab,
                observation,
                entered_at: now,
                candidate: None,
                restore_cooldown_until: 0,
            },
        );
    }

    pub fn update_observation(&mut self, tab_id: u64, observation: TabObservation) -> bool {
        if let Some(managed) = self.tabs.get_mut(&tab_id) {
            managed.observation = observation;
            true
        } else {
            false
        }
    }

    pub fn set_browser_working_set(&mut self, bytes: f64) {
        self.browser_working_set = bytes.max(0.0);
    }

    pub fn tab(&self, tab_id: u64) -> Option<&Tab> {
        self.tabs.get(&tab_id).map(|managed| &managed.tab)
    }

    pub fn tab_mut(&mut self, tab_id: u64) -> Option<&mut Tab> {
        self.tabs.get_mut(&tab_id).map(|managed| &mut managed.tab)
    }

    pub fn focus_tab(&mut self, tab_id: u64, now: u64) -> Result<(), EvictionError> {
        let managed = self
            .tabs
            .get_mut(&tab_id)
            .ok_or(EvictionError::UnknownTab(tab_id))?;
        if managed.tab.state() != TabState::Active {
            managed
                .tab
                .transition_to(TabState::Active)
                .map_err(EvictionError::Transition)?;
        }
        managed.entered_at = now;
        managed.candidate = None;
        managed.restore_cooldown_until = now.saturating_add(RESTORE_COOLDOWN_SECS);
        Ok(())
    }

    pub fn tick(&mut self, now: u64) -> Vec<EvictionEvent> {
        let budget_pressure = self.budget_pressure();
        let ids: Vec<u64> = self.tabs.keys().copied().collect();
        let mut events = Vec::new();

        for id in ids {
            let Some(managed) = self.tabs.get_mut(&id) else {
                continue;
            };
            let state = managed.tab.state();
            if state == TabState::Active || now < managed.restore_cooldown_until {
                managed.candidate = None;
                continue;
            }
            if managed.tab.protection().blocks_reclaim() {
                managed.candidate = None;
                continue;
            }

            let pressure = managed.observation.reclaim_pressure(budget_pressure);
            let target = target_for(state, pressure, budget_pressure, self.config);
            let Some(target) = target else {
                managed.candidate = None;
                continue;
            };

            let dwell = dwell_for(state, target, self.config);
            if now.saturating_sub(managed.entered_at) < dwell {
                managed.candidate = None;
                continue;
            }

            let count = match managed.candidate {
                Some((candidate, count)) if candidate == target => count.saturating_add(1),
                _ => 1,
            };
            managed.candidate = Some((target, count));
            if count < self.config.sample_debounce {
                continue;
            }

            let from = managed.tab.state();
            match managed.tab.transition_to(target) {
                Ok(()) => {
                    managed.entered_at = now;
                    managed.candidate = None;
                    events.push(EvictionEvent::Transition {
                        tab_id: id,
                        from,
                        to: target,
                        reclaim_pressure: pressure,
                    });
                }
                Err(tab_manager::TransitionError::Protected { protection, .. }) => {
                    events.push(EvictionEvent::Blocked {
                        tab_id: id,
                        target,
                        protection,
                    });
                    managed.candidate = None;
                }
                Err(error) => {
                    managed.candidate = None;
                    debug_assert!(false, "policy produced illegal transition: {error}");
                }
            }
        }
        events
    }

    fn budget_pressure(&self) -> f64 {
        let Some(budget) = self.memory_budget else {
            return 0.0;
        };
        if budget <= 0.0 {
            return 1.0;
        }
        clamp((self.browser_working_set - budget) / budget)
    }
}

fn target_for(
    state: TabState,
    pressure: f64,
    budget_pressure: f64,
    config: PolicyConfig,
) -> Option<TabState> {
    match state {
        TabState::RecentlyActive if pressure >= 0.20 => Some(TabState::Background),
        TabState::Background if pressure >= config.discard_entry() && budget_pressure >= 0.25 => {
            Some(TabState::Discardable)
        }
        TabState::Background if pressure >= config.frozen_entry() => Some(TabState::Frozen),
        TabState::Frozen if pressure >= config.suspended_entry() => Some(TabState::Suspended),
        _ => None,
    }
}

fn dwell_for(state: TabState, target: TabState, config: PolicyConfig) -> u64 {
    match (state, target) {
        (TabState::RecentlyActive, TabState::Background) => {
            config.dwell(config.recently_active_dwell_secs)
        }
        (TabState::Background, TabState::Frozen) => config.dwell(config.frozen_dwell_secs),
        (TabState::Frozen, TabState::Suspended) => config.dwell(config.suspended_dwell_secs),
        (TabState::Background, TabState::Discardable) => {
            config.dwell(config.discardable_dwell_secs)
        }
        _ => 0,
    }
}

fn clamp(value: f64) -> f64 {
    if value.is_nan() {
        0.0
    } else {
        value.clamp(0.0, 1.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EvictionError {
    UnknownTab(u64),
    Transition(tab_manager::TransitionError),
}

impl std::fmt::Display for EvictionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownTab(id) => write!(f, "unknown tab {id}"),
            Self::Transition(error) => write!(f, "tab transition failed: {error}"),
        }
    }
}

impl std::error::Error for EvictionError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_is_bounded_and_budget_pressure_does_not_reclaim_active_tab() {
        let observation = TabObservation {
            activity: 2.0,
            foreground: 2.0,
            memory: 100.0,
            cpu: 100.0,
            ..TabObservation::default()
        };
        assert!((0.0..=1.0).contains(&observation.keep_resident_score()));
        assert!(observation.reclaim_pressure(1.0) < 1.0);
    }

    #[test]
    fn nan_inputs_are_safe() {
        let observation = TabObservation {
            memory: f64::NAN,
            cpu: f64::NAN,
            ..TabObservation::default()
        };
        assert!(observation.keep_resident_score().is_finite());
    }
}
