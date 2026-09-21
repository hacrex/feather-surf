// FeatherSurf Permission Management
//
// Handles camera, microphone, location, notifications, and other permissions.
// Supports one-time, session, and persistent permission choices.

use std::collections::HashMap;
use std::time::{Duration, SystemTime};

// ── Permission Types ───────────────────────────────────────────────

/// Browser permission types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum PermissionType {
    /// Camera access.
    Camera,
    /// Microphone access.
    Microphone,
    /// Location access.
    Location,
    /// Notifications.
    Notifications,
    /// Clipboard access.
    Clipboard,
    /// MIDI access.
    Midi,
    /// USB device access.
    Usb,
    /// Bluetooth access.
    Bluetooth,
    /// Idle detection.
    IdleDetection,
    /// Periodic background sync.
    PeriodicBackgroundSync,
    /// Storage access.
    StorageAccess,
    /// Screen sharing.
    ScreenSharing,
}

impl PermissionType {
    pub fn display_name(&self) -> &'static str {
        match self {
            PermissionType::Camera => "Camera",
            PermissionType::Microphone => "Microphone",
            PermissionType::Location => "Location",
            PermissionType::Notifications => "Notifications",
            PermissionType::Clipboard => "Clipboard",
            PermissionType::Midi => "MIDI",
            PermissionType::Usb => "USB",
            PermissionType::Bluetooth => "Bluetooth",
            PermissionType::IdleDetection => "Idle Detection",
            PermissionType::PeriodicBackgroundSync => "Background Sync",
            PermissionType::StorageAccess => "Storage Access",
            PermissionType::ScreenSharing => "Screen Sharing",
        }
    }
}

/// Permission grant duration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PermissionDuration {
    /// Grant for this page load only.
    OneTime,
    /// Grant for this session.
    Session,
    /// Grant persistently until revoked.
    Persistent,
}

/// Permission decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PermissionDecision {
    /// Permission granted.
    Granted,
    /// Permission denied.
    Denied,
    /// Permission not yet decided (prompt user).
    Prompt,
}

/// A stored permission.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Permission {
    /// Permission type.
    pub permission_type: PermissionType,
    /// Origin the permission was granted to.
    pub origin: String,
    /// Decision.
    pub decision: PermissionDecision,
    /// Duration.
    pub duration: PermissionDuration,
    /// When granted (None if denied).
    pub granted_at: Option<u64>,
    /// When it expires (None = never for persistent).
    pub expires_at: Option<u64>,
}

/// Permission manager for a profile.
pub struct PermissionManager {
    /// Stored permissions indexed by origin.
    permissions: HashMap<String, Vec<Permission>>,
    /// Default decision for each permission type.
    defaults: HashMap<PermissionType, PermissionDecision>,
    /// Site-specific defaults.
    site_defaults: HashMap<String, HashMap<PermissionType, PermissionDecision>>,
}

impl PermissionManager {
    pub fn new() -> Self {
        let mut defaults = HashMap::new();
        // Default to prompt for most permissions
        defaults.insert(PermissionType::Camera, PermissionDecision::Prompt);
        defaults.insert(PermissionType::Microphone, PermissionDecision::Prompt);
        defaults.insert(PermissionType::Location, PermissionDecision::Prompt);
        defaults.insert(PermissionType::Notifications, PermissionDecision::Prompt);
        defaults.insert(PermissionType::Clipboard, PermissionDecision::Prompt);
        defaults.insert(PermissionType::Midi, PermissionDecision::Prompt);
        defaults.insert(PermissionType::Usb, PermissionDecision::Prompt);
        defaults.insert(PermissionType::Bluetooth, PermissionDecision::Prompt);
        defaults.insert(PermissionType::IdleDetection, PermissionDecision::Prompt);
        defaults.insert(PermissionType::PeriodicBackgroundSync, PermissionDecision::Prompt);
        defaults.insert(PermissionType::StorageAccess, PermissionDecision::Prompt);
        defaults.insert(PermissionType::ScreenSharing, PermissionDecision::Prompt);

        Self {
            permissions: HashMap::new(),
            defaults,
            site_defaults: HashMap::new(),
        }
    }

    /// Check if a permission is granted.
    pub fn is_granted(&self, origin: &str, permission_type: PermissionType) -> bool {
        if let Some(permissions) = self.permissions.get(origin) {
            if let Some(permission) = permissions.iter().find(|p| p.permission_type == permission_type) {
                return permission.decision == PermissionDecision::Granted
                    && !self.is_expired(permission);
            }
        }
        false
    }

    /// Check if a permission is denied.
    pub fn is_denied(&self, origin: &str, permission_type: PermissionType) -> bool {
        if let Some(permissions) = self.permissions.get(origin) {
            if let Some(permission) = permissions.iter().find(|p| p.permission_type == permission_type) {
                return permission.decision == PermissionDecision::Denied;
            }
        }
        false
    }

    /// Get permission decision for a request.
    pub fn get_decision(&self, origin: &str, permission_type: PermissionType) -> PermissionDecision {
        // Check stored permissions first
        if let Some(permissions) = self.permissions.get(origin) {
            if let Some(permission) = permissions.iter().find(|p| p.permission_type == permission_type) {
                if self.is_expired(permission) {
                    // Expired, fall through to default
                } else {
                    return permission.decision;
                }
            }
        }

        // Check site-specific defaults
        if let Some(site_defaults) = self.site_defaults.get(origin) {
            if let Some(&decision) = site_defaults.get(&permission_type) {
                return decision;
            }
        }

        // Fall back to global default
        self.defaults
            .get(&permission_type)
            .copied()
            .unwrap_or(PermissionDecision::Prompt)
    }

    /// Grant a permission.
    pub fn grant(
        &mut self,
        origin: impl Into<String>,
        permission_type: PermissionType,
        duration: PermissionDuration,
    ) {
        let now = now_secs();
        let expires_at = match duration {
            PermissionDuration::OneTime => Some(now + 3600), // 1 hour
            PermissionDuration::Session => Some(now + 86400), // 24 hours
            PermissionDuration::Persistent => None,
        };

        let permission = Permission {
            permission_type,
            origin: origin.into(),
            decision: PermissionDecision::Granted,
            duration,
            granted_at: Some(now),
            expires_at,
        };

        let origin = permission.origin.clone();
        let permissions = self.permissions.entry(origin).or_default();

        // Update existing or add new
        if let Some(existing) = permissions.iter_mut().find(|p| p.permission_type == permission_type) {
            *existing = permission;
        } else {
            permissions.push(permission);
        }
    }

    /// Deny a permission.
    pub fn deny(&mut self, origin: impl Into<String>, permission_type: PermissionType) {
        let permission = Permission {
            permission_type,
            origin: origin.into(),
            decision: PermissionDecision::Denied,
            duration: PermissionDuration::Persistent,
            granted_at: None,
            expires_at: None,
        };

        let origin = permission.origin.clone();
        let permissions = self.permissions.entry(origin).or_default();

        if let Some(existing) = permissions.iter_mut().find(|p| p.permission_type == permission_type) {
            *existing = permission;
        } else {
            permissions.push(permission);
        }
    }

    /// Reset a permission (revert to prompt).
    pub fn reset(&mut self, origin: &str, permission_type: PermissionType) -> bool {
        if let Some(permissions) = self.permissions.get_mut(origin) {
            let len = permissions.len();
            permissions.retain(|p| p.permission_type != permission_type);
            permissions.len() < len
        } else {
            false
        }
    }

    /// Reset all permissions for an origin.
    pub fn reset_origin(&mut self, origin: &str) -> usize {
        if let Some(permissions) = self.permissions.remove(origin) {
            permissions.len()
        } else {
            0
        }
    }

    /// Reset all permissions.
    pub fn reset_all(&mut self) {
        self.permissions.clear();
    }

    /// Get all permissions for an origin.
    pub fn permissions_for_origin(&self, origin: &str) -> Vec<&Permission> {
        self.permissions.get(origin).map(|p| p.as_ref()).unwrap_or_default()
    }

    /// Get all granted permissions.
    pub fn granted_permissions(&self) -> Vec<&Permission> {
        self.permissions
            .values()
            .flat_map(|p| p.iter())
            .filter(|p| p.decision == PermissionDecision::Granted)
            .collect()
    }

    /// Set default decision for a permission type.
    pub fn set_default(&mut self, permission_type: PermissionType, decision: PermissionDecision) {
        self.defaults.insert(permission_type, decision);
    }

    /// Set site-specific default.
    pub fn set_site_default(
        &mut self,
        origin: impl Into<String>,
        permission_type: PermissionType,
        decision: PermissionDecision,
    ) {
        self.site_defaults
            .entry(origin.into())
            .or_default()
            .insert(permission_type, decision);
    }

    /// Check if a permission is expired.
    fn is_expired(&self, permission: &Permission) -> bool {
        if let Some(expires_at) = permission.expires_at {
            now_secs() > expires_at
        } else {
            false
        }
    }

    /// Get permission count.
    pub fn count(&self) -> usize {
        self.permissions.values().map(|p| p.len()).sum()
    }

    /// Get granted count.
    pub fn granted_count(&self) -> usize {
        self.granted_permissions().len()
    }
}

impl Default for PermissionManager {
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
    fn default_is_prompt() {
        let manager = PermissionManager::new();
        assert_eq!(
            manager.get_decision("https://example.com", PermissionType::Camera),
            PermissionDecision::Prompt
        );
    }

    #[test]
    fn grant_persistent() {
        let mut manager = PermissionManager::new();
        manager.grant(
            "https://example.com",
            PermissionType::Camera,
            PermissionDuration::Persistent,
        );
        assert!(manager.is_granted("https://example.com", PermissionType::Camera));
    }

    #[test]
    fn deny_permission() {
        let mut manager = PermissionManager::new();
        manager.deny("https://example.com", PermissionType::Camera);
        assert!(manager.is_denied("https://example.com", PermissionType::Camera));
        assert!(!manager.is_granted("https://example.com", PermissionType::Camera));
    }

    #[test]
    fn reset_permission() {
        let mut manager = PermissionManager::new();
        manager.grant(
            "https://example.com",
            PermissionType::Camera,
            PermissionDuration::Persistent,
        );
        assert!(manager.is_granted("https://example.com", PermissionType::Camera));

        manager.reset("https://example.com", PermissionType::Camera);
        assert!(!manager.is_granted("https://example.com", PermissionType::Camera));
        assert_eq!(
            manager.get_decision("https://example.com", PermissionType::Camera),
            PermissionDecision::Prompt
        );
    }

    #[test]
    fn site_specific_default() {
        let mut manager = PermissionManager::new();
        manager.set_site_default(
            "https://example.com",
            PermissionType::Camera,
            PermissionDecision::Denied,
        );

        assert!(manager.is_denied("https://example.com", PermissionType::Camera));
        assert_eq!(
            manager.get_decision("https://other.com", PermissionType::Camera),
            PermissionDecision::Prompt
        );
    }

    #[test]
    fn permissions_for_origin() {
        let mut manager = PermissionManager::new();
        manager.grant(
            "https://example.com",
            PermissionType::Camera,
            PermissionDuration::Persistent,
        );
        manager.grant(
            "https://example.com",
            PermissionType::Microphone,
            PermissionDuration::Persistent,
        );

        let permissions = manager.permissions_for_origin("https://example.com");
        assert_eq!(permissions.len(), 2);
    }

    #[test]
    fn granted_count() {
        let mut manager = PermissionManager::new();
        manager.grant("https://a.com", PermissionType::Camera, PermissionDuration::Persistent);
        manager.grant("https://b.com", PermissionType::Camera, PermissionDuration::Persistent);
        manager.deny("https://c.com", PermissionType::Camera);

        assert_eq!(manager.granted_count(), 2);
    }
}
