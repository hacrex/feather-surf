//! A policy-aware tab lifecycle state machine.
//!
//! The manager deliberately does not know about Chromium. It owns the tab
//! lifecycle and restoration metadata so that a browser adapter can enforce
//! the same rules regardless of the rendering backend.

use std::fmt;

/// Lifecycle states, ordered from most active to most reclaimable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TabProtection {
    /// User explicitly pinned the tab.
    pub pinned: bool,
    /// Audio or video is currently playing.
    pub media_playing: bool,
    /// The page contains unsaved user input.
    pub dirty_form: bool,
}

impl TabProtection {
    pub const fn blocks_reclaim(self) -> bool {
        self.pinned || self.media_playing || self.dirty_form
    }
}

/// State that must survive suspension or discard so a browser adapter can
/// restore the tab without losing the user's place.
#[derive(Debug, Clone, PartialEq, Eq)]
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
}

/// A browser-independent tab record managed by the lifecycle state machine.
#[derive(Debug, Clone, PartialEq, Eq)]
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
    IllegalTransition { from: TabState, to: TabState },
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
    fn_media_playing_tabs_cannot_be_suspended_or_discarded() {
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
mod property_like_tests {
    use super::*;

    #[test]
    fn every_reclaimable_state_requires_unprotected_tab() {
        for target in [TabState::Frozen, TabState::Suspended, TabState::Discardable] {
            let mut tab = Tab::new(1, "about:blank");
            tab.set_protection(TabProtection {
                pinned: true,
                ..TabProtection::default()
            });
            tab.transition_to(TabState::RecentlyActive).unwrap();
            tab.transition_to(TabState::Background).unwrap();
            if target == TabState::Suspended {
                tab.transition_to(TabState::Frozen).unwrap();
            }
            assert!(tab.transition_to(target).is_err());
        }
    }
}
