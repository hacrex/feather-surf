use feathersurf_memory_manager::{
    EvictionEvent, EvictionManager, MemoryMode, PolicyConfig, TabObservation,
};
use tab_manager::{Tab, TabProtection, TabState};

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

#[test]
fn idle_tab_is_frozen_only_after_three_consecutive_samples() {
    let mut manager = EvictionManager::new(fast_config(MemoryMode::Balanced), None);
    manager.add_tab(
        background_tab(1, TabProtection::default()),
        TabObservation::idle(),
        0,
    );

    assert!(manager.tick(1).is_empty());
    assert!(manager.tick(2).is_empty());
    assert_eq!(
        transition_events(&manager.tick(3)),
        vec![(1, TabState::Background, TabState::Frozen)]
    );
    assert_eq!(manager.tab(1).unwrap().state(), TabState::Frozen);
}

#[test]
fn eviction_workflow_freezes_then_suspends_and_focus_restores() {
    let mut manager = EvictionManager::new(fast_config(MemoryMode::Balanced), None);
    manager.add_tab(
        background_tab(2, TabProtection::default()),
        TabObservation::idle(),
        0,
    );

    for now in 1..=3 {
        manager.tick(now);
    }
    assert_eq!(manager.tab(2).unwrap().state(), TabState::Frozen);

    for now in 4..=5 {
        assert!(manager.tick(now).is_empty());
    }
    assert_eq!(
        transition_events(&manager.tick(6)),
        vec![(2, TabState::Frozen, TabState::Suspended)]
    );

    manager.focus_tab(2, 100).unwrap();
    assert_eq!(manager.tab(2).unwrap().state(), TabState::Active);
    assert_eq!(
        manager.tab(2).unwrap().snapshot().url,
        "https://example.test/2"
    );
}

#[test]
fn threshold_hysteresis_resets_candidate_counter_when_pressure_drops() {
    let mut manager = EvictionManager::new(fast_config(MemoryMode::Balanced), None);
    manager.add_tab(
        background_tab(3, TabProtection::default()),
        TabObservation::idle(),
        0,
    );

    assert!(manager.tick(1).is_empty());
    manager.update_observation(
        3,
        TabObservation {
            activity: 1.0,
            foreground: 1.0,
            user_interaction: 1.0,
            network: 1.0,
            ..TabObservation::default()
        },
    );
    assert!(manager.tick(2).is_empty());

    manager.update_observation(3, TabObservation::idle());
    assert!(manager.tick(3).is_empty());
    assert!(manager.tick(4).is_empty());
    assert_eq!(
        transition_events(&manager.tick(5)),
        vec![(3, TabState::Background, TabState::Frozen)]
    );
}

#[test]
fn pinned_media_and_dirty_form_tabs_are_never_evicted() {
    let mut manager = EvictionManager::new(fast_config(MemoryMode::Balanced), None);
    for (id, protection) in [
        (
            10,
            TabProtection {
                pinned: true,
                ..TabProtection::default()
            },
        ),
        (
            11,
            TabProtection {
                media_playing: true,
                ..TabProtection::default()
            },
        ),
        (
            12,
            TabProtection {
                dirty_form: true,
                ..TabProtection::default()
            },
        ),
    ] {
        manager.add_tab(background_tab(id, protection), TabObservation::idle(), 0);
    }

    for now in 1..=10 {
        assert!(manager.tick(now).is_empty());
    }
    for id in [10, 11, 12] {
        assert_eq!(manager.tab(id).unwrap().state(), TabState::Background);
    }
}

#[test]
fn budget_pressure_increases_reclaim_pressure_for_inactive_tabs() {
    let observation = TabObservation {
        activity: 0.9,
        media: 0.9,
        user_interaction: 0.9,
        network: 1.0,
        ..TabObservation::default()
    };
    let without_budget = observation.reclaim_pressure(0.0);
    let with_budget = observation.reclaim_pressure(1.0);
    assert!(with_budget > without_budget);

    let mut manager = EvictionManager::new(fast_config(MemoryMode::Balanced), Some(100.0));
    manager.set_browser_working_set(120.0);
    manager.add_tab(background_tab(20, TabProtection::default()), observation, 0);
    for now in 1..=3 {
        manager.tick(now);
    }
    assert_eq!(manager.tab(20).unwrap().state(), TabState::Frozen);
}

#[test]
fn severe_budget_pressure_can_discard_background_tab_after_debounce() {
    let mut manager = EvictionManager::new(fast_config(MemoryMode::Balanced), Some(100.0));
    manager.set_browser_working_set(200.0);
    manager.add_tab(
        background_tab(21, TabProtection::default()),
        TabObservation::idle(),
        0,
    );

    for now in 1..=2 {
        assert!(manager.tick(now).is_empty());
    }
    assert_eq!(
        transition_events(&manager.tick(3)),
        vec![(21, TabState::Background, TabState::Discardable)]
    );
}

#[test]
fn memory_modes_change_threshold_behavior_without_changing_score() {
    let observation = TabObservation {
        activity: 0.9,
        foreground: 0.9,
        ..TabObservation::default()
    };
    assert!((observation.keep_resident_score() - 0.45).abs() < f64::EPSILON);

    let mut lite = EvictionManager::new(fast_config(MemoryMode::Lite), None);
    lite.add_tab(background_tab(30, TabProtection::default()), observation, 0);
    for now in 1..=3 {
        lite.tick(now);
    }
    assert_eq!(lite.tab(30).unwrap().state(), TabState::Frozen);

    let mut performance = EvictionManager::new(fast_config(MemoryMode::Performance), None);
    performance.add_tab(background_tab(31, TabProtection::default()), observation, 0);
    for now in 1..=3 {
        performance.tick(now);
    }
    assert_eq!(performance.tab(31).unwrap().state(), TabState::Background);
}

#[test]
fn unknown_tab_focus_is_reported_without_mutating_manager() {
    let mut manager = EvictionManager::new(fast_config(MemoryMode::Balanced), None);
    assert!(manager.focus_tab(999, 0).is_err());
}

#[test]
fn active_tab_is_never_evicted_by_policy_ticks() {
    let mut manager = EvictionManager::new(fast_config(MemoryMode::Lite), Some(100.0));
    manager.set_browser_working_set(1_000.0);
    manager.add_tab(
        Tab::new(40, "https://example.test/active"),
        TabObservation::idle(),
        0,
    );

    for now in 1..=20 {
        assert!(manager.tick(now).is_empty());
    }
    assert_eq!(manager.tab(40).unwrap().state(), TabState::Active);
}

#[test]
fn recently_active_tab_enters_background_after_dwell_and_debounce() {
    let config = PolicyConfig {
        recently_active_dwell_secs: 10,
        frozen_dwell_secs: 0,
        suspended_dwell_secs: 0,
        discardable_dwell_secs: 0,
        ..fast_config(MemoryMode::Balanced)
    };
    let mut manager = EvictionManager::new(config, None);
    manager.add_tab(
        Tab::new(41, "https://example.test/recent"),
        TabObservation::idle(),
        0,
    );
    assert_eq!(manager.tab(41).unwrap().state(), TabState::Active);
    manager
        .tab_mut(41)
        .unwrap()
        .transition_to(TabState::RecentlyActive)
        .unwrap();

    for now in 1..=12 {
        let events = manager.tick(now);
        if now < 12 {
            assert!(events.is_empty());
        } else {
            assert_eq!(
                transition_events(&events),
                vec![(41, TabState::RecentlyActive, TabState::Background)]
            );
        }
    }
    assert_eq!(manager.tab(41).unwrap().state(), TabState::Background);
}

#[test]
fn discard_requires_budget_pressure_and_is_not_normal_idle_behavior() {
    let mut manager = EvictionManager::new(fast_config(MemoryMode::Balanced), None);
    manager.add_tab(
        background_tab(42, TabProtection::default()),
        TabObservation::idle(),
        0,
    );
    for now in 1..=3 {
        manager.tick(now);
    }
    assert_eq!(manager.tab(42).unwrap().state(), TabState::Frozen);
}
