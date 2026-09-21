// FeatherSurf Session Migrations
//
// Session schema versioning and migration support.
// Handles backward-compatible upgrades of session data.

use std::collections::HashMap;
use std::time::SystemTime;

// ── Schema Version ─────────────────────────────────────────────────

/// Current session schema version.
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// A migration from one schema version to another.
pub trait Migration {
    /// The version this migration goes from.
    fn from_version(&self) -> u32;

    /// The version this migration goes to.
    fn to_version(&self) -> u32;

    /// Apply the migration to the data.
    fn apply(&self, data: &mut SessionData) -> Result<(), MigrationError>;
}

/// Migration error.
#[derive(Debug, Clone, thiserror::Error)]
pub enum MigrationError {
    #[error("Unsupported version: {0}")]
    UnsupportedVersion(u32),

    #[error("Migration failed: {0}")]
    Failed(String),

    #[error("Data corruption detected")]
    Corruption,
}

// ── Session Data ───────────────────────────────────────────────────

/// Session data that can be migrated.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionData {
    /// Schema version of this data.
    pub schema_version: u32,
    /// Open windows.
    pub windows: Vec<SessionWindow>,
    /// Active window index.
    pub active_window: usize,
    /// Profile data.
    pub profile_id: Option<u64>,
    /// Timestamp.
    pub timestamp: u64,
}

/// A window in the session.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionWindow {
    /// Window position and size.
    pub bounds: WindowBounds,
    /// Tabs in this window.
    pub tabs: Vec<SessionTab>,
    /// Active tab index.
    pub active_tab: usize,
}

/// Window position and size.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WindowBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub maximized: bool,
}

/// A tab in the session.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionTab {
    pub url: String,
    pub title: String,
    pub scroll_x: i32,
    pub scroll_y: i32,
    pub zoom_level: f64,
    pub pinned: bool,
    pub keep_awake: bool,
    /// Navigation history for this tab.
    pub history: Vec<HistoryEntry>,
    /// Current history index.
    pub history_index: usize,
}

/// A history entry for a tab.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HistoryEntry {
    pub url: String,
    pub title: String,
    pub timestamp: u64,
}

// ── Migrator ───────────────────────────────────────────────────────

/// Manages schema migrations.
pub struct Migrator {
    migrations: Vec<Box<dyn Migration>>,
}

impl Migrator {
    pub fn new() -> Self {
        let mut migrator = Self {
            migrations: Vec::new(),
        };

        // Register built-in migrations
        migrator.register(MigrationV0ToV1);

        migrator
    }

    /// Register a migration.
    pub fn register(&mut self, migration: impl Migration + 'static) {
        self.migrations.push(Box::new(migration));
    }

    /// Migrate session data to the current schema version.
    pub fn migrate(&self, data: &mut SessionData) -> Result<(), MigrationError> {
        while data.schema_version < CURRENT_SCHEMA_VERSION {
            let migration = self
                .migrations
                .iter()
                .find(|m| m.from_version() == data.schema_version)
                .ok_or(MigrationError::UnsupportedVersion(data.schema_version))?;

            migration.apply(data)?;
            data.schema_version = migration.to_version();
        }

        Ok(())
    }

    /// Get all available migration paths.
    pub fn migration_paths(&self) -> Vec<(u32, u32)> {
        self.migrations
            .iter()
            .map(|m| (m.from_version(), m.to_version()))
            .collect()
    }

    /// Check if migration is needed.
    pub fn needs_migration(&self, version: u32) -> bool {
        version < CURRENT_SCHEMA_VERSION
    }
}

impl Default for Migrator {
    fn default() -> Self {
        Self::new()
    }
}

// ── Built-in Migrations ────────────────────────────────────────────

/// Migration from version 0 to 1.
struct MigrationV0ToV1;

impl Migration for MigrationV0ToV1 {
    fn from_version(&self) -> u32 {
        0
    }

    fn to_version(&self) -> u32 {
        1
    }

    fn apply(&self, data: &mut SessionData) -> Result<(), MigrationError> {
        // Version 0 -> 1: Add history fields to tabs
        for window in &mut data.windows {
            for tab in &mut window.tabs {
                if tab.history.is_empty() {
                    // Initialize history from current URL
                    tab.history = vec![HistoryEntry {
                        url: tab.url.clone(),
                        title: tab.title.clone(),
                        timestamp: now_secs(),
                    }];
                    tab.history_index = 0;
                }
            }
        }
        Ok(())
    }
}

// ── Session Manager ────────────────────────────────────────────────

/// Session manager with migration support.
pub struct SessionManager {
    migrator: Migrator,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            migrator: Migrator::new(),
        }
    }

    /// Load and migrate session data.
    pub fn load(&self, data: &mut SessionData) -> Result<(), MigrationError> {
        self.migrator.migrate(data)
    }

    /// Create a new empty session.
    pub fn new_session(&self) -> SessionData {
        SessionData {
            schema_version: CURRENT_SCHEMA_VERSION,
            windows: vec![SessionWindow {
                bounds: WindowBounds {
                    x: 100,
                    y: 100,
                    width: 1200,
                    height: 800,
                    maximized: false,
                },
                tabs: vec![SessionTab {
                    url: "about:blank".to_string(),
                    title: "New Tab".to_string(),
                    scroll_x: 0,
                    scroll_y: 0,
                    zoom_level: 1.0,
                    pinned: false,
                    keep_awake: false,
                    history: vec![HistoryEntry {
                        url: "about:blank".to_string(),
                        title: "New Tab".to_string(),
                        timestamp: now_secs(),
                    }],
                    history_index: 0,
                }],
                active_tab: 0,
            }],
            active_window: 0,
            profile_id: None,
            timestamp: now_secs(),
        }
    }

    /// Create a tab with history support.
    pub fn new_tab(&self, url: &str, title: &str) -> SessionTab {
        SessionTab {
            url: url.to_string(),
            title: title.to_string(),
            scroll_x: 0,
            scroll_y: 0,
            zoom_level: 1.0,
            pinned: false,
            keep_awake: false,
            history: vec![HistoryEntry {
                url: url.to_string(),
                title: title.to_string(),
                timestamp: now_secs(),
            }],
            history_index: 0,
        }
    }

    /// Navigate a tab to a new URL (adds to history).
    pub fn navigate_tab(&self, tab: &mut SessionTab, url: &str, title: &str) {
        // Truncate forward history
        tab.history.truncate(tab.history_index + 1);

        // Add new entry
        tab.history.push(HistoryEntry {
            url: url.to_string(),
            title: title.to_string(),
            timestamp: now_secs(),
        });
        tab.history_index = tab.history.len() - 1;

        tab.url = url.to_string();
        tab.title = title.to_string();
    }

    /// Go back in tab history.
    pub fn go_back(&self, tab: &mut SessionTab) -> bool {
        if tab.history_index > 0 {
            tab.history_index -= 1;
            let entry = &tab.history[tab.history_index];
            tab.url = entry.url.clone();
            tab.title = entry.title.clone();
            true
        } else {
            false
        }
    }

    /// Go forward in tab history.
    pub fn go_forward(&self, tab: &mut SessionTab) -> bool {
        if tab.history_index < tab.history.len() - 1 {
            tab.history_index += 1;
            let entry = &tab.history[tab.history_index];
            tab.url = entry.url.clone();
            tab.title = entry.title.clone();
            true
        } else {
            false
        }
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
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
    fn new_session_has_correct_version() {
        let manager = SessionManager::new();
        let session = manager.new_session();
        assert_eq!(session.schema_version, CURRENT_SCHEMA_VERSION);
    }

    #[test]
    fn navigate_adds_history() {
        let manager = SessionManager::new();
        let mut tab = manager.new_tab("https://example.com", "Example");
        assert_eq!(tab.history.len(), 1);

        manager.navigate_tab(&mut tab, "https://rust-lang.org", "Rust");
        assert_eq!(tab.history.len(), 2);
        assert_eq!(tab.history_index, 1);
        assert_eq!(tab.url, "https://rust-lang.org");
    }

    #[test]
    fn go_back_and_forward() {
        let manager = SessionManager::new();
        let mut tab = manager.new_tab("https://example.com", "Example");
        manager.navigate_tab(&mut tab, "https://rust-lang.org", "Rust");
        manager.navigate_tab(&mut tab, "https://docs.rs", "Docs");

        assert!(manager.go_back(&mut tab));
        assert_eq!(tab.url, "https://rust-lang.org");

        assert!(manager.go_forward(&mut tab));
        assert_eq!(tab.url, "https://docs.rs");
    }

    #[test]
    fn go_back_at_start_returns_false() {
        let manager = SessionManager::new();
        let mut tab = manager.new_tab("https://example.com", "Example");
        assert!(!manager.go_back(&mut tab));
    }

    #[test]
    fn go_forward_at_end_returns_false() {
        let manager = SessionManager::new();
        let mut tab = manager.new_tab("https://example.com", "Example");
        assert!(!manager.go_forward(&mut tab));
    }

    #[test]
    fn navigate_truncates_forward_history() {
        let manager = SessionManager::new();
        let mut tab = manager.new_tab("https://example.com", "Example");
        manager.navigate_tab(&mut tab, "https://rust-lang.org", "Rust");
        manager.navigate_tab(&mut tab, "https://docs.rs", "Docs");

        // Go back twice
        manager.go_back(&mut tab);
        manager.go_back(&mut tab);

        // Navigate to new URL - should truncate forward history
        manager.navigate_tab(&mut tab, "https://new.com", "New");
        assert_eq!(tab.history.len(), 2); // example.com and new.com
        assert_eq!(tab.history_index, 1);
    }

    #[test]
    fn migration_from_v0() {
        let mut data = SessionData {
            schema_version: 0,
            windows: vec![SessionWindow {
                bounds: WindowBounds {
                    x: 0,
                    y: 0,
                    width: 800,
                    height: 600,
                    maximized: false,
                },
                tabs: vec![SessionTab {
                    url: "https://example.com".to_string(),
                    title: "Example".to_string(),
                    scroll_x: 0,
                    scroll_y: 0,
                    zoom_level: 1.0,
                    pinned: false,
                    keep_awake: false,
                    history: Vec::new(), // Empty in v0
                    history_index: 0,
                }],
                active_tab: 0,
            }],
            active_window: 0,
            profile_id: None,
            timestamp: 0,
        };

        let manager = SessionManager::new();
        manager.load(&mut data).unwrap();

        assert_eq!(data.schema_version, CURRENT_SCHEMA_VERSION);
        assert!(!data.windows[0].tabs[0].history.is_empty());
    }
}
