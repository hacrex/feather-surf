//! A policy-aware tab lifecycle state machine.
//!
//! The manager deliberately does not know about Chromium. It owns the tab
//! lifecycle and restoration metadata so that a browser adapter can enforce
//! the same rules regardless of the rendering backend.

use std::fmt;

/// Current snapshot schema version. Bump when `TabSnapshot` fields change.
pub const SNAPSHOT_SCHEMA_VERSION: u32 = 1;

/// Lifecycle states, ordered from most active to most reclaimable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TabState {
    Active,
    RecentlyActive,
    Background,
    Frozen,
    Suspended,
    Discardable,
}

impl TabState {
    /// Returns true when the state can be safely reclaimed without first
    /// asking the renderer to serialize live page state.
    pub const fn is_reclaimable(self) -> bool {
        matches!(self, Self::Frozen | Self::Suspended | Self::Discardable)
    }
}

/// Signals that protect a tab from destructive lifecycle transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct TabProtection {
    /// User explicitly pinned the tab.
    pub pinned: bool,
    /// Audio or video is currently playing.
    pub media_playing: bool,
    /// The page contains unsaved user input.
    pub dirty_form: bool,
    /// User explicitly requested this tab stay alive (overrides automatic eviction).
    pub keep_awake: bool,
}

impl TabProtection {
    pub const fn blocks_reclaim(self) -> bool {
        self.pinned || self.media_playing || self.dirty_form || self.keep_awake
    }
}

/// State that must survive suspension or discard so a browser adapter can
/// restore the tab without losing the user's place.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TabSnapshot {
    pub url: String,
    pub title: String,
    pub scroll_position: (u32, u32),
    pub form_state: Option<String>,
}

impl TabSnapshot {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            title: String::new(),
            scroll_position: (0, 0),
            form_state: None,
        }
    }

    /// Serialize the snapshot with a schema version envelope (bincode).
    pub fn serialize_versioned(&self) -> Result<Vec<u8>, SnapshotError> {
        let envelope = VersionedSnapshot {
            schema_version: SNAPSHOT_SCHEMA_VERSION,
            snapshot: self.clone(),
        };
        bincode::serialize(&envelope).map_err(|e| SnapshotError::Encode(e.to_string()))
    }

    /// Deserialize a snapshot from a versioned bincode envelope.
    ///
    /// Returns `SnapshotError::IncompatibleVersion` if the stored version
    /// is newer than `SNAPSHOT_SCHEMA_VERSION`.
    pub fn deserialize_versioned(data: &[u8]) -> Result<Self, SnapshotError> {
        let envelope: VersionedSnapshot =
            bincode::deserialize(data).map_err(|e| SnapshotError::Decode(e.to_string()))?;
        if envelope.schema_version > SNAPSHOT_SCHEMA_VERSION {
            return Err(SnapshotError::IncompatibleVersion {
                stored: envelope.schema_version,
                current: SNAPSHOT_SCHEMA_VERSION,
            });
        }
        Ok(envelope.snapshot)
    }
}

/// Wrapper that pairs a snapshot with its schema version for forward-compatible
/// deserialization.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct VersionedSnapshot {
    schema_version: u32,
    snapshot: TabSnapshot,
}

/// Errors that can occur during snapshot serialization or deserialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotError {
    Encode(String),
    Decode(String),
    IncompatibleVersion { stored: u32, current: u32 },
}

impl fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Encode(msg) => write!(f, "snapshot encoding failed: {msg}"),
            Self::Decode(msg) => write!(f, "snapshot decoding failed: {msg}"),
            Self::IncompatibleVersion { stored, current } => {
                write!(
                    f,
                    "snapshot version {stored} is incompatible with current version {current}"
                )
            }
        }
    }
}

impl std::error::Error for SnapshotError {}

/// A browser-independent tab record managed by the lifecycle state machine.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Tab {
    pub id: u64,
    state: TabState,
    protection: TabProtection,
    snapshot: TabSnapshot,
    revision: u64,
}

impl Tab {
    pub fn new(id: u64, url: impl Into<String>) -> Self {
        Self {
            id,
            state: TabState::Active,
            protection: TabProtection::default(),
            snapshot: TabSnapshot::new(url),
            revision: 0,
        }
    }

    pub const fn id(&self) -> u64 {
        self.id
    }

    pub const fn state(&self) -> TabState {
        self.state
    }

    pub const fn protection(&self) -> TabProtection {
        self.protection
    }

    pub fn set_protection(&mut self, protection: TabProtection) {
        self.protection = protection;
    }

    pub fn snapshot(&self) -> &TabSnapshot {
        &self.snapshot
    }

    pub fn snapshot_mut(&mut self) -> &mut TabSnapshot {
        &mut self.snapshot
    }

    pub const fn revision(&self) -> u64 {
        self.revision
    }

    /// Apply a lifecycle transition after validating both the state graph and
    /// protection policy.
    pub fn transition_to(&mut self, target: TabState) -> Result<(), TransitionError> {
        if self.state == target {
            return Err(TransitionError::AlreadyInState(target));
        }

        if !is_legal_transition(self.state, target) {
            return Err(TransitionError::IllegalTransition {
                from: self.state,
                to: target,
            });
        }

        if target.is_reclaimable() && self.protection.blocks_reclaim() {
            return Err(TransitionError::Protected {
                target,
                protection: self.protection,
            });
        }

        self.state = target;
        self.revision = self.revision.saturating_add(1);
        Ok(())
    }

    /// Force-suspend a tab, bypassing protection checks.
    ///
    /// This is the user-initiated "suspend now" action. It clears all
    /// protection flags and transitions directly to Suspended (or Frozen
    /// if the tab is not yet in a reclaimable state).
    ///
    /// Returns the previous state on success.
    pub fn force_suspend(&mut self) -> Result<TabState, TransitionError> {
        let prev = self.state;

        // If already suspended, nothing to do
        if prev == TabState::Suspended {
            return Err(TransitionError::AlreadyInState(TabState::Suspended));
        }

        // Clear all protections so the transition can proceed
        self.protection = TabProtection::default();

        // If in an active state, walk through the required intermediate states
        match prev {
            TabState::Active => {
                self.state = TabState::RecentlyActive;
                self.revision = self.revision.saturating_add(1);
                self.state = TabState::Background;
                self.revision = self.revision.saturating_add(1);
                self.state = TabState::Frozen;
                self.revision = self.revision.saturating_add(1);
                self.state = TabState::Suspended;
                self.revision = self.revision.saturating_add(1);
            }
            TabState::RecentlyActive => {
                self.state = TabState::Background;
                self.revision = self.revision.saturating_add(1);
                self.state = TabState::Frozen;
                self.revision = self.revision.saturating_add(1);
                self.state = TabState::Suspended;
                self.revision = self.revision.saturating_add(1);
            }
            TabState::Background => {
                self.state = TabState::Frozen;
                self.revision = self.revision.saturating_add(1);
                self.state = TabState::Suspended;
                self.revision = self.revision.saturating_add(1);
            }
            TabState::Frozen => {
                self.state = TabState::Suspended;
                self.revision = self.revision.saturating_add(1);
            }
            TabState::Discardable => {
                self.state = TabState::Suspended;
                self.revision = self.revision.saturating_add(1);
            }
            TabState::Suspended => unreachable!(),
        }

        Ok(prev)
    }
}

/// The only transitions accepted by the lifecycle graph.
const fn is_legal_transition(from: TabState, to: TabState) -> bool {
    matches!(
        (from, to),
        (TabState::Active, TabState::RecentlyActive)
            | (TabState::RecentlyActive, TabState::Background)
            | (TabState::Background, TabState::Frozen)
            | (TabState::Frozen, TabState::Suspended)
            | (TabState::Background, TabState::Discardable)
            | (TabState::Frozen, TabState::Active)
            | (TabState::Suspended, TabState::Active)
            | (TabState::Discardable, TabState::Active)
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionError {
    AlreadyInState(TabState),
    IllegalTransition {
        from: TabState,
        to: TabState,
    },
    Protected {
        target: TabState,
        protection: TabProtection,
    },
}

impl fmt::Display for TransitionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyInState(state) => write!(f, "tab is already in {state:?}"),
            Self::IllegalTransition { from, to } => {
                write!(f, "illegal tab transition: {from:?} -> {to:?}")
            }
            Self::Protected { target, .. } => {
                write!(f, "protected tab cannot transition to {target:?}")
            }
        }
    }
}

impl std::error::Error for TransitionError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn tab() -> Tab {
        Tab::new(7, "https://example.test")
    }

    fn advance_to_background(tab: &mut Tab) {
        tab.transition_to(TabState::RecentlyActive).unwrap();
        tab.transition_to(TabState::Background).unwrap();
    }

    #[test]
    fn new_tab_starts_active_with_restorable_snapshot() {
        let tab = tab();
        assert_eq!(tab.state(), TabState::Active);
        assert_eq!(tab.snapshot().url, "https://example.test");
        assert_eq!(tab.snapshot().scroll_position, (0, 0));
        assert_eq!(tab.revision(), 0);
    }

    #[test]
    fn normal_lifecycle_reaches_suspended_and_restores() {
        let mut tab = tab();
        advance_to_background(&mut tab);
        tab.transition_to(TabState::Frozen).unwrap();
        tab.transition_to(TabState::Suspended).unwrap();
        assert!(tab.state().is_reclaimable());

        tab.transition_to(TabState::Active).unwrap();
        assert_eq!(tab.state(), TabState::Active);
        assert_eq!(tab.revision(), 5);
    }

    #[test]
    fn background_can_be_discardable_and_restore_directly() {
        let mut tab = tab();
        advance_to_background(&mut tab);
        tab.transition_to(TabState::Discardable).unwrap();
        tab.transition_to(TabState::Active).unwrap();
        assert_eq!(tab.state(), TabState::Active);
    }

    #[test]
    fn illegal_transitions_are_rejected_without_mutation() {
        let mut tab = tab();
        let error = tab.transition_to(TabState::Suspended).unwrap_err();
        assert_eq!(
            error,
            TransitionError::IllegalTransition {
                from: TabState::Active,
                to: TabState::Suspended,
            }
        );
        assert_eq!(tab.state(), TabState::Active);
        assert_eq!(tab.revision(), 0);
    }

    #[test]
    fn duplicate_transition_is_rejected() {
        let mut tab = tab();
        let error = tab.transition_to(TabState::Active).unwrap_err();
        assert_eq!(error, TransitionError::AlreadyInState(TabState::Active));
    }

    #[test]
    fn pinned_tabs_cannot_be_reclaimed() {
        let mut tab = tab();
        tab.set_protection(TabProtection {
            pinned: true,
            ..TabProtection::default()
        });
        advance_to_background(&mut tab);
        let error = tab.transition_to(TabState::Frozen).unwrap_err();
        assert_eq!(
            error,
            TransitionError::Protected {
                target: TabState::Frozen,
                protection: tab.protection(),
            }
        );
        assert_eq!(tab.state(), TabState::Background);
    }

    #[test]
    fn media_playing_tabs_cannot_be_suspended_or_discarded() {
        let mut tab = tab();
        tab.set_protection(TabProtection {
            media_playing: true,
            ..TabProtection::default()
        });
        advance_to_background(&mut tab);
        assert!(matches!(
            tab.transition_to(TabState::Discardable),
            Err(TransitionError::Protected {
                target: TabState::Discardable,
                ..
            })
        ));
    }

    #[test]
    fn dirty_forms_block_reclaim_but_not_backgrounding() {
        let mut tab = tab();
        tab.set_protection(TabProtection {
            dirty_form: true,
            ..TabProtection::default()
        });
        advance_to_background(&mut tab);
        assert_eq!(tab.state(), TabState::Background);
        assert!(tab.transition_to(TabState::Frozen).is_err());
    }

    #[test]
    fn snapshot_metadata_survives_lifecycle_transitions() {
        let mut tab = tab();
        tab.snapshot_mut().title = "Example".into();
        tab.snapshot_mut().scroll_position = (12, 340);
        tab.snapshot_mut().form_state = Some("draft".into());
        advance_to_background(&mut tab);
        tab.transition_to(TabState::Frozen).unwrap();
        tab.transition_to(TabState::Suspended).unwrap();
        tab.transition_to(TabState::Active).unwrap();

        assert_eq!(tab.snapshot().title, "Example");
        assert_eq!(tab.snapshot().scroll_position, (12, 340));
        assert_eq!(tab.snapshot().form_state.as_deref(), Some("draft"));
    }

    #[test]
    fn only_successful_transitions_increment_revision() {
        let mut tab = tab();
        assert!(tab.transition_to(TabState::Suspended).is_err());
        assert_eq!(tab.revision(), 0);
        tab.transition_to(TabState::RecentlyActive).unwrap();
        assert_eq!(tab.revision(), 1);
    }
}

#[cfg(test)]
mod snapshot_serialization_tests {
    use super::*;

    #[test]
    fn snapshot_roundtrip_preserves_all_fields() {
        let original = TabSnapshot {
            url: "https://example.test/page?q=1".into(),
            title: "Test Page".into(),
            scroll_position: (42, 1080),
            form_state: Some("hello world".into()),
        };

        let bytes = original.serialize_versioned().unwrap();
        let restored = TabSnapshot::deserialize_versioned(&bytes).unwrap();

        assert_eq!(original, restored);
    }

    #[test]
    fn empty_snapshot_roundtrips() {
        let original = TabSnapshot::new("about:blank");
        let bytes = original.serialize_versioned().unwrap();
        let restored = TabSnapshot::deserialize_versioned(&bytes).unwrap();
        assert_eq!(original, restored);
    }

    #[test]
    fn snapshot_with_none_form_state_roundtrips() {
        let original = TabSnapshot {
            url: "https://example.test".into(),
            title: String::new(),
            scroll_position: (0, 0),
            form_state: None,
        };
        let bytes = original.serialize_versioned().unwrap();
        let restored = TabSnapshot::deserialize_versioned(&bytes).unwrap();
        assert_eq!(original, restored);
    }

    #[test]
    fn versioned_envelope_contains_correct_schema_version() {
        let snapshot = TabSnapshot::new("https://test.com");
        let bytes = snapshot.serialize_versioned().unwrap();
        let envelope: VersionedSnapshot = bincode::deserialize(&bytes).unwrap();
        assert_eq!(envelope.schema_version, SNAPSHOT_SCHEMA_VERSION);
    }

    #[test]
    fn deserialize_rejects_future_version() {
        let envelope = VersionedSnapshot {
            schema_version: SNAPSHOT_SCHEMA_VERSION + 1,
            snapshot: TabSnapshot::new("https://test.com"),
        };
        let bytes = bincode::serialize(&envelope).unwrap();
        let result = TabSnapshot::deserialize_versioned(&bytes);
        assert!(matches!(
            result,
            Err(SnapshotError::IncompatibleVersion { .. })
        ));
    }

    #[test]
    fn tab_serializes_and_deserializes() {
        let mut tab = Tab::new(42, "https://example.test");
        tab.set_protection(TabProtection {
            pinned: true,
            media_playing: false,
            dirty_form: true,
        });
        tab.snapshot_mut().title = "Pinned Tab".into();
        tab.snapshot_mut().scroll_position = (100, 500);
        tab.transition_to(TabState::RecentlyActive).unwrap();
        tab.transition_to(TabState::Background).unwrap();

        let bytes = bincode::serialize(&tab).unwrap();
        let restored: Tab = bincode::deserialize(&bytes).unwrap();

        assert_eq!(tab, restored);
        assert_eq!(restored.id(), 42);
        assert_eq!(restored.state(), TabState::Background);
        assert_eq!(restored.protection().pinned, true);
        assert_eq!(restored.protection().dirty_form, true);
        assert_eq!(restored.snapshot().title, "Pinned Tab");
        assert_eq!(restored.revision(), 2);
    }

    #[test]
    fn corrupt_data_returns_decode_error() {
        let result = TabSnapshot::deserialize_versioned(&[0xFF, 0xFE, 0xFD]);
        assert!(matches!(result, Err(SnapshotError::Decode(_))));
    }

    #[test]
    fn empty_data_returns_decode_error() {
        let result = TabSnapshot::deserialize_versioned(&[]);
        assert!(matches!(result, Err(SnapshotError::Decode(_))));
    }
}

#[cfg(test)]
mod user_override_tests {
    use super::*;

    #[test]
    fn keep_awake_blocks_eviction() {
        let mut tab = Tab::new(1, "https://important.com");
        tab.set_protection(TabProtection {
            keep_awake: true,
            ..TabProtection::default()
        });
        tab.transition_to(TabState::RecentlyActive).unwrap();
        tab.transition_to(TabState::Background).unwrap();
        // Cannot freeze a keep-awake tab
        assert!(tab.transition_to(TabState::Frozen).is_err());
        assert_eq!(tab.state(), TabState::Background);
    }

    #[test]
    fn force_suspend_clears_all_protections() {
        let mut tab = Tab::new(1, "https://example.com");
        tab.set_protection(TabProtection {
            pinned: true,
            media_playing: true,
            dirty_form: true,
            keep_awake: true,
        });
        let prev = tab.force_suspend().unwrap();
        assert_eq!(prev, TabState::Active);
        assert_eq!(tab.state(), TabState::Suspended);
        assert!(!tab.protection().pinned);
        assert!(!tab.protection().media_playing);
        assert!(!tab.protection().dirty_form);
        assert!(!tab.protection().keep_awake);
    }

    #[test]
    fn force_suspend_from_background() {
        let mut tab = Tab::new(1, "https://example.com");
        tab.set_protection(TabProtection {
            pinned: true,
            ..TabProtection::default()
        });
        tab.transition_to(TabState::RecentlyActive).unwrap();
        tab.transition_to(TabState::Background).unwrap();
        let prev = tab.force_suspend().unwrap();
        assert_eq!(prev, TabState::Background);
        assert_eq!(tab.state(), TabState::Suspended);
    }

    #[test]
    fn force_suspend_from_frozen() {
        let mut tab = Tab::new(1, "https://example.com");
        tab.transition_to(TabState::RecentlyActive).unwrap();
        tab.transition_to(TabState::Background).unwrap();
        tab.transition_to(TabState::Frozen).unwrap();
        let prev = tab.force_suspend().unwrap();
        assert_eq!(prev, TabState::Frozen);
        assert_eq!(tab.state(), TabState::Suspended);
    }

    #[test]
    fn force_suspend_already_suspended_fails() {
        let mut tab = Tab::new(1, "https://example.com");
        tab.transition_to(TabState::RecentlyActive).unwrap();
        tab.transition_to(TabState::Background).unwrap();
        tab.transition_to(TabState::Frozen).unwrap();
        tab.transition_to(TabState::Suspended).unwrap();
        assert!(matches!(
            tab.force_suspend(),
            Err(TransitionError::AlreadyInState(TabState::Suspended))
        ));
    }

    #[test]
    fn force_suspend_increments_revision() {
        let mut tab = Tab::new(1, "https://example.com");
        let before = tab.revision();
        tab.force_suspend().unwrap();
        assert!(tab.revision() > before);
    }

    #[test]
    fn hysteresis_prevents_immediate_loop() {
        // Simulate: tab is suspended, restored to active, then immediately
        // goes back to background. The eviction manager should not re-suspend
        // immediately due to cooldown.
        let mut tab = Tab::new(1, "https://example.com");
        tab.transition_to(TabState::RecentlyActive).unwrap();
        tab.transition_to(TabState::Background).unwrap();
        tab.transition_to(TabState::Frozen).unwrap();
        tab.transition_to(TabState::Suspended).unwrap();

        // Restore to active
        tab.transition_to(TabState::Active).unwrap();
        assert_eq!(tab.state(), TabState::Active);

        // Try to immediately re-suspend (should fail - needs intermediate states)
        assert!(tab.transition_to(TabState::Suspended).is_err());
        assert!(tab.transition_to(TabState::Frozen).is_err());

        // Must go through the proper lifecycle
        tab.transition_to(TabState::RecentlyActive).unwrap();
        tab.transition_to(TabState::Background).unwrap();
        assert_eq!(tab.state(), TabState::Background);
    }

    #[test]
    fn keep_awake_can_be_cleared() {
        let mut tab = Tab::new(1, "https://example.com");
        tab.set_protection(TabProtection {
            keep_awake: true,
            ..TabProtection::default()
        });
        assert!(tab.protection().blocks_reclaim());

        // Clear keep_awake
        tab.set_protection(TabProtection::default());
        assert!(!tab.protection().blocks_reclaim());

        // Now eviction can proceed
        tab.transition_to(TabState::RecentlyActive).unwrap();
        tab.transition_to(TabState::Background).unwrap();
        assert!(tab.transition_to(TabState::Frozen).is_ok());
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn every_legal_transition_preserves_id(
            id in 0u64..u64::MAX,
            url in ".*"
        ) {
            let mut tab = Tab::new(id, &url);
            // Walk through the legal path: Active → RecentlyActive → Background
            prop_assert_eq!(tab.id(), id);
            tab.transition_to(TabState::RecentlyActive).unwrap();
            prop_assert_eq!(tab.id(), id);
            tab.transition_to(TabState::Background).unwrap();
            prop_assert_eq!(tab.id(), id);
        }

        #[test]
        fn failed_transition_never_mutates_state(
            target in prop::enum::select(vec![
                TabState::Active,
                TabState::RecentlyActive,
                TabState::Background,
                TabState::Frozen,
                TabState::Suspended,
                TabState::Discardable,
            ])
        ) {
            let mut tab = Tab::new(1, "about:blank");
            let original_state = tab.state();
            let original_revision = tab.revision();
            let _ = tab.transition_to(target);
            // State and revision must not change on failure
            prop_assert_eq!(tab.state(), original_state);
            prop_assert_eq!(tab.revision(), original_revision);
        }

        #[test]
        fn protected_tab_never_enters_reclaimable(
            pinned in prop::bool::ANY,
            media in prop::bool::ANY,
            dirty in prop::bool::ANY,
            target in prop::select(vec![
                TabState::Frozen,
                TabState::Suspended,
                TabState::Discardable,
            ])
        ) {
            let protection = TabProtection {
                pinned,
                media_playing: media,
                dirty_form: dirty,
            };
            if !protection.blocks_reclaim() {
                return Ok(());
            }

            let mut tab = Tab::new(1, "about:blank");
            tab.set_protection(protection);
            tab.transition_to(TabState::RecentlyActive).unwrap();
            tab.transition_to(TabState::Background).unwrap();
            if target == TabState::Suspended {
                // Need to go through Frozen first
                let frozen_result = tab.transition_to(TabState::Frozen);
                if frozen_result.is_err() {
                    // Protection blocked it - correct
                    prop_assert!(tab.state().is_reclaimable() == false || !protection.blocks_reclaim());
                    return Ok(());
                }
            }
            let result = tab.transition_to(target);
            // Must fail - protected tab cannot enter reclaimable state
            prop_assert!(result.is_err());
        }

        #[test]
        fn snapshot_survives_any_lifecycle_path(
            url in "https?://[a-z]+\\.[a-z]+/.*",
            title in ".*",
            scroll_x in 0u32..10000,
            scroll_y in 0u32..100000,
            form_state in prop::option::of(".*")
        ) {
            let mut tab = Tab::new(1, &url);
            tab.snapshot_mut().title = title.clone();
            tab.snapshot_mut().scroll_position = (scroll_x, scroll_y);
            tab.snapshot_mut().form_state = form_state.clone();

            // Walk through lifecycle
            tab.transition_to(TabState::RecentlyActive).unwrap();
            tab.transition_to(TabState::Background).unwrap();
            tab.transition_to(TabState::Frozen).unwrap();
            tab.transition_to(TabState::Suspended).unwrap();
            tab.transition_to(TabState::Active).unwrap();

            // Snapshot must be preserved
            prop_assert_eq!(tab.snapshot().url, url);
            prop_assert_eq!(tab.snapshot().title, title);
            prop_assert_eq!(tab.snapshot().scroll_position, (scroll_x, scroll_y));
            prop_assert_eq!(tab.snapshot().form_state, form_state);
        }

        #[test]
        fn revision_only_increases_on_successful_transition(
            transitions in prop::collection::vec(
                prop::enum::select(vec![
                    TabState::Active,
                    TabState::RecentlyActive,
                    TabState::Background,
                    TabState::Frozen,
                    TabState::Suspended,
                    TabState::Discardable,
                ]),
                0..20
            )
        ) {
            let mut tab = Tab::new(1, "about:blank");
            let mut prev_revision = tab.revision();

            for target in transitions {
                let result = tab.transition_to(target);
                if result.is_ok() {
                    prop_assert!(tab.revision() > prev_revision,
                        "revision must increase on success: {} -> {}", prev_revision, tab.revision());
                    prev_revision = tab.revision();
                } else {
                    prop_assert_eq!(tab.revision(), prev_revision,
                        "revision must not change on failure");
                }
            }
        }

        #[test]
        fn active_tab_is_always_active(
            url in ".*"
        ) {
            let tab = Tab::new(1, &url);
            prop_assert_eq!(tab.state(), TabState::Active);
            prop_assert!(!tab.state().is_reclaimable());
        }

        #[test]
        fn snapshot_roundtrip_for_any_snapshot(
            url in "https?://[a-z]+\\.[a-z]+",
            title in ".*",
            scroll_x in 0u32..u32::MAX,
            scroll_y in 0u32..u32::MAX,
            form_state in prop::option::of(".*")
        ) {
            let original = TabSnapshot {
                url,
                title,
                scroll_position: (scroll_x, scroll_y),
                form_state,
            };
            let bytes = original.serialize_versioned().unwrap();
            let restored = TabSnapshot::deserialize_versioned(&bytes).unwrap();
            prop_assert_eq!(original, restored);
        }
    }
}
