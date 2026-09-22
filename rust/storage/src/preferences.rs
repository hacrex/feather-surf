// FeatherSurf Preferences
//
// Browser preferences with defaults, validation, and serialization.
// Supports per-profile preferences and privacy-safe defaults.

use std::collections::HashMap;
use std::time::SystemTime;

// ── Preference Categories ──────────────────────────────────────────

/// Browser preferences organized by category.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Preferences {
    /// General preferences.
    pub general: GeneralPrefs,
    /// Privacy and security preferences.
    pub privacy: PrivacyPrefs,
    /// Performance preferences.
    pub performance: PerformancePrefs,
    /// Appearance preferences.
    pub appearance: AppearancePrefs,
    /// Keyboard shortcuts.
    pub shortcuts: HashMap<String, String>,
    /// Custom settings.
    pub custom: HashMap<String, String>,
    /// Last modified timestamp.
    pub modified_at: u64,
}

/// General browser preferences.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GeneralPrefs {
    /// Homepage URL.
    pub homepage: String,
    /// Search engine URL (with {query} placeholder).
    pub search_engine: String,
    /// Default download directory.
    pub download_dir: String,
    /// Open tabs from previous session on startup.
    pub restore_session: bool,
    /// Show home button in toolbar.
    pub show_home_button: bool,
}

impl Default for GeneralPrefs {
    fn default() -> Self {
        Self {
            homepage: "about:blank".to_string(),
            search_engine: "https://duckduckgo.com/?q={query}".to_string(),
            download_dir: dirs::download_dir()
                .map(|d| d.to_string_lossy().to_string())
                .unwrap_or_default(),
            restore_session: true,
            show_home_button: false,
        }
    }
}

/// Privacy and security preferences.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PrivacyPrefs {
    /// Block trackers.
    pub block_trackers: bool,
    /// Block third-party cookies.
    pub block_third_party_cookies: bool,
    /// Clear data on close.
    pub clear_on_close: bool,
    /// Clear cookies on close.
    pub clear_cookies_on_close: bool,
    /// Clear history on close.
    pub clear_history_on_close: bool,
    /// Clear downloads on close.
    pub clear_downloads_on_close: bool,
    /// Send Do Not Track header.
    pub send_dnt: bool,
    /// HTTPS-only mode.
    pub https_only: bool,
    /// Block fingerprinting.
    pub block_fingerprinting: bool,
    /// Block popups.
    pub block_popups: bool,
    /// Block notifications.
    pub block_notifications: bool,
    /// Auto-play media.
    pub auto_play_media: bool,
    /// Memory protection level.
    pub memory_protection: MemoryProtection,
}

impl Default for PrivacyPrefs {
    fn default() -> Self {
        Self {
            block_trackers: true,
            block_third_party_cookies: true,
            clear_on_close: false,
            clear_cookies_on_close: false,
            clear_history_on_close: false,
            clear_downloads_on_close: false,
            send_dnt: true,
            https_only: true,
            block_fingerprinting: true,
            block_popups: true,
            block_notifications: false,
            auto_play_media: false,
            memory_protection: MemoryProtection::Balanced,
        }
    }
}

/// Memory protection level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MemoryProtection {
    /// Aggressive freezing, minimal background tabs.
    Aggressive,
    /// Balanced between performance and memory.
    Balanced,
    /// Allow more background tabs, less freezing.
    Relaxed,
    /// No automatic memory management.
    Disabled,
}

/// Performance preferences.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PerformancePrefs {
    /// Memory mode (affects eviction thresholds).
    pub memory_mode: String,
    /// Enable hardware acceleration.
    pub hardware_acceleration: bool,
    /// Maximum background tabs.
    pub max_background_tabs: usize,
    /// Freeze tabs after N seconds of inactivity.
    pub freeze_after_secs: u64,
    /// Suspend tabs after N seconds of inactivity.
    pub suspend_after_secs: u64,
    /// Discard tabs after N seconds of inactivity.
    pub discard_after_secs: u64,
    /// Memory budget in MB (0 = unlimited).
    pub memory_budget_mb: u64,
    /// Enable tab preloading.
    pub preload_tabs: bool,
}

impl Default for PerformancePrefs {
    fn default() -> Self {
        Self {
            memory_mode: "Balanced".to_string(),
            hardware_acceleration: true,
            max_background_tabs: 20,
            freeze_after_secs: 300,   // 5 minutes
            suspend_after_secs: 1800, // 30 minutes
            discard_after_secs: 3600, // 1 hour
            memory_budget_mb: 0,      // unlimited
            preload_tabs: true,
        }
    }
}

/// Appearance preferences.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppearancePrefs {
    /// Theme (system, light, dark).
    pub theme: String,
    /// Font size multiplier.
    pub font_size: f64,
    /// Show bookmarks bar.
    pub show_bookmarks_bar: bool,
    /// Show status bar.
    pub show_status_bar: bool,
    /// Tab position (top, bottom, left, right).
    pub tab_position: String,
    /// Show Developer Center in toolbar.
    pub show_dev_center: bool,
}

impl Default for AppearancePrefs {
    fn default() -> Self {
        Self {
            theme: "system".to_string(),
            font_size: 1.0,
            show_bookmarks_bar: false,
            show_status_bar: true,
            tab_position: "top".to_string(),
            show_dev_center: false,
        }
    }
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            general: GeneralPrefs::default(),
            privacy: PrivacyPrefs::default(),
            performance: PerformancePrefs::default(),
            appearance: AppearancePrefs::default(),
            shortcuts: default_shortcuts(),
            custom: HashMap::new(),
            modified_at: now_secs(),
        }
    }
}

impl Preferences {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update modified timestamp.
    pub fn touch(&mut self) {
        self.modified_at = now_secs();
    }

    /// Get a shortcut by action name.
    pub fn get_shortcut(&self, action: &str) -> Option<&str> {
        self.shortcuts.get(action).map(|s| s.as_str())
    }

    /// Set a shortcut.
    pub fn set_shortcut(&mut self, action: impl Into<String>, key: impl Into<String>) {
        self.shortcuts.insert(action.into(), key.into());
        self.touch();
    }

    /// Get a custom setting.
    pub fn get_custom(&self, key: &str) -> Option<&str> {
        self.custom.get(key).map(|s| s.as_str())
    }

    /// Set a custom setting.
    pub fn set_custom(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.custom.insert(key.into(), value.into());
        self.touch();
    }

    /// Validate preferences and fix invalid values.
    pub fn validate(&mut self) {
        // Clamp font size
        self.appearance.font_size = self.appearance.font_size.clamp(0.5, 3.0);

        // Validate memory mode
        if !["Lite", "Balanced", "Performance"].contains(&self.performance.memory_mode.as_str()) {
            self.performance.memory_mode = "Balanced".to_string();
        }

        // Validate theme
        if !["system", "light", "dark"].contains(&self.appearance.theme.as_str()) {
            self.appearance.theme = "system".to_string();
        }

        // Validate tab position
        if !["top", "bottom", "left", "right"].contains(&self.appearance.tab_position.as_str()) {
            self.appearance.tab_position = "top".to_string();
        }

        // Ensure freeze < suspend < discard
        if self.performance.freeze_after_secs >= self.performance.suspend_after_secs {
            self.performance.suspend_after_secs = self.performance.freeze_after_secs + 300;
        }
        if self.performance.suspend_after_secs >= self.performance.discard_after_secs {
            self.performance.discard_after_secs = self.performance.suspend_after_secs + 1800;
        }
    }

    /// Export as JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Import from JSON.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let mut prefs: Self = serde_json::from_str(json)?;
        prefs.validate();
        Ok(prefs)
    }
}

// ── Default Shortcuts ──────────────────────────────────────────────

fn default_shortcuts() -> HashMap<String, String> {
    let mut shortcuts = HashMap::new();
    shortcuts.insert("new_tab".to_string(), "Ctrl+T".to_string());
    shortcuts.insert("close_tab".to_string(), "Ctrl+W".to_string());
    shortcuts.insert("address_bar".to_string(), "Ctrl+L".to_string());
    shortcuts.insert("reload".to_string(), "Ctrl+R".to_string());
    shortcuts.insert("back".to_string(), "Alt+Left".to_string());
    shortcuts.insert("forward".to_string(), "Alt+Right".to_string());
    shortcuts.insert("find".to_string(), "Ctrl+F".to_string());
    shortcuts.insert("devtools".to_string(), "F12".to_string());
    shortcuts.insert("zoom_in".to_string(), "Ctrl+Plus".to_string());
    shortcuts.insert("zoom_out".to_string(), "Ctrl+Minus".to_string());
    shortcuts.insert("zoom_reset".to_string(), "Ctrl+0".to_string());
    shortcuts.insert("next_tab".to_string(), "Ctrl+Tab".to_string());
    shortcuts.insert("prev_tab".to_string(), "Ctrl+Shift+Tab".to_string());
    shortcuts.insert("reopen_tab".to_string(), "Ctrl+Shift+T".to_string());
    shortcuts.insert("preferences".to_string(), "Ctrl+,".to_string());
    shortcuts.insert("dev_center".to_string(), "Ctrl+Shift+I".to_string());
    shortcuts
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
    fn default_preferences() {
        let prefs = Preferences::new();
        assert_eq!(prefs.general.homepage, "about:blank");
        assert!(prefs.privacy.block_trackers);
        assert_eq!(prefs.performance.memory_mode, "Balanced");
    }

    #[test]
    fn validate_fixes_invalid_values() {
        let mut prefs = Preferences::new();
        prefs.appearance.font_size = 0.1; // Too small
        prefs.performance.memory_mode = "Invalid".to_string();
        prefs.appearance.theme = "Invalid".to_string();
        prefs.validate();
        assert_eq!(prefs.appearance.font_size, 0.5);
        assert_eq!(prefs.performance.memory_mode, "Balanced");
        assert_eq!(prefs.appearance.theme, "system");
    }

    #[test]
    fn validate_enforces_freeze_lt_suspend_lt_discard() {
        let mut prefs = Preferences::new();
        prefs.performance.freeze_after_secs = 1000;
        prefs.performance.suspend_after_secs = 500; // Invalid
        prefs.performance.discard_after_secs = 200; // Invalid
        prefs.validate();
        assert!(prefs.performance.freeze_after_secs < prefs.performance.suspend_after_secs);
        assert!(prefs.performance.suspend_after_secs < prefs.performance.discard_after_secs);
    }

    #[test]
    fn shortcuts() {
        let mut prefs = Preferences::new();
        assert_eq!(prefs.get_shortcut("new_tab").unwrap(), "Ctrl+T");
        prefs.set_shortcut("new_tab", "Cmd+T");
        assert_eq!(prefs.get_shortcut("new_tab").unwrap(), "Cmd+T");
    }

    #[test]
    fn json_roundtrip() {
        let prefs = Preferences::new();
        let json = prefs.to_json().unwrap();
        let restored = Preferences::from_json(&json).unwrap();
        assert_eq!(restored.general.homepage, prefs.general.homepage);
        assert_eq!(
            restored.privacy.block_trackers,
            prefs.privacy.block_trackers
        );
    }

    #[test]
    fn custom_settings() {
        let mut prefs = Preferences::new();
        prefs.set_custom("custom_key", "custom_value");
        assert_eq!(prefs.get_custom("custom_key").unwrap(), "custom_value");
    }
}
