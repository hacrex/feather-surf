// FeatherSurf Update Manifest
//
// Signed update manifest for secure auto-updates.

use std::collections::HashMap;
use std::time::SystemTime;

// ── Update Manifest ────────────────────────────────────────────────

/// Signed update manifest.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpdateManifest {
    /// Manifest version.
    pub manifest_version: u32,
    /// Application version.
    pub version: String,
    /// Release channel.
    pub channel: ReleaseChannel,
    /// When this manifest was created.
    pub created_at: u64,
    /// Minimum required version for upgrade.
    pub min_version: String,
    /// Platform-specific updates.
    pub platforms: HashMap<String, PlatformUpdate>,
    /// Release notes.
    pub release_notes: String,
    /// Signature of the manifest (base64).
    pub signature: String,
    /// Public key ID used for signing.
    pub key_id: String,
}

/// Release channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ReleaseChannel {
    Nightly,
    Beta,
    Stable,
}

/// Platform-specific update.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlatformUpdate {
    /// Download URL.
    pub url: String,
    /// File size in bytes.
    pub size: u64,
    /// SHA-256 checksum.
    pub sha256: String,
    /// Signature (base64).
    pub signature: String,
    /// File type (msi, deb, dmg, etc.).
    pub file_type: String,
    /// Minimum OS version required.
    pub min_os_version: String,
    /// Whether this is a mandatory update.
    pub mandatory: bool,
    /// Staged rollout percentage (0-100).
    pub rollout_percentage: u8,
}

impl UpdateManifest {
    /// Create a new manifest.
    pub fn new(version: impl Into<String>, channel: ReleaseChannel) -> Self {
        Self {
            manifest_version: 1,
            version: version.into(),
            channel,
            created_at: now_secs(),
            min_version: "0.0.1".to_string(),
            platforms: HashMap::new(),
            release_notes: String::new(),
            signature: String::new(),
            key_id: String::new(),
        }
    }

    /// Add a platform update.
    pub fn add_platform(&mut self, platform: impl Into<String>, update: PlatformUpdate) {
        self.platforms.insert(platform.into(), update);
    }

    /// Get update for a specific platform.
    pub fn get_platform(&self, platform: &str) -> Option<&PlatformUpdate> {
        self.platforms.get(platform)
    }

    /// Check if an update is available.
    pub fn is_update_available(&self, current_version: &str) -> bool {
        compare_versions(&self.version, current_version) > 0
    }

    /// Check if an update is mandatory.
    pub fn is_mandatory(&self, platform: &str) -> bool {
        self.platforms
            .get(platform)
            .map(|p| p.mandatory)
            .unwrap_or(false)
    }

    /// Check if an update is in staged rollout.
    pub fn is_in_rollout(&self, platform: &str, user_id: &str) -> bool {
        if let Some(update) = self.platforms.get(platform) {
            if update.rollout_percentage == 100 {
                return true;
            }
            // Simple hash-based rollout
            let hash = simple_hash(&format!("{}-{}", self.version, user_id));
            (hash % 100) < update.rollout_percentage as u32
        } else {
            false
        }
    }

    /// Serialize to JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize from JSON.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

// ── Update Checker ─────────────────────────────────────────────────

/// Checks for updates.
pub struct UpdateChecker {
    /// Current version.
    current_version: String,
    /// User ID for staged rollout.
    user_id: String,
    /// Last checked timestamp.
    last_checked: u64,
    /// Check interval in seconds.
    check_interval: u64,
    /// Cached manifest.
    cached_manifest: Option<UpdateManifest>,
}

impl UpdateChecker {
    pub fn new(current_version: impl Into<String>, user_id: impl Into<String>) -> Self {
        Self {
            current_version: current_version.into(),
            user_id: user_id.into(),
            last_checked: 0,
            check_interval: 86400, // 24 hours
            cached_manifest: None,
        }
    }

    /// Check if it's time to check for updates.
    pub fn should_check(&self) -> bool {
        now_secs() - self.last_checked >= self.check_interval
    }

    /// Process an update manifest.
    pub fn process_manifest(&mut self, manifest: UpdateManifest) -> UpdateResult {
        self.last_checked = now_secs();
        self.cached_manifest = Some(manifest.clone());

        // Check if update is available
        if !manifest.is_update_available(&self.current_version) {
            return UpdateResult::UpToDate;
        }

        // Get platform update
        let platform = std::env::consts::OS;
        let platform_update = match manifest.get_platform(platform) {
            Some(update) => update,
            None => return UpdateResult::NoUpdateForPlatform,
        };

        // Check if in staged rollout
        if !manifest.is_in_rollout(platform, &self.user_id) {
            return UpdateResult::NotInRollout;
        }

        // Check if mandatory
        if platform_update.mandatory {
            return UpdateResult::MandatoryUpdate {
                version: manifest.version,
                url: platform_update.url.clone(),
                size: platform_update.size,
                release_notes: manifest.release_notes.clone(),
            };
        }

        UpdateResult::OptionalUpdate {
            version: manifest.version,
            url: platform_update.url.clone(),
            size: platform_update.size,
            release_notes: manifest.release_notes.clone(),
        }
    }

    /// Get cached manifest.
    pub fn cached_manifest(&self) -> Option<&UpdateManifest> {
        self.cached_manifest.as_ref()
    }
}

impl Default for UpdateChecker {
    fn default() -> Self {
        Self::new("0.0.1", "default-user")
    }
}

// ── Update Result ──────────────────────────────────────────────────

/// Result of an update check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateResult {
    /// Already on latest version.
    UpToDate,
    /// No update available for this platform.
    NoUpdateForPlatform,
    /// Not in staged rollout.
    NotInRollout,
    /// Optional update available.
    OptionalUpdate {
        version: String,
        url: String,
        size: u64,
        release_notes: String,
    },
    /// Mandatory update required.
    MandatoryUpdate {
        version: String,
        url: String,
        size: u64,
        release_notes: String,
    },
}

// ── Helpers ────────────────────────────────────────────────────────

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Compare two semver versions.
/// Returns positive if a > b, negative if a < b, 0 if equal.
fn compare_versions(a: &str, b: &str) -> i32 {
    let a_parts: Vec<u32> = a.split('.').filter_map(|s| s.parse().ok()).collect();
    let b_parts: Vec<u32> = b.split('.').filter_map(|s| s.parse().ok()).collect();

    for i in 0..3 {
        let a_val = a_parts.get(i).copied().unwrap_or(0);
        let b_val = b_parts.get(i).copied().unwrap_or(0);
        if a_val != b_val {
            return (a_val as i32) - (b_val as i32);
        }
    }
    0
}

/// Simple hash function.
fn simple_hash(input: &str) -> u32 {
    let mut hash: u32 = 5381;
    for byte in input.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(byte as u32);
    }
    hash
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_comparison() {
        assert!(compare_versions("1.0.0", "0.9.9") > 0);
        assert!(compare_versions("0.9.9", "1.0.0") < 0);
        assert_eq!(compare_versions("1.0.0", "1.0.0"), 0);
        assert!(compare_versions("1.0.1", "1.0.0") > 0);
        assert!(compare_versions("1.1.0", "1.0.9") > 0);
    }

    #[test]
    fn update_available() {
        let mut manifest = UpdateManifest::new("1.0.0", ReleaseChannel::Stable);
        manifest.add_platform("windows", PlatformUpdate {
            url: "https://example.com/update.msi".to_string(),
            size: 50_000_000,
            sha256: "abc123".to_string(),
            signature: "sig".to_string(),
            file_type: "msi".to_string(),
            min_os_version: "10.0.19041".to_string(),
            mandatory: false,
            rollout_percentage: 100,
        });

        assert!(manifest.is_update_available("0.9.9"));
        assert!(!manifest.is_update_available("1.0.0"));
        assert!(!manifest.is_update_available("1.0.1"));
    }

    #[test]
    fn staged_rollout() {
        let mut manifest = UpdateManifest::new("1.0.0", ReleaseChannel::Stable);
        manifest.add_platform("windows", PlatformUpdate {
            url: "https://example.com/update.msi".to_string(),
            size: 50_000_000,
            sha256: "abc123".to_string(),
            signature: "sig".to_string(),
            file_type: "msi".to_string(),
            min_os_version: "10.0.19041".to_string(),
            mandatory: false,
            rollout_percentage: 10,
        });

        // Some users will be in rollout, some won't
        let mut in_rollout = 0;
        for i in 0..100 {
            let user_id = format!("user-{}", i);
            if manifest.is_in_rollout("windows", &user_id) {
                in_rollout += 1;
            }
        }
        // Should be approximately 10% (within reasonable variance)
        assert!(in_rollout > 0 && in_rollout < 30);
    }

    #[test]
    fn update_checker() {
        let mut checker = UpdateChecker::new("0.9.9", "test-user");
        assert!(checker.should_check());

        let mut manifest = UpdateManifest::new("1.0.0", ReleaseChannel::Stable);
        manifest.add_platform("windows", PlatformUpdate {
            url: "https://example.com/update.msi".to_string(),
            size: 50_000_000,
            sha256: "abc123".to_string(),
            signature: "sig".to_string(),
            file_type: "msi".to_string(),
            min_os_version: "10.0.19041".to_string(),
            mandatory: false,
            rollout_percentage: 100,
        });

        let result = checker.process_manifest(manifest);
        assert!(matches!(result, UpdateResult::OptionalUpdate { .. }));
    }

    #[test]
    fn manifest_json_roundtrip() {
        let mut manifest = UpdateManifest::new("1.0.0", ReleaseChannel::Stable);
        manifest.add_platform("linux", PlatformUpdate {
            url: "https://example.com/update.deb".to_string(),
            size: 40_000_000,
            sha256: "def456".to_string(),
            signature: "sig".to_string(),
            file_type: "deb".to_string(),
            min_os_version: "22.04".to_string(),
            mandatory: false,
            rollout_percentage: 100,
        });

        let json = manifest.to_json().unwrap();
        let restored = UpdateManifest::from_json(&json).unwrap();
        assert_eq!(restored.version, "1.0.0");
        assert!(restored.get_platform("linux").is_some());
    }
}
