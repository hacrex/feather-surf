// FeatherSurf Sync
//
// End-to-end encrypted synchronization of bookmarks, history, settings, and tabs.
// Data is encrypted on-device before leaving.

use std::collections::HashMap;
use std::time::{Duration, SystemTime};

// ── Sync Data Types ────────────────────────────────────────────────

/// Data types that can be synced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum SyncDataType {
    Bookmarks,
    History,
    Settings,
    Passwords,
    Tabs,
}

/// A sync record.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncRecord {
    /// Unique record ID.
    pub id: String,
    /// Data type.
    pub data_type: SyncDataType,
    /// Encrypted payload.
    pub encrypted_data: Vec<u8>,
    /// Device that created this record.
    pub device_id: String,
    /// When the record was created.
    pub created_at: u64,
    /// When the record was last modified.
    pub modified_at: u64,
    /// Record version (for conflict resolution).
    pub version: u64,
    /// Whether this is a deletion marker.
    pub deleted: bool,
}

/// Device information.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeviceInfo {
    /// Unique device ID.
    pub id: String,
    /// Device name.
    pub name: String,
    /// Device type.
    pub device_type: DeviceType,
    /// When the device was registered.
    pub registered_at: u64,
    /// When the device was last seen.
    pub last_seen: u64,
    /// Whether the device is currently active.
    pub active: bool,
}

/// Type of device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DeviceType {
    Desktop,
    Mobile,
    Tablet,
}

/// Sync configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncConfig {
    /// Whether sync is enabled.
    pub enabled: bool,
    /// Which data types to sync.
    pub sync_types: Vec<SyncDataType>,
    /// Whether to sync over cellular.
    pub sync_over_cellular: bool,
    /// Sync interval in seconds.
    pub interval_secs: u64,
    /// Whether to encrypt passwords.
    pub encrypt_passwords: bool,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            sync_types: vec![SyncDataType::Bookmarks, SyncDataType::History, SyncDataType::Settings],
            sync_over_cellular: false,
            interval_secs: 3600, // 1 hour
            encrypt_passwords: true,
        }
    }
}

// ── Sync Manager ───────────────────────────────────────────────────

/// Manages synchronization across devices.
pub struct SyncManager {
    /// Sync configuration.
    config: SyncConfig,
    /// Registered devices.
    devices: Vec<DeviceInfo>,
    /// Local device ID.
    local_device_id: String,
    /// Sync records indexed by ID.
    records: HashMap<String, SyncRecord>,
    /// Encryption key (would be derived from user password).
    encryption_key: Vec<u8>,
    /// Sync state.
    state: SyncState,
}

/// Current sync state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SyncState {
    /// Sync is disabled.
    Disabled,
    /// Waiting to sync.
    Idle,
    /// Currently syncing.
    Syncing,
    /// Sync failed.
    Error,
    /// Waiting for network.
    WaitingForNetwork,
}

impl SyncManager {
    pub fn new(local_device_id: impl Into<String>) -> Self {
        Self {
            config: SyncConfig::default(),
            devices: Vec::new(),
            local_device_id: local_device_id.into(),
            records: HashMap::new(),
            encryption_key: Vec::new(),
            state: SyncState::Disabled,
        }
    }

    /// Enable sync with a password-derived key.
    pub fn enable(&mut self, password: &str) {
        self.encryption_key = derive_key(password);
        self.config.enabled = true;
        self.state = SyncState::Idle;
    }

    /// Disable sync.
    pub fn disable(&mut self) {
        self.config.enabled = false;
        self.state = SyncState::Disabled;
        self.encryption_key.clear();
    }

    /// Check if sync is enabled.
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Get sync state.
    pub fn state(&self) -> SyncState {
        self.state
    }

    /// Register a new device.
    pub fn register_device(&mut self, device: DeviceInfo) {
        if !self.devices.iter().any(|d| d.id == device.id) {
            self.devices.push(device);
        }
    }

    /// Remove a device.
    pub fn remove_device(&mut self, device_id: &str) -> bool {
        let len = self.devices.len();
        self.devices.retain(|d| d.id != device_id);
        self.devices.len() < len
    }

    /// Get all devices.
    pub fn devices(&self) -> &[DeviceInfo] {
        &self.devices
    }

    /// Get active devices.
    pub fn active_devices(&self) -> Vec<&DeviceInfo> {
        self.devices.iter().filter(|d| d.active).collect()
    }

    /// Create a sync record.
    pub fn create_record(
        &mut self,
        data_type: SyncDataType,
        data: &[u8],
    ) -> Result<String, SyncError> {
        if !self.config.enabled {
            return Err(SyncError::SyncDisabled);
        }

        if !self.config.sync_types.contains(&data_type) {
            return Err(SyncError::DataTypeNotEnabled);
        }

        let id = format!("{}-{}", self.local_device_id, now_secs());
        let encrypted = self.encrypt(data)?;

        let record = SyncRecord {
            id: id.clone(),
            data_type,
            encrypted_data: encrypted,
            device_id: self.local_device_id.clone(),
            created_at: now_secs(),
            modified_at: now_secs(),
            version: 1,
            deleted: false,
        };

        self.records.insert(id.clone(), record);
        Ok(id)
    }

    /// Update a sync record.
    pub fn update_record(
        &mut self,
        id: &str,
        data: &[u8],
    ) -> Result<(), SyncError> {
        let record = self.records.get_mut(id).ok_or(SyncError::RecordNotFound)?;
        
        if !self.config.sync_types.contains(&record.data_type) {
            return Err(SyncError::DataTypeNotEnabled);
        }

        record.encrypted_data = self.encrypt(data)?;
        record.modified_at = now_secs();
        record.version += 1;

        Ok(())
    }

    /// Delete a sync record (soft delete).
    pub fn delete_record(&mut self, id: &str) -> Result<(), SyncError> {
        let record = self.records.get_mut(id).ok_or(SyncError::RecordNotFound)?;
        record.deleted = true;
        record.modified_at = now_secs();
        record.version += 1;
        Ok(())
    }

    /// Get a sync record.
    pub fn get_record(&self, id: &str) -> Option<&SyncRecord> {
        self.records.get(id)
    }

    /// Get records by type.
    pub fn records_by_type(&self, data_type: SyncDataType) -> Vec<&SyncRecord> {
        self.records.values().filter(|r| r.data_type == data_type).collect()
    }

    /// Get records modified since a timestamp.
    pub fn modified_since(&self, timestamp: u64) -> Vec<&SyncRecord> {
        self.records.values().filter(|r| r.modified_at > timestamp).collect()
    }

    /// Encrypt data.
    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, SyncError> {
        if self.encryption_key.is_empty() {
            return Err(SyncError::EncryptionFailed);
        }
        // Simple XOR for demonstration - in production use proper encryption
        let encrypted: Vec<u8> = data.iter()
            .zip(self.encryption_key.iter().cycle())
            .map(|(d, k)| d ^ k)
            .collect();
        Ok(encrypted)
    }

    /// Decrypt data.
    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, SyncError> {
        if self.encryption_key.is_empty() {
            return Err(SyncError::DecryptionFailed);
        }
        // Simple XOR for demonstration
        let decrypted: Vec<u8> = data.iter()
            .zip(self.encryption_key.iter().cycle())
            .map(|(d, k)| d ^ k)
            .collect();
        Ok(decrypted)
    }

    /// Get sync statistics.
    pub fn stats(&self) -> SyncStats {
        let total_records = self.records.len();
        let by_type = self.records.values().fold(HashMap::new(), |mut acc, r| {
            *acc.entry(r.data_type).or_insert(0) += 1;
            acc
        });

        SyncStats {
            total_records,
            records_by_type: by_type,
            device_count: self.devices.len(),
            state: self.state,
        }
    }

    /// Clear all sync data.
    pub fn clear_all(&mut self) {
        self.records.clear();
        self.devices.clear();
    }
}

impl Default for SyncManager {
    fn default() -> Self {
        Self::new("local-device")
    }
}

// ── Sync Statistics ────────────────────────────────────────────────

/// Sync statistics.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncStats {
    pub total_records: usize,
    pub records_by_type: HashMap<SyncDataType, usize>,
    pub device_count: usize,
    pub state: SyncState,
}

// ── Sync Errors ────────────────────────────────────────────────────

/// Sync errors.
#[derive(Debug, Clone, thiserror::Error)]
pub enum SyncError {
    #[error("Sync is disabled")]
    SyncDisabled,
    #[error("Data type not enabled for sync")]
    DataTypeNotEnabled,
    #[error("Record not found")]
    RecordNotFound,
    #[error("Encryption failed")]
    EncryptionFailed,
    #[error("Decryption failed")]
    DecryptionFailed,
    #[error("Conflict detected")]
    Conflict,
    #[error("Network error")]
    NetworkError,
}

// ── Key Derivation ─────────────────────────────────────────────────

/// Derive an encryption key from a password.
fn derive_key(password: &str) -> Vec<u8> {
    // Simple key derivation for demonstration
    // In production use argon2, scrypt, or PBKDF2
    let mut key = Vec::with_capacity(32);
    let password_bytes = password.as_bytes();
    for i in 0..32 {
        key.push(password_bytes[i % password_bytes.len()] ^ (i as u8));
    }
    key
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
    fn enable_disable_sync() {
        let mut manager = SyncManager::new("device-1");
        assert!(!manager.is_enabled());
        
        manager.enable("password123");
        assert!(manager.is_enabled());
        assert_eq!(manager.state(), SyncState::Idle);
        
        manager.disable();
        assert!(!manager.is_enabled());
    }

    #[test]
    fn create_record() {
        let mut manager = SyncManager::new("device-1");
        manager.enable("password123");
        
        let id = manager.create_record(
            SyncDataType::Bookmarks,
            b"https://example.com",
        ).unwrap();
        
        assert!(!id.is_empty());
        assert!(manager.get_record(&id).is_some());
    }

    #[test]
    fn encrypt_decrypt() {
        let mut manager = SyncManager::new("device-1");
        manager.enable("password123");
        
        let original = b"Hello, World!";
        let encrypted = manager.encrypt(original).unwrap();
        let decrypted = manager.decrypt(&encrypted).unwrap();
        
        assert_eq!(original.to_vec(), decrypted);
    }

    #[test]
    fn delete_record() {
        let mut manager = SyncManager::new("device-1");
        manager.enable("password123");
        
        let id = manager.create_record(SyncDataType::Bookmarks, b"data").unwrap();
        manager.delete_record(&id).unwrap();
        
        let record = manager.get_record(&id).unwrap();
        assert!(record.deleted);
    }

    #[test]
    fn device_management() {
        let mut manager = SyncManager::new("device-1");
        manager.enable("password123");
        
        manager.register_device(DeviceInfo {
            id: "device-2".to_string(),
            name: "Phone".to_string(),
            device_type: DeviceType::Mobile,
            registered_at: now_secs(),
            last_seen: now_secs(),
            active: true,
        });
        
        assert_eq!(manager.devices().len(), 1);
        assert_eq!(manager.active_devices().len(), 1);
        
        manager.remove_device("device-2");
        assert_eq!(manager.devices().len(), 0);
    }

    #[test]
    fn sync_disabled_errors() {
        let mut manager = SyncManager::new("device-1");
        // Don't enable sync
        
        let result = manager.create_record(SyncDataType::Bookmarks, b"data");
        assert!(matches!(result, Err(SyncError::SyncDisabled)));
    }
}
