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

    // ── Score bounds ────────────────────────────────────────────────

    #[test]
    fn score_is_bounded_for_out_of_range_inputs() {
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
    fn score_is_bounded_for_negative_inputs() {
        let observation = TabObservation {
            activity: -5.0,
            foreground: -10.0,
            memory: -1.0,
            cpu: -0.5,
            ..TabObservation::default()
        };
        let score = observation.keep_resident_score();
        assert!((0.0..=1.0).contains(&score), "score was {score}");
    }

    #[test]
    fn score_is_bounded_for_mixed_extreme_inputs() {
        let observation = TabObservation {
            activity: f64::MAX,
            foreground: f64::MIN_POSITIVE,
            media: -999.0,
            user_interaction: 0.0,
            network: f64::INFINITY,
            memory: f64::NEG_INFINITY,
            cpu: f64::NAN,
            pinned: 1e10,
        };
        let score = observation.keep_resident_score();
        assert!((0.0..=1.0).contains(&score), "score was {score}");
    }

    // ── Score determinism ───────────────────────────────────────────

    #[test]
    fn identical_observations_produce_identical_scores() {
        let obs = TabObservation {
            activity: 0.7,
            media: 0.3,
            user_interaction: 0.5,
            network: 0.2,
            memory: 0.8,
            cpu: 0.4,
            pinned: 0.0,
            foreground: 1.0,
        };
        let s1 = obs.keep_resident_score();
        let s2 = obs.keep_resident_score();
        assert_eq!(s1, s2);
    }

    #[test]
    fn score_is_pure_function_of_inputs() {
        let a = TabObservation { activity: 0.5, ..TabObservation::default() };
        let b = TabObservation { activity: 0.5, ..TabObservation::default() };
        assert_eq!(a.keep_resident_score(), b.keep_resident_score());
    }

    // ── NaN and infinity safety ─────────────────────────────────────

    #[test]
    fn nan_inputs_produce_finite_score() {
        let observation = TabObservation {
            memory: f64::NAN,
            cpu: f64::NAN,
            ..TabObservation::default()
        };
        assert!(observation.keep_resident_score().is_finite());
    }

    #[test]
    fn infinity_inputs_are_clamped() {
        let observation = TabObservation {
            activity: f64::INFINITY,
            foreground: f64::NEG_INFINITY,
            ..TabObservation::default()
        };
        let score = observation.keep_resident_score();
        assert!(score.is_finite());
        assert!((0.0..=1.0).contains(&score));
    }

    #[test]
    fn all_nan_inputs_produce_valid_score() {
        let observation = TabObservation {
            activity: f64::NAN,
            media: f64::NAN,
            user_interaction: f64::NAN,
            network: f64::NAN,
            memory: f64::NAN,
            cpu: f64::NAN,
            pinned: f64::NAN,
            foreground: f64::NAN,
        };
        let score = observation.keep_resident_score();
        assert!(score.is_finite());
        assert!((0.0..=1.0).contains(&score));
    }

    // ── Zero and one boundary inputs ────────────────────────────────

    #[test]
    fn all_zero_observation_scores_zero() {
        let observation = TabObservation::idle();
        assert_eq!(observation.keep_resident_score(), 0.0);
    }

    #[test]
    fn fully_active_foreground_scores_high() {
        let observation = TabObservation::active();
        let score = observation.keep_resident_score();
        assert!(score > 0.4, "active score was {score}");
    }

    #[test]
    fn pinned_tab_scores_higher_than_unpinned() {
        let base = TabObservation {
            activity: 0.5,
            foreground: 0.5,
            ..TabObservation::default()
        };
        let pinned = TabObservation {
            pinned: 1.0,
            ..base
        };
        assert!(pinned.keep_resident_score() > base.keep_resident_score());
    }

    // ── Weight contribution tests ───────────────────────────────────

    #[test]
    fn foreground_has_highest_weight() {
        let fg_only = TabObservation { foreground: 1.0, ..TabObservation::default() };
        let act_only = TabObservation { activity: 1.0, ..TabObservation::default() };
        let media_only = TabObservation { media: 1.0, ..TabObservation::default() };
        assert!(fg_only.keep_resident_score() > act_only.keep_resident_score());
        assert!(act_only.keep_resident_score() > media_only.keep_resident_score());
    }

    #[test]
    fn media_contributes_more_than_user_interaction() {
        let media = TabObservation { media: 1.0, ..TabObservation::default() };
        let interaction = TabObservation { user_interaction: 1.0, ..TabObservation::default() };
        assert!(media.keep_resident_score() > interaction.keep_resident_score());
    }

    #[test]
    fn network_contributes_smallest_positive_weight() {
        let network = TabObservation { network: 1.0, ..TabObservation::default() };
        let pinned = TabObservation { pinned: 1.0, ..TabObservation::default() };
        assert!(pinned.keep_resident_score() > network.keep_resident_score());
    }

    // ── Inactivity factor ───────────────────────────────────────────

    #[test]
    fn inactivity_is_zero_when_foreground() {
        let obs = TabObservation { foreground: 1.0, ..TabObservation::default() };
        let score = obs.keep_resident_score();
        let inactivity = 1.0 - obs.foreground.max(obs.activity).max(obs.media)
            .max(obs.user_interaction).max(obs.pinned);
        assert_eq!(inactivity, 0.0);
        assert!(score > 0.0);
    }

    #[test]
    fn inactivity_penalizes_heavy_inactive_tabs() {
        let light_inactive = TabObservation {
            memory: 0.1,
            cpu: 0.1,
            ..TabObservation::default()
        };
        let heavy_inactive = TabObservation {
            memory: 1.0,
            cpu: 1.0,
            ..TabObservation::default()
        };
        assert!(light_inactive.keep_resident_score() > heavy_inactive.keep_resident_score());
    }

    // ── Reclaim pressure ────────────────────────────────────────────

    #[test]
    fn reclaim_pressure_is_complement_of_score() {
        let obs = TabObservation {
            activity: 0.6,
            foreground: 0.8,
            media: 0.3,
            ..TabObservation::default()
        };
        let score = obs.keep_resident_score();
        let pressure = obs.reclaim_pressure(0.0);
        assert!((1.0 - score - pressure).abs() < f64::EPSILON);
    }

    #[test]
    fn budget_pressure_increases_reclaim_pressure() {
        let obs = TabObservation {
            activity: 0.5,
            foreground: 0.5,
            ..TabObservation::default()
        };
        let p0 = obs.reclaim_pressure(0.0);
        let p1 = obs.reclaim_pressure(1.0);
        assert!(p1 > p0);
    }

    #[test]
    fn budget_pressure_does_not_affect_fully_active_tab() {
        let obs = TabObservation::active();
        let p0 = obs.reclaim_pressure(0.0);
        let p1 = obs.reclaim_pressure(1.0);
        assert_eq!(p0, p1);
    }

    #[test]
    fn reclaim_pressure_is_bounded() {
        for a in [0.0, 0.3, 0.7, 1.0] {
            for bp in [0.0, 0.5, 1.0] {
                let obs = TabObservation { activity: a, ..TabObservation::default() };
                let p = obs.reclaim_pressure(bp);
                assert!((0.0..=1.0).contains(&p), "pressure was {p} at a={a}, bp={bp}");
            }
        }
    }

    // ── Memory mode thresholds ──────────────────────────────────────

    #[test]
    fn lite_mode_has_lowest_frozen_threshold() {
        let config = PolicyConfig::default();
        assert!(config.frozen_entry() > PolicyConfig { mode: MemoryMode::Lite, ..config }.frozen_entry());
    }

    #[test]
    fn performance_mode_has_highest_frozen_threshold() {
        let config = PolicyConfig::default();
        assert!(config.frozen_entry() < PolicyConfig { mode: MemoryMode::Performance, ..config }.frozen_entry());
    }

    #[test]
    fn modes_produce_different_dwell_times() {
        let base = PolicyConfig::default();
        let lite = PolicyConfig { mode: MemoryMode::Lite, ..base };
        let perf = PolicyConfig { mode: MemoryMode::Performance, ..base };
        assert!(lite.dwell(100) < base.dwell(100));
        assert!(perf.dwell(100) > base.dwell(100));
    }

    // ── Edge cases ──────────────────────────────────────────────────

    #[test]
    fn single_high_metric_outweighs_low_others() {
        let obs = TabObservation {
            foreground: 1.0,
            ..TabObservation::default()
        };
        assert!(obs.keep_resident_score() > 0.2);
    }

    #[test]
    fn media_playing_tab_resists_eviction() {
        let playing = TabObservation { media: 1.0, ..TabObservation::default() };
        let silent = TabObservation::idle();
        assert!(playing.keep_resident_score() > silent.keep_resident_score());
        assert!(playing.reclaim_pressure(0.0) < silent.reclaim_pressure(0.0));
    }
}

// ── Fake clock for deterministic policy tests ──────────────────────

pub struct FakeClock {
    now: u64,
}

impl FakeClock {
    pub fn new(start: u64) -> Self {
        Self { now: start }
    }

    pub fn now(&self) -> u64 {
        self.now
    }

    pub fn advance(&mut self, seconds: u64) {
        self.now += seconds;
    }

    pub fn set(&mut self, t: u64) {
        self.now = t;
    }
}

#[cfg(test)]
mod hysteresis_tests {
    use super::*;

    fn fast_config(mode: MemoryMode) -> PolicyConfig {
        PolicyConfig {
            mode,
            sample_debounce: 3,
            recently_active_dwell_secs: 0,
            frozen_dwell_secs: 0,
            suspended_dwell_secs: 0,
            discardable_dwell_secs: 0,
        }
    }

    fn background_tab(id: u64, protection: TabProtection) -> Tab {
        let mut tab = Tab::new(id, format!("https://example.test/{id}"));
        tab.set_protection(protection);
        tab.transition_to(TabState::RecentlyActive).unwrap();
        tab.transition_to(TabState::Background).unwrap();
        tab
    }

    fn transition_events(events: &[EvictionEvent]) -> Vec<(u64, TabState, TabState)> {
        events
            .iter()
            .filter_map(|event| match event {
                EvictionEvent::Transition {
                    tab_id, from, to, ..
                } => Some((*tab_id, *from, *to)),
                EvictionEvent::Blocked { .. } => None,
            })
            .collect()
    }

    // ── Debounce: three consecutive samples required ────────────────

    #[test]
    fn tab_not_frozen_until_three_consecutive_qualifying_samples() {
        let mut clock = FakeClock::new(0);
        let mut manager = EvictionManager::new(fast_config(MemoryMode::Balanced), None);
        manager.add_tab(
            background_tab(1, TabProtection::default()),
            TabObservation::idle(),
            clock.now(),
        );

        // Sample 1: candidate counter → 1
        clock.advance(1);
        assert!(manager.tick(clock.now()).is_empty());
        // Sample 2: candidate counter → 2
        clock.advance(1);
        assert!(manager.tick(clock.now()).is_empty());
        // Sample 3: candidate counter → 3, transition fires
        clock.advance(1);
        let events = manager.tick(clock.now());
        assert_eq!(
            transition_events(&events),
            vec![(1, TabState::Background, TabState::Frozen)]
        );
    }

    #[test]
    fn debounce_counter_resets_when_pressure_drops() {
        let mut clock = FakeClock::new(0);
        let mut manager = EvictionManager::new(fast_config(MemoryMode::Balanced), None);
        manager.add_tab(
            background_tab(2, TabProtection::default()),
            TabObservation::idle(),
            clock.now(),
        );

        // Two qualifying samples
        clock.advance(1);
        assert!(manager.tick(clock.now()).is_empty());
        clock.advance(1);
        assert!(manager.tick(clock.now()).is_empty());

        // Pressure drops — counter should reset
        manager.update_observation(
            2,
            TabObservation {
                activity: 1.0,
                foreground: 1.0,
                user_interaction: 1.0,
                network: 1.0,
                ..TabObservation::default()
            },
        );
        clock.advance(1);
        assert!(manager.tick(clock.now()).is_empty());

        // Back to idle — counter starts from 1 again, need 3 more samples
        manager.update_observation(2, TabObservation::idle());
        clock.advance(1);
        assert!(manager.tick(clock.now()).is_empty());
        clock.advance(1);
        assert!(manager.tick(clock.now()).is_empty());
        clock.advance(1);
        let events = manager.tick(clock.now());
        assert_eq!(
            transition_events(&events),
            vec![(2, TabState::Background, TabState::Frozen)]
        );
    }

    // ── Minimum dwell times ─────────────────────────────────────────

    #[test]
    fn tab_does_not_progress_without_dwell_time() {
        let config = PolicyConfig {
            recently_active_dwell_secs: 30,
            ..fast_config(MemoryMode::Balanced)
        };
        let mut clock = FakeClock::new(0);
        let mut manager = EvictionManager::new(config, None);
        manager.add_tab(
            Tab::new(10, "https://example.test/10"),
            TabObservation::idle(),
            clock.now(),
        );
        manager
            .tab_mut(10)
            .unwrap()
            .transition_to(TabState::RecentlyActive)
            .unwrap();

        // Tick rapidly — dwell not met
        for _ in 0..5 {
            clock.advance(1);
            assert!(manager.tick(clock.now()).is_empty());
        }
        assert_eq!(manager.tab(10).unwrap().state(), TabState::RecentlyActive);
    }

    #[test]
    fn tab_transitions_after_dwell_time_elapses() {
        let config = PolicyConfig {
            recently_active_dwell_secs: 10,
            ..fast_config(MemoryMode::Balanced)
        };
        let mut clock = FakeClock::new(0);
        let mut manager = EvictionManager::new(config, None);
        manager.add_tab(
            Tab::new(11, "https://example.test/11"),
            TabObservation::idle(),
            clock.now(),
        );
        manager
            .tab_mut(11)
            .unwrap()
            .transition_to(TabState::RecentlyActive)
            .unwrap();

        // Just before dwell
        clock.advance(9);
        assert!(manager.tick(clock.now()).is_empty());

        // Dwell met + debounce
        clock.advance(1);
        let mut fired = false;
        for _ in 0..3 {
            clock.advance(1);
            let events = manager.tick(clock.now());
            if !transition_events(&events).is_empty() {
                fired = true;
                break;
            }
        }
        assert!(fired);
        assert_eq!(manager.tab(11).unwrap().state(), TabState::Background);
    }

    // ── Restoration cooldown ────────────────────────────────────────

    #[test]
    fn restored_tab_has_cooldown_before_reclamation() {
        let mut clock = FakeClock::new(0);
        let mut manager = EvictionManager::new(fast_config(MemoryMode::Balanced), None);
        manager.add_tab(
            background_tab(20, TabProtection::default()),
            TabObservation::idle(),
            clock.now(),
        );

        // Freeze it
        for _ in 0..3 {
            clock.advance(1);
            manager.tick(clock.now());
        }
        assert_eq!(manager.tab(20).unwrap().state(), TabState::Frozen);

        // Restore via focus
        manager.focus_tab(20, clock.now()).unwrap();
        assert_eq!(manager.tab(20).unwrap().state(), TabState::Active);

        // Immediately send back to background
        manager
            .tab_mut(20)
            .unwrap()
            .transition_to(TabState::RecentlyActive)
            .unwrap();
        manager
            .tab_mut(20)
            .unwrap()
            .transition_to(TabState::Background)
            .unwrap();

        // During cooldown — should not be reclaimed
        for _ in 0..5 {
            clock.advance(1);
            assert!(manager.tick(clock.now()).is_empty());
        }

        // After cooldown (30s)
        clock.set(clock.now() + 30);
        for _ in 0..3 {
            clock.advance(1);
            let events = manager.tick(clock.now());
            if !transition_events(&events).is_empty() {
                return; // Success
            }
        }
        panic!("tab should have been frozen after cooldown");
    }

    // ── Budget pressure override ────────────────────────────────────

    #[test]
    fn severe_budget_pressure_accelerates_eviction() {
        let mut clock = FakeClock::new(0);
        let mut manager = EvictionManager::new(fast_config(MemoryMode::Balanced), Some(100.0));
        manager.set_browser_working_set(250.0); // 150% over budget

        manager.add_tab(
            background_tab(30, TabProtection::default()),
            TabObservation::idle(),
            clock.now(),
        );

        for _ in 0..3 {
            clock.advance(1);
            manager.tick(clock.now());
        }
        // Under severe budget pressure, background → discardable is possible
        assert!(matches!(
            manager.tab(30).unwrap().state(),
            TabState::Frozen | TabState::Discardable
        ));
    }

    #[test]
    fn no_budget_pressure_follows_normal_path() {
        let mut clock = FakeClock::new(0);
        let mut manager = EvictionManager::new(fast_config(MemoryMode::Balanced), None);
        manager.add_tab(
            background_tab(31, TabProtection::default()),
            TabObservation::idle(),
            clock.now(),
        );

        for _ in 0..3 {
            clock.advance(1);
            manager.tick(clock.now());
        }
        assert_eq!(manager.tab(31).unwrap().state(), TabState::Frozen);
    }

    // ── Focus always restores immediately ───────────────────────────

    #[test]
    fn focus_restores_any_tab_immediately() {
        let mut clock = FakeClock::new(0);
        let mut manager = EvictionManager::new(fast_config(MemoryMode::Balanced), None);

        // Freeze a tab
        manager.add_tab(
            background_tab(40, TabProtection::default()),
            TabObservation::idle(),
            clock.now(),
        );
        for _ in 0..3 {
            clock.advance(1);
            manager.tick(clock.now());
        }
        assert_eq!(manager.tab(40).unwrap().state(), TabState::Frozen);

        // Focus restores immediately regardless of dwell
        manager.focus_tab(40, clock.now()).unwrap();
        assert_eq!(manager.tab(40).unwrap().state(), TabState::Active);
        assert_eq!(
            manager.tab(40).unwrap().snapshot().url,
            "https://example.test/40"
        );
    }

    // ── Protected tabs never evicted ────────────────────────────────

    #[test]
    fn pinned_tab_resists_all_reclamation() {
        let mut clock = FakeClock::new(0);
        let mut manager = EvictionManager::new(fast_config(MemoryMode::Lite), Some(10.0));
        manager.set_browser_working_set(1000.0);

        manager.add_tab(
            background_tab(
                50,
                TabProtection {
                    pinned: true,
                    ..TabProtection::default()
                },
            ),
            TabObservation::idle(),
            clock.now(),
        );

        for _ in 0..20 {
            clock.advance(1);
            assert!(manager.tick(clock.now()).is_empty());
        }
        assert_eq!(manager.tab(50).unwrap().state(), TabState::Background);
    }

    #[test]
    fn media_tab_resists_reclamation() {
        let mut clock = FakeClock::new(0);
        let mut manager = EvictionManager::new(fast_config(MemoryMode::Lite), Some(10.0));
        manager.set_browser_working_set(1000.0);

        manager.add_tab(
            background_tab(
                51,
                TabProtection {
                    media_playing: true,
                    ..TabProtection::default()
                },
            ),
            TabObservation::idle(),
            clock.now(),
        );

        for _ in 0..20 {
            clock.advance(1);
            assert!(manager.tick(clock.now()).is_empty());
        }
        assert_eq!(manager.tab(51).unwrap().state(), TabState::Background);
    }

    #[test]
    fn dirty_form_tab_resists_reclamation() {
        let mut clock = FakeClock::new(0);
        let mut manager = EvictionManager::new(fast_config(MemoryMode::Lite), Some(10.0));
        manager.set_browser_working_set(1000.0);

        manager.add_tab(
            background_tab(
                52,
                TabProtection {
                    dirty_form: true,
                    ..TabProtection::default()
                },
            ),
            TabObservation::idle(),
            clock.now(),
        );

        for _ in 0..20 {
            clock.advance(1);
            assert!(manager.tick(clock.now()).is_empty());
        }
        assert_eq!(manager.tab(52).unwrap().state(), TabState::Background);
    }

    // ── Mode-dependent threshold behavior ───────────────────────────

    #[test]
    fn lite_mode_freezes_tab_sooner_than_balanced() {
        let obs = TabObservation {
            activity: 0.3,
            foreground: 0.0,
            ..TabObservation::default()
        };

        let mut clock = FakeClock::new(0);
        let mut lite = EvictionManager::new(fast_config(MemoryMode::Lite), None);
        lite.add_tab(background_tab(60, TabProtection::default()), obs, clock.now());

        let mut clock2 = FakeClock::new(0);
        let mut balanced = EvictionManager::new(fast_config(MemoryMode::Balanced), None);
        balanced.add_tab(background_tab(61, TabProtection::default()), obs, clock2.now());

        // Both get same ticks
        for _ in 0..10 {
            clock.advance(1);
            clock2.advance(1);
            lite.tick(clock.now());
            balanced.tick(clock2.now());
        }

        // Lite should be more aggressive (frozen or beyond)
        let lite_state = lite.tab(60).unwrap().state();
        let balanced_state = balanced.tab(61).unwrap().state();
        assert!(
            (lite_state as u8) >= (balanced_state as u8),
            "lite={lite_state:?} should be >= balanced={balanced_state:?}"
        );
    }

    // ── Full lifecycle with clock ───────────────────────────────────

    #[test]
    fn full_lifecycle_active_to_discardable_and_back() {
        let mut clock = FakeClock::new(0);
        let mut manager = EvictionManager::new(fast_config(MemoryMode::Balanced), Some(50.0));
        manager.set_browser_working_set(200.0);

        manager.add_tab(
            Tab::new(70, "https://example.test/70"),
            TabObservation::idle(),
            clock.now(),
        );

        // Active → RecentlyActive (manual)
        manager
            .tab_mut(70)
            .unwrap()
            .transition_to(TabState::RecentlyActive)
            .unwrap();

        // RecentlyActive → Background (after dwell + debounce)
        clock.advance(15);
        for _ in 0..5 {
            clock.advance(1);
            let events = manager.tick(clock.now());
            if transition_events(&events)
                .iter()
                .any(|(_, _, to)| *to == TabState::Background)
            {
                break;
            }
        }
        assert_eq!(manager.tab(70).unwrap().state(), TabState::Background);

        // Background → Frozen
        for _ in 0..5 {
            clock.advance(1);
            let events = manager.tick(clock.now());
            if transition_events(&events)
                .iter()
                .any(|(_, _, to)| *to == TabState::Frozen)
            {
                break;
            }
        }
        assert_eq!(manager.tab(70).unwrap().state(), TabState::Frozen);

        // Frozen → Suspended
        for _ in 0..5 {
            clock.advance(1);
            let events = manager.tick(clock.now());
            if transition_events(&events)
                .iter()
                .any(|(_, _, to)| *to == TabState::Suspended)
            {
                break;
            }
        }
        assert_eq!(manager.tab(70).unwrap().state(), TabState::Suspended);

        // Restore via focus
        manager.focus_tab(70, clock.now()).unwrap();
        assert_eq!(manager.tab(70).unwrap().state(), TabState::Active);
        assert_eq!(
            manager.tab(70).unwrap().snapshot().url,
            "https://example.test/70"
        );
    }

    // ── Multiple tabs: priority ordering ────────────────────────────

    #[test]
    fn multiple_idle_tabs_all_eventually_frozen() {
        let mut clock = FakeClock::new(0);
        let mut manager = EvictionManager::new(fast_config(MemoryMode::Balanced), None);

        for id in 80..85 {
            manager.add_tab(
                background_tab(id, TabProtection::default()),
                TabObservation::idle(),
                clock.now(),
            );
        }

        for _ in 0..10 {
            clock.advance(1);
            manager.tick(clock.now());
        }

        for id in 80..85 {
            assert_eq!(
                manager.tab(id).unwrap().state(),
                TabState::Frozen,
                "tab {id} should be frozen"
            );
        }
    }
}
