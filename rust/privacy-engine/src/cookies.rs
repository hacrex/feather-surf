// FeatherSurf Cookie Controls
//
// Cookie management, third-party cookie blocking, and storage controls.

use std::collections::HashMap;
use std::time::SystemTime;

// ── Cookie ─────────────────────────────────────────────────────────

/// A browser cookie.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Cookie {
    /// Cookie name.
    pub name: String,
    /// Cookie value.
    pub value: String,
    /// Domain the cookie belongs to.
    pub domain: String,
    /// Path within the domain.
    pub path: String,
    /// Expiration time (None = session cookie).
    pub expires: Option<u64>,
    /// Whether the cookie is secure only.
    pub secure: bool,
    /// Whether the cookie is HTTP-only (not accessible via JavaScript).
    pub http_only: bool,
    /// Same-site policy.
    pub same_site: SameSite,
    /// Whether this is a first-party cookie.
    pub first_party: bool,
    /// When the cookie was created.
    pub created_at: u64,
}

/// Same-site cookie policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SameSite {
    /// No same-site restriction.
    None,
    /// Send cookie only for same-site requests.
    Lax,
    /// Send cookie only in same-site context.
    Strict,
}

/// Cookie storage for a profile.
pub struct CookieStore {
    /// Cookies indexed by domain and name.
    cookies: HashMap<String, Vec<Cookie>>,
    /// Third-party cookie policy.
    third_party_policy: ThirdPartyPolicy,
    /// Site-specific exceptions.
    site_exceptions: HashMap<String, bool>,
    /// Maximum cookies per domain.
    max_per_domain: usize,
    /// Maximum total cookies.
    max_total: usize,
}

/// Third-party cookie policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ThirdPartyPolicy {
    /// Block all third-party cookies.
    BlockAll,
    /// Block third-party cookies with exceptions.
    BlockWithExceptions,
    /// Allow all third-party cookies.
    AllowAll,
    /// Block unclassified third-party cookies.
    BlockUnclassified,
}

impl CookieStore {
    pub fn new() -> Self {
        Self {
            cookies: HashMap::new(),
            third_party_policy: ThirdPartyPolicy::BlockWithExceptions,
            site_exceptions: HashMap::new(),
            max_per_domain: 180,
            max_total: 4000,
        }
    }

    /// Create with custom policy.
    pub fn with_policy(policy: ThirdPartyPolicy) -> Self {
        Self {
            third_party_policy: policy,
            ..Self::new()
        }
    }

    /// Add a cookie.
    pub fn add_cookie(&mut self, cookie: Cookie) -> bool {
        // Check if third-party and policy allows
        if !cookie.first_party && !self.allows_third_party(&cookie.domain) {
            return false;
        }

        // Check total limit first (before holding any domain entry)
        let total: usize = self.cookies.values().map(|c| c.len()).sum();
        if total >= self.max_total {
            // Remove oldest cookie overall
            if let Some(cookies) = self.cookies.values_mut().next() {
                if !cookies.is_empty() {
                    cookies.remove(0);
                }
            }
        }

        // Check per-domain limit
        let domain_cookies = self.cookies.entry(cookie.domain.clone()).or_default();
        if domain_cookies.len() >= self.max_per_domain {
            // Remove oldest cookie
            domain_cookies.remove(0);
        }

        // Add or update cookie
        if let Some(existing) = domain_cookies.iter_mut().find(|c| c.name == cookie.name) {
            *existing = cookie;
        } else {
            domain_cookies.push(cookie);
        }

        true
    }

    /// Get a cookie by domain and name.
    pub fn get_cookie(&self, domain: &str, name: &str) -> Option<&Cookie> {
        self.cookies.get(domain)?.iter().find(|c| c.name == name)
    }

    /// Get all cookies for a domain.
    pub fn cookies_for_domain(&self, domain: &str) -> Vec<&Cookie> {
        self.cookies
            .get(domain)
            .map(|c| c.iter().collect())
            .unwrap_or_default()
    }

    /// Get all cookies matching a URL.
    pub fn cookies_for_url(&self, url: &str, first_party: bool) -> Vec<&Cookie> {
        let mut result = Vec::new();
        for (_, cookies) in &self.cookies {
            for cookie in cookies {
                if self.matches_url(cookie, url, first_party) {
                    result.push(cookie);
                }
            }
        }
        result
    }

    /// Delete a cookie.
    pub fn delete_cookie(&mut self, domain: &str, name: &str) -> bool {
        if let Some(cookies) = self.cookies.get_mut(domain) {
            let len = cookies.len();
            cookies.retain(|c| c.name != name);
            cookies.len() < len
        } else {
            false
        }
    }

    /// Delete all cookies for a domain.
    pub fn delete_domain(&mut self, domain: &str) -> usize {
        if let Some(cookies) = self.cookies.remove(domain) {
            cookies.len()
        } else {
            0
        }
    }

    /// Clear all cookies.
    pub fn clear(&mut self) {
        self.cookies.clear();
    }

    /// Clear expired cookies.
    pub fn clear_expired(&mut self) -> usize {
        let now = now_secs();
        let before: usize = self.cookies.values().map(|c| c.len()).sum();

        for cookies in self.cookies.values_mut() {
            cookies.retain(|c| c.expires.map_or(true, |exp| exp > now));
        }

        // Remove empty domains
        self.cookies.retain(|_, cookies| !cookies.is_empty());

        let after: usize = self.cookies.values().map(|c| c.len()).sum();
        before - after
    }

    /// Set third-party policy.
    pub fn set_third_party_policy(&mut self, policy: ThirdPartyPolicy) {
        self.third_party_policy = policy;
    }

    /// Add a site exception for third-party cookies.
    pub fn add_third_party_exception(&mut self, domain: impl Into<String>, allow: bool) {
        self.site_exceptions.insert(domain.into(), allow);
    }

    /// Check if third-party cookies are allowed for a domain.
    fn allows_third_party(&self, domain: &str) -> bool {
        match self.third_party_policy {
            ThirdPartyPolicy::BlockAll => false,
            ThirdPartyPolicy::AllowAll => true,
            ThirdPartyPolicy::BlockWithExceptions => {
                self.site_exceptions.get(domain).copied().unwrap_or(false)
            }
            ThirdPartyPolicy::BlockUnclassified => {
                // In practice, this would check a classification database
                self.site_exceptions.get(domain).copied().unwrap_or(false)
            }
        }
    }

    /// Check if a cookie matches a URL.
    fn matches_url(&self, cookie: &Cookie, url: &str, first_party: bool) -> bool {
        if first_party && !cookie.first_party {
            return false;
        }

        // Simple domain matching
        if let Ok(parsed) = url::Url::parse(url) {
            let host = parsed.host_str().unwrap_or("");
            host.ends_with(&cookie.domain) || host == cookie.domain.trim_start_matches('.')
        } else {
            false
        }
    }

    /// Get cookie count.
    pub fn count(&self) -> usize {
        self.cookies.values().map(|c| c.len()).sum()
    }

    /// Get domain count.
    pub fn domain_count(&self) -> usize {
        self.cookies.len()
    }
}

impl Default for CookieStore {
    fn default() -> Self {
        Self::new()
    }
}

// ── Storage Controls ───────────────────────────────────────────────

/// Browser storage types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum StorageType {
    /// Local Storage.
    LocalStorage,
    /// Session Storage.
    SessionStorage,
    /// IndexedDB.
    IndexedDB,
    /// Cache API.
    CacheApi,
    /// Service Worker cache.
    ServiceWorkerCache,
    /// Web SQL (deprecated).
    WebSql,
}

/// Storage entry.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StorageEntry {
    /// Origin (protocol + host).
    pub origin: String,
    /// Storage type.
    pub storage_type: StorageType,
    /// Key.
    pub key: String,
    /// Value size in bytes.
    pub size: u64,
    /// When created.
    pub created_at: u64,
    /// When last accessed.
    pub last_accessed: u64,
}

/// Storage manager for a profile.
pub struct StorageManager {
    /// Storage entries indexed by origin.
    entries: HashMap<String, Vec<StorageEntry>>,
    /// Per-origin quotas.
    quotas: HashMap<String, u64>,
    /// Default quota (5MB).
    default_quota: u64,
    /// Total storage limit.
    total_limit: u64,
}

impl StorageManager {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            quotas: HashMap::new(),
            default_quota: 5 * 1024 * 1024, // 5MB
            total_limit: 100 * 1024 * 1024, // 100MB
        }
    }

    /// Add a storage entry.
    pub fn add_entry(&mut self, entry: StorageEntry) -> bool {
        // Check total limit first (before holding any origin entry)
        let total_size: u64 = self
            .entries
            .values()
            .flat_map(|e| e.iter())
            .map(|e| e.size)
            .sum();
        if total_size + entry.size > self.total_limit {
            return false;
        }

        let origin_entries = self.entries.entry(entry.origin.clone()).or_default();

        // Check per-origin quota
        let current_size: u64 = origin_entries.iter().map(|e| e.size).sum();
        let quota = self
            .quotas
            .get(&entry.origin)
            .copied()
            .unwrap_or(self.default_quota);

        if current_size + entry.size > quota {
            return false;
        }

        origin_entries.push(entry);
        true
    }

    /// Get all entries for an origin.
    pub fn entries_for_origin(&self, origin: &str) -> Vec<&StorageEntry> {
        self.entries
            .get(origin)
            .map(|e| e.iter().collect())
            .unwrap_or_default()
    }

    /// Get entries by type.
    pub fn entries_by_type(&self, storage_type: StorageType) -> Vec<&StorageEntry> {
        self.entries
            .values()
            .flat_map(|e| e.iter())
            .filter(|e| e.storage_type == storage_type)
            .collect()
    }

    /// Delete an entry.
    pub fn delete_entry(&mut self, origin: &str, key: &str) -> bool {
        if let Some(entries) = self.entries.get_mut(origin) {
            let len = entries.len();
            entries.retain(|e| e.key != key);
            entries.len() < len
        } else {
            false
        }
    }

    /// Clear all storage for an origin.
    pub fn clear_origin(&mut self, origin: &str) -> usize {
        if let Some(entries) = self.entries.remove(origin) {
            entries.len()
        } else {
            0
        }
    }

    /// Clear all storage.
    pub fn clear_all(&mut self) {
        self.entries.clear();
    }

    /// Clear storage by type.
    pub fn clear_by_type(&mut self, storage_type: StorageType) -> usize {
        let before: usize = self.entries.values().map(|e| e.len()).sum();

        for entries in self.entries.values_mut() {
            entries.retain(|e| e.storage_type != storage_type);
        }

        self.entries.retain(|_, entries| !entries.is_empty());

        let after: usize = self.entries.values().map(|e| e.len()).sum();
        before - after
    }

    /// Get total storage size.
    pub fn total_size(&self) -> u64 {
        self.entries
            .values()
            .flat_map(|e| e.iter())
            .map(|e| e.size)
            .sum()
    }

    /// Get storage size for an origin.
    pub fn origin_size(&self, origin: &str) -> u64 {
        self.entries
            .get(origin)
            .map(|e| e.iter().map(|e| e.size).sum())
            .unwrap_or(0)
    }

    /// Set quota for an origin.
    pub fn set_quota(&mut self, origin: impl Into<String>, quota: u64) {
        self.quotas.insert(origin.into(), quota);
    }
}

impl Default for StorageManager {
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

    fn test_cookie(domain: &str, name: &str, first_party: bool) -> Cookie {
        Cookie {
            name: name.to_string(),
            value: "value".to_string(),
            domain: domain.to_string(),
            path: "/".to_string(),
            expires: None,
            secure: false,
            http_only: false,
            same_site: SameSite::Lax,
            first_party,
            created_at: now_secs(),
        }
    }

    #[test]
    fn add_and_get_cookie() {
        let mut store = CookieStore::new();
        let cookie = test_cookie("example.com", "session", true);
        assert!(store.add_cookie(cookie));
        assert!(store.get_cookie("example.com", "session").is_some());
    }

    #[test]
    fn third_party_blocking() {
        let mut store = CookieStore::new();
        let cookie = test_cookie("tracker.com", "id", false);
        assert!(!store.add_cookie(cookie));
    }

    #[test]
    fn third_party_exception() {
        let mut store = CookieStore::new();
        store.add_third_party_exception("tracker.com", true);
        let cookie = test_cookie("tracker.com", "id", false);
        assert!(store.add_cookie(cookie));
    }

    #[test]
    fn delete_cookie() {
        let mut store = CookieStore::new();
        store.add_cookie(test_cookie("example.com", "session", true));
        assert!(store.delete_cookie("example.com", "session"));
        assert!(store.get_cookie("example.com", "session").is_none());
    }

    #[test]
    fn clear_expired() {
        let mut store = CookieStore::new();
        let mut cookie = test_cookie("example.com", "expired", true);
        cookie.expires = Some(0); // Already expired
        store.add_cookie(cookie);
        store.add_cookie(test_cookie("example.com", "valid", true));

        let cleared = store.clear_expired();
        assert_eq!(cleared, 1);
        assert_eq!(store.count(), 1);
    }

    #[test]
    fn storage_add_and_get() {
        let mut manager = StorageManager::new();
        let entry = StorageEntry {
            origin: "https://example.com".to_string(),
            storage_type: StorageType::LocalStorage,
            key: "theme".to_string(),
            size: 100,
            created_at: now_secs(),
            last_accessed: now_secs(),
        };
        assert!(manager.add_entry(entry));
        assert_eq!(manager.entries_for_origin("https://example.com").len(), 1);
    }

    #[test]
    fn storage_quota() {
        let mut manager = StorageManager::new();
        manager.set_quota("https://example.com", 200);

        let entry = StorageEntry {
            origin: "https://example.com".to_string(),
            storage_type: StorageType::LocalStorage,
            key: "data".to_string(),
            size: 300,
            created_at: now_secs(),
            last_accessed: now_secs(),
        };
        assert!(!manager.add_entry(entry));
    }

    #[test]
    fn clear_by_type() {
        let mut manager = StorageManager::new();
        manager.add_entry(StorageEntry {
            origin: "https://example.com".to_string(),
            storage_type: StorageType::LocalStorage,
            key: "a".to_string(),
            size: 100,
            created_at: now_secs(),
            last_accessed: now_secs(),
        });
        manager.add_entry(StorageEntry {
            origin: "https://example.com".to_string(),
            storage_type: StorageType::SessionStorage,
            key: "b".to_string(),
            size: 100,
            created_at: now_secs(),
            last_accessed: now_secs(),
        });

        let cleared = manager.clear_by_type(StorageType::LocalStorage);
        assert_eq!(cleared, 1);
        assert_eq!(manager.total_size(), 100);
    }
}
