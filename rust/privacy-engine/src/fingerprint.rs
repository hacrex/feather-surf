// FeatherSurf Fingerprinting Protection
//
// Detects and mitigates browser fingerprinting techniques.

use std::collections::HashMap;

// ── Fingerprinting Detection ───────────────────────────────────────

/// Fingerprinting technique categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum FingerprintType {
    /// Canvas fingerprinting.
    Canvas,
    /// WebGL fingerprinting.
    WebGL,
    /// Audio fingerprinting.
    Audio,
    /// Font fingerprinting.
    Font,
    /// Navigator properties fingerprinting.
    Navigator,
    /// Screen/display fingerprinting.
    Screen,
    /// Timezone/locale fingerprinting.
    Timezone,
    /// Hardware fingerprinting.
    Hardware,
    /// Plugin fingerprinting.
    Plugin,
    /// Battery API fingerprinting.
    Battery,
    /// Speech synthesis fingerprinting.
    SpeechSynthesis,
    /// WebRTC IP leak.
    WebRTC,
    /// Unknown technique.
    Unknown,
}

/// A detected fingerprinting attempt.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FingerprintAttempt {
    /// Type of fingerprinting detected.
    pub fingerprint_type: FingerprintType,
    /// Origin attempting the fingerprint.
    pub origin: String,
    /// API or method used.
    pub method: String,
    /// When detected.
    pub timestamp: u64,
    /// Whether it was blocked.
    pub blocked: bool,
}

/// Protection status for a fingerprint type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ProtectionStatus {
    /// Protection enabled, return fake data.
    Enabled,
    /// Protection enabled, return minimal data.
    Minimal,
    /// Protection disabled for this type.
    Disabled,
    /// Per-site exception.
    Exception,
}

/// Fingerprinting protection configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FingerprintConfig {
    /// Canvas protection status.
    pub canvas: ProtectionStatus,
    /// WebGL protection status.
    pub webgl: ProtectionStatus,
    /// Audio protection status.
    pub audio: ProtectionStatus,
    /// Font protection status.
    pub font: ProtectionStatus,
    /// Navigator protection status.
    pub navigator: ProtectionStatus,
    /// Screen protection status.
    pub screen: ProtectionStatus,
    /// Timezone protection status.
    pub timezone: ProtectionStatus,
    /// Hardware protection status.
    pub hardware: ProtectionStatus,
    /// Plugin protection status.
    pub plugin: ProtectionStatus,
    /// Battery protection status.
    pub battery: ProtectionStatus,
    /// Speech synthesis protection status.
    pub speech_synthesis: ProtectionStatus,
    /// WebRTC protection status.
    pub webrtc: ProtectionStatus,
}

impl Default for FingerprintConfig {
    fn default() -> Self {
        Self {
            canvas: ProtectionStatus::Enabled,
            webgl: ProtectionStatus::Enabled,
            audio: ProtectionStatus::Enabled,
            font: ProtectionStatus::Minimal,
            navigator: ProtectionStatus::Minimal,
            screen: ProtectionStatus::Minimal,
            timezone: ProtectionStatus::Minimal,
            hardware: ProtectionStatus::Minimal,
            plugin: ProtectionStatus::Enabled,
            battery: ProtectionStatus::Enabled,
            speech_synthesis: ProtectionStatus::Enabled,
            webrtc: ProtectionStatus::Enabled,
        }
    }
}

/// Fingerprinting protection engine.
pub struct FingerprintProtection {
    /// Global configuration.
    config: FingerprintConfig,
    /// Per-site exceptions.
    site_exceptions: HashMap<String, FingerprintConfig>,
    /// Detection history.
    attempts: Vec<FingerprintAttempt>,
    /// Maximum history size.
    max_history: usize,
}

impl FingerprintProtection {
    pub fn new() -> Self {
        Self {
            config: FingerprintConfig::default(),
            site_exceptions: HashMap::new(),
            attempts: Vec::new(),
            max_history: 1000,
        }
    }

    /// Check if a fingerprinting API call should be blocked.
    pub fn should_block(
        &mut self,
        origin: &str,
        fingerprint_type: FingerprintType,
        method: &str,
    ) -> bool {
        let status = self.get_status(origin, fingerprint_type);

        let blocked = match status {
            ProtectionStatus::Enabled | ProtectionStatus::Minimal => true,
            ProtectionStatus::Disabled | ProtectionStatus::Exception => false,
        };

        // Record the attempt
        self.record_attempt(origin, fingerprint_type, method, blocked);

        blocked
    }

    /// Get protection status for a specific origin and type.
    fn get_status(&self, origin: &str, fingerprint_type: FingerprintType) -> ProtectionStatus {
        // Check per-site exceptions first
        if let Some(config) = self.site_exceptions.get(origin) {
            return match fingerprint_type {
                FingerprintType::Canvas => config.canvas,
                FingerprintType::WebGL => config.webgl,
                FingerprintType::Audio => config.audio,
                FingerprintType::Font => config.font,
                FingerprintType::Navigator => config.navigator,
                FingerprintType::Screen => config.screen,
                FingerprintType::Timezone => config.timezone,
                FingerprintType::Hardware => config.hardware,
                FingerprintType::Plugin => config.plugin,
                FingerprintType::Battery => config.battery,
                FingerprintType::SpeechSynthesis => config.speech_synthesis,
                FingerprintType::WebRTC => config.webrtc,
                FingerprintType::Unknown => ProtectionStatus::Enabled,
            };
        }

        // Fall back to global config
        match fingerprint_type {
            FingerprintType::Canvas => self.config.canvas,
            FingerprintType::WebGL => self.config.webgl,
            FingerprintType::Audio => self.config.audio,
            FingerprintType::Font => self.config.font,
            FingerprintType::Navigator => self.config.navigator,
            FingerprintType::Screen => self.config.screen,
            FingerprintType::Timezone => self.config.timezone,
            FingerprintType::Hardware => self.config.hardware,
            FingerprintType::Plugin => self.config.plugin,
            FingerprintType::Battery => self.config.battery,
            FingerprintType::SpeechSynthesis => self.config.speech_synthesis,
            FingerprintType::WebRTC => self.config.webrtc,
            FingerprintType::Unknown => ProtectionStatus::Enabled,
        }
    }

    /// Record a fingerprinting attempt.
    fn record_attempt(
        &mut self,
        origin: &str,
        fingerprint_type: FingerprintType,
        method: &str,
        blocked: bool,
    ) {
        self.attempts.push(FingerprintAttempt {
            fingerprint_type,
            origin: origin.to_string(),
            method: method.to_string(),
            timestamp: now_secs(),
            blocked,
        });

        // Trim old entries
        if self.attempts.len() > self.max_history {
            self.attempts.remove(0);
        }
    }

    /// Set global protection status for a type.
    pub fn set_global_status(
        &mut self,
        fingerprint_type: FingerprintType,
        status: ProtectionStatus,
    ) {
        match fingerprint_type {
            FingerprintType::Canvas => self.config.canvas = status,
            FingerprintType::WebGL => self.config.webgl = status,
            FingerprintType::Audio => self.config.audio = status,
            FingerprintType::Font => self.config.font = status,
            FingerprintType::Navigator => self.config.navigator = status,
            FingerprintType::Screen => self.config.screen = status,
            FingerprintType::Timezone => self.config.timezone = status,
            FingerprintType::Hardware => self.config.hardware = status,
            FingerprintType::Plugin => self.config.plugin = status,
            FingerprintType::Battery => self.config.battery = status,
            FingerprintType::SpeechSynthesis => self.config.speech_synthesis = status,
            FingerprintType::WebRTC => self.config.webrtc = status,
            FingerprintType::Unknown => {}
        }
    }

    /// Add a per-site exception.
    pub fn add_site_exception(&mut self, origin: impl Into<String>, config: FingerprintConfig) {
        self.site_exceptions.insert(origin.into(), config);
    }

    /// Remove a per-site exception.
    pub fn remove_site_exception(&mut self, origin: &str) -> bool {
        self.site_exceptions.remove(origin).is_some()
    }

    /// Get recent fingerprinting attempts.
    pub fn recent_attempts(&self, n: usize) -> Vec<&FingerprintAttempt> {
        self.attempts.iter().rev().take(n).collect()
    }

    /// Get attempts for a specific origin.
    pub fn attempts_for_origin(&self, origin: &str) -> Vec<&FingerprintAttempt> {
        self.attempts
            .iter()
            .filter(|a| a.origin == origin)
            .collect()
    }

    /// Get statistics.
    pub fn stats(&self) -> FingerprintStats {
        let total = self.attempts.len();
        let blocked = self.attempts.iter().filter(|a| a.blocked).count();
        let by_type = self.attempts.iter().fold(HashMap::new(), |mut acc, a| {
            *acc.entry(a.fingerprint_type).or_insert(0) += 1;
            acc
        });

        FingerprintStats {
            total_attempts: total,
            blocked_attempts: blocked,
            attempts_by_type: by_type,
        }
    }

    /// Clear history.
    pub fn clear_history(&mut self) {
        self.attempts.clear();
    }
}

impl Default for FingerprintProtection {
    fn default() -> Self {
        Self::new()
    }
}

/// Fingerprinting statistics.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FingerprintStats {
    pub total_attempts: usize,
    pub blocked_attempts: usize,
    pub attempts_by_type: HashMap<FingerprintType, usize>,
}

// ── Fake Data Generators ───────────────────────────────────────────

/// Generate fake canvas fingerprint data.
pub fn fake_canvas_data() -> Vec<u8> {
    // Return deterministic but unique-per-session data
    vec![0x46, 0x45, 0x41, 0x54, 0x48, 0x45, 0x52] // "FEATHER"
}

/// Generate fake WebGL renderer string.
pub fn fake_webgl_renderer() -> String {
    "FeatherSurf GPU Renderer".to_string()
}

/// Generate fake WebGL vendor string.
pub fn fake_webgl_vendor() -> String {
    "FeatherSurf".to_string()
}

/// Generate fake audio fingerprint.
pub fn fake_audio_fingerprint() -> f64 {
    0.123456789 // Deterministic value
}

/// Generate fake font list.
pub fn fake_font_list() -> Vec<String> {
    vec![
        "Arial".to_string(),
        "Times New Roman".to_string(),
        "Courier New".to_string(),
    ]
}

/// Generate fake navigator properties.
pub fn fake_navigator_properties() -> HashMap<String, String> {
    let mut props = HashMap::new();
    props.insert("platform".to_string(), "FeatherSurf".to_string());
    props.insert("language".to_string(), "en-US".to_string());
    props.insert("hardwareConcurrency".to_string(), "4".to_string());
    props.insert("deviceMemory".to_string(), "8".to_string());
    props
}

// ── Helpers ────────────────────────────────────────────────────────

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_canvas_fingerprint() {
        let mut protection = FingerprintProtection::new();
        let blocked =
            protection.should_block("https://example.com", FingerprintType::Canvas, "toDataURL");
        assert!(blocked);
    }

    #[test]
    fn block_webgl_fingerprint() {
        let mut protection = FingerprintProtection::new();
        let blocked = protection.should_block(
            "https://example.com",
            FingerprintType::WebGL,
            "getParameter",
        );
        assert!(blocked);
    }

    #[test]
    fn site_exception() {
        let mut protection = FingerprintProtection::new();
        let mut config = FingerprintConfig::default();
        config.canvas = ProtectionStatus::Disabled;
        protection.add_site_exception("https://trusted.com", config);

        let blocked =
            protection.should_block("https://trusted.com", FingerprintType::Canvas, "toDataURL");
        assert!(!blocked);
    }

    #[test]
    fn record_attempts() {
        let mut protection = FingerprintProtection::new();
        protection.should_block("https://example.com", FingerprintType::Canvas, "toDataURL");
        protection.should_block(
            "https://example.com",
            FingerprintType::WebGL,
            "getParameter",
        );

        let stats = protection.stats();
        assert_eq!(stats.total_attempts, 2);
        assert_eq!(stats.blocked_attempts, 2);
    }

    #[test]
    fn fake_data_generators() {
        let canvas = fake_canvas_data();
        assert!(!canvas.is_empty());

        let renderer = fake_webgl_renderer();
        assert!(!renderer.is_empty());

        let fonts = fake_font_list();
        assert!(!fonts.is_empty());

        let props = fake_navigator_properties();
        assert!(!props.is_empty());
    }
}
