// FeatherSurf Profiles
//
// Profile isolation and container management.
// Each profile has its own cookies, storage, history, bookmarks, and settings.

use std::collections::HashMap;
use std::time::SystemTime;

use crate::storage::{BookmarkStore, HistoryStore};

// ── Profile ────────────────────────────────────────────────────────

/// Profile isolation level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum IsolationLevel {
    /// Default profile: shared cookies and storage within the profile.
    Default,
    /// Container: separate cookies and storage, but shares profile directory.
    Container,
    /// Temporary: data cleared on close.
    Ephemeral,
}

/// A browser profile with isolated data.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Profile {
    pub id: u64,
    pub name: String,
    pub isolation: IsolationLevel,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub created_at: u64,
    pub is_active: bool,

    // Data stores (not serialized for performance, loaded on demand)
    #[serde(skip)]
    pub bookmarks: BookmarkStore,
    #[serde(skip)]
    pub history: HistoryStore,

    // Settings
    pub homepage: Option<String>,
    pub search_engine: Option<String>,
    pub block_trackers: bool,
    pub https_only: bool,
    pub clear_on_close: bool,
}

impl Profile {
    pub fn new(id: u64, name: impl Into<String>, isolation: IsolationLevel) -> Self {
        Self {
            id,
            name: name.into(),
            isolation,
            color: None,
            icon: None,
            created_at: now_secs(),
            is_active: false,
            bookmarks: BookmarkStore::new(),
            history: HistoryStore::new(),
            homepage: None,
            search_engine: None,
            block_trackers: true,
            https_only: true,
            clear_on_close: false,
        }
    }

    /// Create a temporary profile that clears data on close.
    pub fn ephemeral(id: u64, name: impl Into<String>) -> Self {
        Self::new(id, name, IsolationLevel::Ephemeral)
    }

    /// Create a container profile.
    pub fn container(id: u64, name: impl Into<String>) -> Self {
        Self::new(id, name, IsolationLevel::Container)
    }

    /// Clear all browsing data for this profile.
    pub fn clear_data(&mut self) {
        self.bookmarks.clear();
        self.history.clear();
    }
}

// ── Profile Manager ────────────────────────────────────────────────

/// Manages browser profiles.
pub struct ProfileManager {
    profiles: HashMap<u64, Profile>,
    next_id: u64,
    active_profile_id: Option<u64>,
}

impl Default for ProfileManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ProfileManager {
    pub fn new() -> Self {
        Self {
            profiles: HashMap::new(),
            next_id: 0,
            active_profile_id: None,
        }
    }

    /// Create a new profile. Returns the profile ID.
    pub fn create_profile(
        &mut self,
        name: impl Into<String>,
        isolation: IsolationLevel,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let mut profile = Profile::new(id, name, isolation);
        profile.is_active = self.profiles.is_empty(); // First profile is active
        if profile.is_active {
            self.active_profile_id = Some(id);
        }
        self.profiles.insert(id, profile);
        id
    }

    /// Create an ephemeral profile.
    pub fn create_ephemeral(&mut self, name: impl Into<String>) -> u64 {
        self.create_profile(name, IsolationLevel::Ephemeral)
    }

    /// Create a container profile.
    pub fn create_container(&mut self, name: impl Into<String>) -> u64 {
        self.create_profile(name, IsolationLevel::Container)
    }

    /// Remove a profile and all its data.
    pub fn remove_profile(&mut self, id: u64) -> bool {
        if self.profiles.remove(&id).is_some() {
            if self.active_profile_id == Some(id) {
                self.active_profile_id = self.profiles.keys().next().copied();
                if let Some(new_id) = self.active_profile_id {
                    if let Some(p) = self.profiles.get_mut(&new_id) {
                        p.is_active = true;
                    }
                }
            }
            true
        } else {
            false
        }
    }

    /// Set the active profile.
    pub fn set_active(&mut self, id: u64) -> bool {
        if !self.profiles.contains_key(&id) {
            return false;
        }
        // Deactivate all
        for p in self.profiles.values_mut() {
            p.is_active = false;
        }
        // Activate target
        if let Some(p) = self.profiles.get_mut(&id) {
            p.is_active = true;
            self.active_profile_id = Some(id);
            true
        } else {
            false
        }
    }

    /// Get the active profile.
    pub fn active_profile(&self) -> Option<&Profile> {
        self.active_profile_id
            .and_then(|id| self.profiles.get(&id))
    }

    /// Get a profile by ID.
    pub fn get_profile(&self, id: u64) -> Option<&Profile> {
        self.profiles.get(&id)
    }

    /// Get a mutable reference to a profile.
    pub fn get_profile_mut(&mut self, id: u64) -> Option<&mut Profile> {
        self.profiles.get_mut(&id)
    }

    /// Get all profiles.
    pub fn all_profiles(&self) -> Vec<&Profile> {
        self.profiles.values().collect()
    }

    /// Get profiles by isolation level.
    pub fn profiles_with_level(&self, level: IsolationLevel) -> Vec<&Profile> {
        self.profiles
            .values()
            .filter(|p| p.isolation == level)
            .collect()
    }

    /// Get profile count.
    pub fn profile_count(&self) -> usize {
        self.profiles.len()
    }

    /// Check if a profile can access another profile's data.
    pub fn can_access(&self, viewer_id: u64, target_id: u64) -> bool {
        if viewer_id == target_id {
            return true; // Can always access own data
        }
        let viewer = match self.profiles.get(&viewer_id) {
            Some(p) => p,
            None => return false,
        };
        let target = match self.profiles.get(&target_id) {
            Some(p) => p,
            None => return false,
        };
        // Containers cannot access other profiles' data
        // Default profiles cannot access container data
        viewer.isolation == IsolationLevel::Default
            && target.isolation != IsolationLevel::Container
    }

    /// Clear data for ephemeral profiles that should be cleared on close.
    pub fn close_ephemeral_profiles(&mut self) {
        for p in self.profiles.values_mut() {
            if p.clear_on_close || p.isolation == IsolationLevel::Ephemeral {
                p.clear_data();
            }
        }
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
    fn create_default_profile() {
        let mut pm = ProfileManager::new();
        let id = pm.create_profile("Default", IsolationLevel::Default);
        assert_eq!(id, 0);
        let p = pm.get_profile(id).unwrap();
        assert_eq!(p.name, "Default");
        assert!(p.is_active);
        assert!(pm.active_profile().is_some());
    }

    #[test]
    fn create_container_profile() {
        let mut pm = ProfileManager::new();
        let id = pm.create_container("Work");
        let p = pm.get_profile(id).unwrap();
        assert_eq!(p.isolation, IsolationLevel::Container);
    }

    #[test]
    fn create_ephemeral_profile() {
        let mut pm = ProfileManager::new();
        let id = pm.create_ephemeral("Temp");
        let p = pm.get_profile(id).unwrap();
        assert_eq!(p.isolation, IsolationLevel::Ephemeral);
        assert!(p.clear_on_close);
    }

    #[test]
    fn switch_active_profile() {
        let mut pm = ProfileManager::new();
        let p1 = pm.create_profile("Profile 1", IsolationLevel::Default);
        let p2 = pm.create_profile("Profile 2", IsolationLevel::Default);

        assert!(pm.active_profile().unwrap().id == p1);
        assert!(pm.set_active(p2));
        assert!(pm.active_profile().unwrap().id == p2);
    }

    #[test]
    fn remove_profile_switches_active() {
        let mut pm = ProfileManager::new();
        let p1 = pm.create_profile("Profile 1", IsolationLevel::Default);
        let _p2 = pm.create_profile("Profile 2", IsolationLevel::Default);

        assert!(pm.remove_profile(p1));
        assert!(pm.active_profile().unwrap().name == "Profile 2");
    }

    #[test]
    fn isolation_between_profiles() {
        let mut pm = ProfileManager::new();
        let default = pm.create_profile("Default", IsolationLevel::Default);
        let container = pm.create_container("Work");

        // Default can access default (own)
        assert!(pm.can_access(default, default));
        // Default can access container (one-way)
        assert!(pm.can_access(default, container));
        // Container cannot access default
        assert!(!pm.can_access(container, default));
        // Container cannot access other containers
        let container2 = pm.create_container("Personal");
        assert!(!pm.can_access(container, container2));
    }

    #[test]
    fn close_ephemeral_profiles() {
        let mut pm = ProfileManager::new();
        let _ephemeral = pm.create_ephemeral("Temp");
        let _manual = {
            let id = pm.create_container("Manual");
            let p = pm.get_profile_mut(id).unwrap();
            p.clear_on_close = true;
            id
        };
        let _keep = pm.create_profile("Keep", IsolationLevel::Default);

        pm.close_ephemeral_profiles();
        // Ephemeral and manual profiles should have cleared data
        assert_eq!(
            pm.get_profile(0).unwrap().bookmarks.bookmark_count(),
            0
        );
    }

    #[test]
    fn profile_with_settings() {
        let mut pm = ProfileManager::new();
        let id = pm.create_profile("Custom", IsolationLevel::Default);
        let p = pm.get_profile_mut(id).unwrap();
        p.homepage = Some("https://example.com".into());
        p.search_engine = Some("DuckDuckGo".into());
        p.block_trackers = true;
        p.https_only = false;

        let p = pm.get_profile(id).unwrap();
        assert_eq!(p.homepage.as_deref(), Some("https://example.com"));
        assert_eq!(p.search_engine.as_deref(), Some("DuckDuckGo"));
        assert!(!p.https_only);
    }

    #[test]
    fn list_profiles_by_level() {
        let mut pm = ProfileManager::new();
        pm.create_profile("Default", IsolationLevel::Default);
        pm.create_container("Work");
        pm.create_container("Personal");
        pm.create_ephemeral("Temp");

        assert_eq!(pm.profiles_with_level(IsolationLevel::Default).len(), 1);
        assert_eq!(pm.profiles_with_level(IsolationLevel::Container).len(), 2);
        assert_eq!(pm.profiles_with_level(IsolationLevel::Ephemeral).len(), 1);
    }
}
