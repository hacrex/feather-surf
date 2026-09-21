// FeatherSurf Session Manager
//
// Handles session persistence and restoration. Sessions are saved atomically
// using temporary files and rename to prevent corruption on crash.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use tab_manager::{Tab, TabSnapshot, TabState};

/// Current session schema version. Bump when SessionData fields change.
pub const SESSION_SCHEMA_VERSION: u32 = 1;

/// Maximum number of backup session files to keep.
const MAX_BACKUPS: usize = 5;

/// A single tab's session data.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct SessionTab {
    pub id: u64,
    pub url: String,
    pub title: String,
    pub scroll_position: (u32, u32),
    pub form_state: Option<String>,
    pub is_pinned: bool,
    pub is_active: bool,
}

impl SessionTab {
    /// Create a SessionTab from a Tab reference.
    pub fn from_tab(tab: &Tab, is_active: bool) -> Self {
        let snapshot = tab.snapshot();
        Self {
            id: tab.id(),
            url: snapshot.url.clone(),
            title: snapshot.title.clone(),
            scroll_position: snapshot.scroll_position,
            form_state: snapshot.form_state.clone(),
            is_pinned: tab.protection().pinned,
            is_active,
        }
    }

    /// Restore a Tab from session data.
    pub fn to_tab(&self) -> Tab {
        let mut tab = Tab::new(self.id, &self.url);
        tab.snapshot_mut().title = self.title.clone();
        tab.snapshot_mut().scroll_position = self.scroll_position;
        tab.snapshot_mut().form_state = self.form_state.clone();
        if self.is_pinned {
            tab.set_protection(tab_manager::TabProtection {
                pinned: true,
                ..tab_manager::TabProtection::default()
            });
        }
        tab
    }
}

/// A window's session data.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct SessionWindow {
    pub tabs: Vec<SessionTab>,
    pub active_tab_index: usize,
}

/// Complete session data that can be serialized to disk.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct SessionData {
    pub schema_version: u32,
    pub saved_at: u64,  // Unix timestamp in seconds
    pub windows: Vec<SessionWindow>,
}

impl SessionData {
    /// Create a new empty session.
    pub fn new() -> Self {
        Self {
            schema_version: SESSION_SCHEMA_VERSION,
            saved_at: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            windows: vec![SessionWindow {
                tabs: vec![],
                active_tab_index: 0,
            }],
        }
    }

    /// Serialize the session to bytes.
    pub fn serialize(&self) -> Result<Vec<u8>, SessionError> {
        bincode::serialize(self).map_err(|e| SessionError::Serialize(e.to_string()))
    }

    /// Deserialize session from bytes.
    pub fn deserialize(data: &[u8]) -> Result<Self, SessionError> {
        let session: SessionData =
            bincode::deserialize(data).map_err(|e| SessionError::Deserialize(e.to_string()))?;

        if session.schema_version > SESSION_SCHEMA_VERSION {
            return Err(SessionError::IncompatibleVersion {
                stored: session.schema_version,
                current: SESSION_SCHEMA_VERSION,
            });
        }

        Ok(session)
    }

    /// Validate the session data.
    pub fn validate(&self) -> Result<(), SessionError> {
        if self.windows.is_empty() {
            return Err(SessionError::NoWindows);
        }

        for (i, window) in self.windows.iter().enumerate() {
            if window.tabs.is_empty() {
                return Err(SessionError::EmptyWindow(i));
            }
            if window.active_tab_index >= window.tabs.len() {
                return Err(SessionError::InvalidActiveTab {
                    window: i,
                    index: window.active_tab_index,
                    tab_count: window.tabs.len(),
                });
            }
            for (j, tab) in window.tabs.iter().enumerate() {
                if tab.url.is_empty() {
                    return Err(SessionError::EmptyUrl { window: i, tab: j });
                }
            }
        }

        Ok(())
    }
}

impl Default for SessionData {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur during session operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionError {
    Serialize(String),
    Deserialize(String),
    IncompatibleVersion { stored: u32, current: u32 },
    NoWindows,
    EmptyWindow(usize),
    InvalidActiveTab {
        window: usize,
        index: usize,
        tab_count: usize,
    },
    EmptyUrl { window: usize, tab: usize },
    IoError(String),
    CorruptedFile(String),
}

impl std::fmt::Display for SessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Serialize(msg) => write!(f, "session serialization failed: {msg}"),
            Self::Deserialize(msg) => write!(f, "session deserialization failed: {msg}"),
            Self::IncompatibleVersion { stored, current } => {
                write!(f, "session version {stored} incompatible with {current}")
            }
            Self::NoWindows => write!(f, "session has no windows"),
            Self::EmptyWindow(i) => write!(f, "window {i} has no tabs"),
            Self::InvalidActiveTab {
                window,
                index,
                tab_count,
            } => write!(
                f,
                "window {window}: active tab index {index} out of range (0..{tab_count})"
            ),
            Self::EmptyUrl { window, tab } => {
                write!(f, "window {window}, tab {tab} has empty URL")
            }
            Self::IoError(msg) => write!(f, "session I/O error: {msg}"),
            Self::CorruptedFile(msg) => write!(f, "session file corrupted: {msg}"),
        }
    }
}

impl std::error::Error for SessionError {}

/// Session manager that handles saving and loading sessions.
pub struct SessionManager {
    session_dir: PathBuf,
}

impl SessionManager {
    /// Create a new session manager with the given session directory.
    pub fn new(session_dir: impl Into<PathBuf>) -> Self {
        Self {
            session_dir: session_dir.into(),
        }
    }

    /// Get the path to the current session file.
    fn session_path(&self) -> PathBuf {
        self.session_dir.join("session.bin")
    }

    /// Get the path to a backup session file.
    fn backup_path(&self, index: usize) -> PathBuf {
        self.session_dir.join(format!("session.{index}.bak"))
    }

    /// Save a session atomically.
    ///
    /// 1. Write to a temporary file
    /// 2. Rename existing session to backup
    /// 3. Rename temporary file to session file
    ///
    /// This ensures that a crash during save never corrupts the session.
    pub fn save(&self, session: &SessionData) -> Result<(), SessionError> {
        // Validate before saving
        session.validate()?;

        // Ensure directory exists
        fs::create_dir_all(&self.session_dir)
            .map_err(|e| SessionError::IoError(e.to_string()))?;

        let session_path = self.session_path();
        let temp_path = self.session_dir.join("session.tmp");

        // Serialize
        let data = session.serialize()?;

        // Write to temporary file
        fs::write(&temp_path, &data).map_err(|e| SessionError::IoError(e.to_string()))?;

        // Rotate backups
        self.rotate_backups()?;

        // Move existing session to backup
        if session_path.exists() {
            let backup = self.backup_path(0);
            if backup.exists() {
                fs::remove_file(&backup).map_err(|e| SessionError::IoError(e.to_string()))?;
            }
            fs::rename(&session_path, &backup)
                .map_err(|e| SessionError::IoError(e.to_string()))?;
        }

        // Move temporary file to session file
        fs::rename(&temp_path, &session_path)
            .map_err(|e| SessionError::IoError(e.to_string()))?;

        Ok(())
    }

    /// Load the session from disk.
    ///
    /// If the primary session file is corrupted, tries backups in order.
    pub fn load(&self) -> Result<SessionData, SessionError> {
        // Try primary file first
        match self.load_from(&self.session_path()) {
            Ok(session) => return Ok(session),
            Err(e) => {
                eprintln!("Primary session file failed: {e}, trying backups");
            }
        }

        // Try backups in reverse order (newest first)
        for i in 0..MAX_BACKUPS {
            let backup = self.backup_path(i);
            if backup.exists() {
                match self.load_from(&backup) {
                    Ok(session) => {
                        eprintln!("Recovered session from backup {i}");
                        return Ok(session);
                    }
                    Err(_) => continue,
                }
            }
        }

        // No valid session found - return empty
        Ok(SessionData::new())
    }

    /// Load session from a specific file.
    fn load_from(&self, path: &Path) -> Result<SessionData, SessionError> {
        if !path.exists() {
            return Err(SessionError::IoError("file not found".into()));
        }

        let data = fs::read(path).map_err(|e| SessionError::IoError(e.to_string()))?;

        let session = SessionData::deserialize(&data)?;
        session.validate()?;

        Ok(session)
    }

    /// Rotate backup files, removing the oldest if at capacity.
    fn rotate_backups(&self) -> Result<(), SessionError> {
        // Shift backups: session.N-1.bak -> deleted, session.N-2.bak -> session.N-1.bak, ...
        for i in (0..MAX_BACKUPS - 1).rev() {
            let src = self.backup_path(i);
            let dst = self.backup_path(i + 1);
            if src.exists() {
                if dst.exists() {
                    fs::remove_file(&dst).map_err(|e| SessionError::IoError(e.to_string()))?;
                }
                fs::rename(&src, &dst).map_err(|e| SessionError::IoError(e.to_string()))?;
            }
        }
        Ok(())
    }

    /// Delete all session files (for profile deletion).
    pub fn delete_all(&self) -> Result<(), SessionError> {
        let session_path = self.session_path();
        if session_path.exists() {
            fs::remove_file(&session_path).map_err(|e| SessionError::IoError(e.to_string()))?;
        }
        for i in 0..MAX_BACKUPS {
            let backup = self.backup_path(i);
            if backup.exists() {
                fs::remove_file(&backup).map_err(|e| SessionError::IoError(e.to_string()))?;
            }
        }
        Ok(())
    }

    /// Check if a session file exists.
    pub fn has_session(&self) -> bool {
        self.session_path().exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_manager() -> (SessionManager, TempDir) {
        let tmp = TempDir::new().unwrap();
        let mgr = SessionManager::new(tmp.path().join("sessions"));
        (mgr, tmp)
    }

    fn sample_session() -> SessionData {
        SessionData {
            schema_version: SESSION_SCHEMA_VERSION,
            saved_at: 1000000,
            windows: vec![SessionWindow {
                tabs: vec![
                    SessionTab {
                        id: 1,
                        url: "https://example.com".into(),
                        title: "Example".into(),
                        scroll_position: (0, 100),
                        form_state: None,
                        is_pinned: false,
                        is_active: true,
                    },
                    SessionTab {
                        id: 2,
                        url: "https://github.com".into(),
                        title: "GitHub".into(),
                        scroll_position: (0, 0),
                        form_state: Some("draft".into()),
                        is_pinned: true,
                        is_active: false,
                    },
                ],
                active_tab_index: 0,
            }],
        }
    }

    #[test]
    fn save_and_load_roundtrip() {
        let (mgr, _tmp) = test_manager();
        let session = sample_session();
        mgr.save(&session).unwrap();
        let loaded = mgr.load().unwrap();
        assert_eq!(session, loaded);
    }

    #[test]
    fn empty_session_saves_and_loads() {
        let (mgr, _tmp) = test_manager();
        let session = SessionData::new();
        mgr.save(&session).unwrap();
        let loaded = mgr.load().unwrap();
        assert_eq!(session.windows.len(), 1);
        assert!(loaded.windows[0].tabs.is_empty());
    }

    #[test]
    fn load_returns_empty_when_no_file() {
        let (mgr, _tmp) = test_manager();
        let session = mgr.load().unwrap();
        assert!(session.windows[0].tabs.is_empty());
    }

    #[test]
    fn corrupted_file_tries_backups() {
        let (mgr, _tmp) = test_manager();
        let session = sample_session();
        mgr.save(&session).unwrap();

        // Corrupt the primary file
        let session_path = mgr.session_path();
        fs::write(&session_path, b"corrupted data").unwrap();

        // Should recover from backup
        let loaded = mgr.load().unwrap();
        assert_eq!(session, loaded);
    }

    #[test]
    fn validate_rejects_empty_windows() {
        let session = SessionData {
            windows: vec![],
            ..sample_session()
        };
        assert!(matches!(session.validate(), Err(SessionError::NoWindows)));
    }

    #[test]
    fn validate_rejects_empty_window() {
        let session = SessionData {
            windows: vec![SessionWindow {
                tabs: vec![],
                active_tab_index: 0,
            }],
            ..sample_session()
        };
        assert!(matches!(
            session.validate(),
            Err(SessionError::EmptyWindow(0))
        ));
    }

    #[test]
    fn validate_rejects_invalid_active_tab() {
        let session = SessionData {
            windows: vec![SessionWindow {
                tabs: vec![SessionTab {
                    id: 1,
                    url: "https://example.com".into(),
                    title: "Test".into(),
                    scroll_position: (0, 0),
                    form_state: None,
                    is_pinned: false,
                    is_active: false,
                }],
                active_tab_index: 5,  // Out of range
            }],
            ..sample_session()
        };
        assert!(matches!(
            session.validate(),
            Err(SessionError::InvalidActiveTab { .. })
        ));
    }

    #[test]
    fn validate_rejects_empty_url() {
        let session = SessionData {
            windows: vec![SessionWindow {
                tabs: vec![SessionTab {
                    id: 1,
                    url: "".into(),  // Empty URL
                    title: "Test".into(),
                    scroll_position: (0, 0),
                    form_state: None,
                    is_pinned: false,
                    is_active: false,
                }],
                active_tab_index: 0,
            }],
            ..sample_session()
        };
        assert!(matches!(
            session.validate(),
            Err(SessionError::EmptyUrl { .. })
        ));
    }

    #[test]
    fn reject_future_version() {
        let session = SessionData {
            schema_version: SESSION_SCHEMA_VERSION + 1,
            ..sample_session()
        };
        let data = session.serialize().unwrap();
        let result = SessionData::deserialize(&data);
        assert!(matches!(
            result,
            Err(SessionError::IncompatibleVersion { .. })
        ));
    }

    #[test]
    fn multiple_windows_save_and_load() {
        let (mgr, _tmp) = test_manager();
        let session = SessionData {
            windows: vec![
                SessionWindow {
                    tabs: vec![SessionTab {
                        id: 1,
                        url: "https://a.com".into(),
                        title: "A".into(),
                        scroll_position: (0, 0),
                        form_state: None,
                        is_pinned: false,
                        is_active: true,
                    }],
                    active_tab_index: 0,
                },
                SessionWindow {
                    tabs: vec![SessionTab {
                        id: 2,
                        url: "https://b.com".into(),
                        title: "B".into(),
                        scroll_position: (0, 50),
                        form_state: None,
                        is_pinned: true,
                        is_active: false,
                    }],
                    active_tab_index: 0,
                },
            ],
            ..sample_session()
        };
        mgr.save(&session).unwrap();
        let loaded = mgr.load().unwrap();
        assert_eq!(session, loaded);
        assert_eq!(loaded.windows.len(), 2);
    }

    #[test]
    fn delete_all_removes_files() {
        let (mgr, _tmp) = test_manager();
        let session = sample_session();
        mgr.save(&session).unwrap();
        assert!(mgr.has_session());
        mgr.delete_all().unwrap();
        assert!(!mgr.has_session());
    }

    #[test]
    fn session_tab_from_tab_roundtrip() {
        let mut tab = Tab::new(42, "https://example.com");
        tab.snapshot_mut().title = "Example".into();
        tab.snapshot_mut().scroll_position = (10, 200);
        tab.set_protection(tab_manager::TabProtection {
            pinned: true,
            ..tab_manager::TabProtection::default()
        });

        let session_tab = SessionTab::from_tab(&tab, true);
        assert_eq!(session_tab.id, 42);
        assert_eq!(session_tab.url, "https://example.com");
        assert!(session_tab.is_pinned);
        assert!(session_tab.is_active);

        let restored = session_tab.to_tab();
        assert_eq!(restored.id(), 42);
        assert_eq!(restored.snapshot().url, "https://example.com");
        assert_eq!(restored.snapshot().title, "Example");
        assert_eq!(restored.snapshot().scroll_position, (10, 200));
        assert!(restored.protection().pinned);
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn session_roundtrip_for_any_valid_session(
            tab_count in 1usize..20,
            window_count in 1usize..5
        ) {
            let mut windows = Vec::new();
            let mut next_id = 1u64;
            for _ in 0..window_count {
                let mut tabs = Vec::new();
                let active = (next_id as usize) % tab_count;
                for j in 0..tab_count {
                    tabs.push(SessionTab {
                        id: next_id,
                        url: format!("https://example{}.com", next_id),
                        title: format!("Tab {}", next_id),
                        scroll_position: ((j as u32) * 10, (j as u32) * 100),
                        form_state: if j % 3 == 0 { Some("draft".into()) } else { None },
                        is_pinned: j % 5 == 0,
                        is_active: j == active,
                    });
                    next_id += 1;
                }
                windows.push(SessionWindow {
                    tabs,
                    active_tab_index: active,
                });
            }

            let session = SessionData {
                schema_version: SESSION_SCHEMA_VERSION,
                saved_at: 1000000,
                windows,
            };

            let data = session.serialize().unwrap();
            let loaded = SessionData::deserialize(&data).unwrap();
            prop_assert_eq!(&session, &loaded);
        }
    }
}
