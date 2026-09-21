// FeatherSurf Privacy Engine
//
// Request blocking, tracker detection, fingerprinting protection,
// and cookie controls. Uses filter lists for blocking rules.

pub mod cookies;
pub mod fingerprint;
pub mod permissions;

use std::collections::{HashMap, HashSet};
use std::time::{Duration, SystemTime};
use url::Url;

// ── Filter Lists ───────────────────────────────────────────────────

/// A filter rule for blocking requests.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FilterRule {
    /// The pattern to match (domain, URL pattern, or regex).
    pub pattern: String,
    /// Rule type.
    pub rule_type: FilterRuleType,
    /// Options for the rule.
    pub options: FilterOptions,
    /// Source list name.
    pub source: String,
}

/// Type of filter rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FilterRuleType {
    /// Block by domain (e.g., "ads.example.com").
    Domain,
    /// Block by URL pattern (e.g., "*/tracking.js").
    UrlPattern,
    /// Block by regex.
    Regex,
    /// Block all third-party requests to a domain.
    ThirdParty,
    /// Exception rule (allow).
    Exception,
}

/// Options for a filter rule.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct FilterOptions {
    /// Apply only to specific domains.
    pub domains: Vec<String>,
    /// Don't apply to specific domains.
    pub exclude_domains: Vec<String>,
    /// Apply only to specific resource types.
    pub resource_types: Vec<ResourceType>,
    /// Apply only to specific protocols.
    pub protocols: Vec<String>,
}

/// Resource type for filtering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ResourceType {
    Script,
    Stylesheet,
    Image,
    Media,
    Font,
    XmlHttpRequest,
    WebSocket,
    WebRtc,
    CspReport,
    Other,
}

/// A filter list (e.g., EasyList, EasyPrivacy).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FilterList {
    pub name: String,
    pub url: String,
    pub rules: Vec<FilterRule>,
    pub last_updated: u64,
    pub version: String,
    pub enabled: bool,
}

impl FilterList {
    pub fn new(name: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            url: url.into(),
            rules: Vec::new(),
            last_updated: 0,
            version: String::new(),
            enabled: true,
        }
    }

    /// Check if a URL should be blocked.
    pub fn should_block(&self, url: &str, source_url: &str, resource_type: ResourceType) -> bool {
        if !self.enabled {
            return false;
        }

        for rule in &self.rules {
            if rule.matches(url, source_url, resource_type) {
                return !rule.options.is_exception();
            }
        }

        false
    }
}

impl FilterRule {
    /// Check if this rule matches a request.
    pub fn matches(&self, url: &str, source_url: &str, resource_type: ResourceType) -> bool {
        // Check resource type filter
        if !self.options.resource_types.is_empty()
            && !self.options.resource_types.contains(&resource_type)
        {
            return false;
        }

        // Check domain filters
        if !self.options.domains.is_empty() {
            if let Ok(source) = Url::parse(source_url) {
                let host = source.host_str().unwrap_or("");
                if !self.options.domains.iter().any(|d| host.contains(d)) {
                    return false;
                }
            }
        }

        // Check exclude domains
        if !self.options.exclude_domains.is_empty() {
            if let Ok(source) = Url::parse(source_url) {
                let host = source.host_str().unwrap_or("");
                if self.options.exclude_domains.iter().any(|d| host.contains(d)) {
                    return false;
                }
            }
        }

        // Match pattern
        match self.rule_type {
            FilterRuleType::Domain => {
                if let Ok(target) = Url::parse(url) {
                    let host = target.host_str().unwrap_or("");
                    host.contains(&self.pattern) || self.pattern == host
                } else {
                    false
                }
            }
            FilterRuleType::UrlPattern => url.contains(&self.pattern),
            FilterRuleType::Regex => regex_match(url, &self.pattern),
            FilterRuleType::ThirdParty => {
                if let (Ok(target), Ok(source)) = (Url::parse(url), Url::parse(source_url)) {
                    let target_host = target.host_str().unwrap_or("");
                    let source_host = source.host_str().unwrap_or("");
                    target_host != source_host && url.contains(&self.pattern)
                } else {
                    false
                }
            }
            FilterRuleType::Exception => false, // Exceptions handled separately
        }
    }
}

// ── Request Blocker ────────────────────────────────────────────────

/// The main request blocking engine.
pub struct RequestBlocker {
    /// Active filter lists.
    filter_lists: Vec<FilterList>,
    /// Per-site allowlists.
    site_allowlists: HashMap<String, HashSet<String>>,
    /// Per-site blocklists.
    site_blocklists: HashMap<String, HashSet<String>>,
    /// Global allowlist (domains never blocked).
    global_allowlist: HashSet<String>,
    /// Global blocklist (always blocked).
    global_blocklist: HashSet<String>,
    /// Statistics.
    stats: BlockStats,
}

/// Blocking statistics.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct BlockStats {
    pub requests_checked: u64,
    pub requests_blocked: u64,
    pub trackers_blocked: u64,
    pub ads_blocked: u64,
    pub malware_blocked: u64,
}

impl RequestBlocker {
    pub fn new() -> Self {
        Self {
            filter_lists: Vec::new(),
            site_allowlists: HashMap::new(),
            site_blocklists: HashMap::new(),
            global_allowlist: HashSet::new(),
            global_blocklist: HashSet::new(),
            stats: BlockStats::default(),
        }
    }

    /// Add a filter list.
    pub fn add_filter_list(&mut self, list: FilterList) {
        self.filter_lists.push(list);
    }

    /// Remove a filter list by name.
    pub fn remove_filter_list(&mut self, name: &str) -> bool {
        let len = self.filter_lists.len();
        self.filter_lists.retain(|l| l.name != name);
        self.filter_lists.len() < len
    }

    /// Get all filter lists.
    pub fn filter_lists(&self) -> &[FilterList] {
        &self.filter_lists
    }

    /// Enable/disable a filter list.
    pub fn set_filter_list_enabled(&mut self, name: &str, enabled: bool) -> bool {
        if let Some(list) = self.filter_lists.iter_mut().find(|l| l.name == name) {
            list.enabled = enabled;
            true
        } else {
            false
        }
    }

    /// Add a domain to the site allowlist.
    pub fn allow_site(&mut self, site: impl Into<String>, domain: impl Into<String>) {
        self.site_allowlists
            .entry(site.into())
            .or_default()
            .insert(domain.into());
    }

    /// Add a domain to the site blocklist.
    pub fn block_site(&mut self, site: impl Into<String>, domain: impl Into<String>) {
        self.site_blocklists
            .entry(site.into())
            .or_default()
            .insert(domain.into());
    }

    /// Remove a site from the allowlist.
    pub fn remove_site_allow(&mut self, site: &str, domain: &str) -> bool {
        if let Some(allowlist) = self.site_allowlists.get_mut(site) {
            allowlist.remove(domain)
        } else {
            false
        }
    }

    /// Add a domain to the global allowlist.
    pub fn allow_global(&mut self, domain: impl Into<String>) {
        self.global_allowlist.insert(domain.into());
    }

    /// Add a domain to the global blocklist.
    pub fn block_global(&mut self, domain: impl Into<String>) {
        self.global_blocklist.insert(domain.into());
    }

    /// Check if a request should be blocked.
    pub fn should_block(
        &mut self,
        url: &str,
        source_url: &str,
        resource_type: ResourceType,
    ) -> BlockDecision {
        self.stats.requests_checked += 1;

        // Parse the URL
        let target_host = Url::parse(url)
            .ok()
            .and_then(|u| u.host_str().map(|s| s.to_string()))
            .unwrap_or_default();

        let source_host = Url::parse(source_url)
            .ok()
            .and_then(|u| u.host_str().map(|s| s.to_string()))
            .unwrap_or_default();

        // Check global allowlist first
        if self.global_allowlist.contains(&target_host) {
            return BlockDecision::Allow;
        }

        // Check global blocklist
        if self.global_blocklist.contains(&target_host) {
            self.stats.requests_blocked += 1;
            self.stats.trackers_blocked += 1;
            return BlockDecision::Block {
                reason: BlockReason::GlobalBlocklist,
                rule_source: "global".to_string(),
            };
        }

        // Check site allowlist
        if let Some(allowlist) = self.site_allowlists.get(&source_host) {
            if allowlist.contains(&target_host) {
                return BlockDecision::Allow;
            }
        }

        // Check site blocklist
        if let Some(blocklist) = self.site_blocklists.get(&source_host) {
            if blocklist.contains(&target_host) {
                self.stats.requests_blocked += 1;
                return BlockDecision::Block {
                    reason: BlockReason::SiteBlocklist,
                    rule_source: format!("site:{}", source_host),
                };
            }
        }

        // Check filter lists
        for list in &self.filter_lists {
            if list.should_block(url, source_url, resource_type) {
                self.stats.requests_blocked += 1;
                self.stats.trackers_blocked += 1;
                return BlockDecision::Block {
                    reason: BlockReason::FilterList {
                        list_name: list.name.clone(),
                    },
                    rule_source: list.name.clone(),
                };
            }
        }

        BlockDecision::Allow
    }

    /// Get blocking statistics.
    pub fn stats(&self) -> &BlockStats {
        &self.stats
    }

    /// Reset statistics.
    pub fn reset_stats(&mut self) {
        self.stats = BlockStats::default();
    }
}

impl Default for RequestBlocker {
    fn default() -> Self {
        Self::new()
    }
}

// ── Block Decision ─────────────────────────────────────────────────

/// Decision on whether to block a request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockDecision {
    /// Allow the request.
    Allow,
    /// Block the request.
    Block {
        reason: BlockReason,
        rule_source: String,
    },
}

/// Reason for blocking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockReason {
    /// Blocked by global blocklist.
    GlobalBlocklist,
    /// Blocked by site-specific blocklist.
    SiteBlocklist,
    /// Blocked by a filter list.
    FilterList { list_name: String },
    /// Blocked as tracker.
    Tracker,
    /// Blocked as ad.
    Ad,
    /// Blocked as malware.
    Malware,
}

// ── Tracker Detection ──────────────────────────────────────────────

/// Known tracker categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TrackerCategory {
    /// Analytics/tracking scripts.
    Analytics,
    /// Advertising.
    Advertising,
    /// Fingerprinting.
    Fingerprinting,
    /// Cryptomining.
    Cryptomining,
    /// Social media widgets.
    Social,
    /// Unknown/suspicious.
    Unknown,
}

/// Known tracker database.
pub struct TrackerDatabase {
    /// Known tracker domains.
    trackers: HashMap<String, TrackerCategory>,
}

impl TrackerDatabase {
    pub fn new() -> Self {
        let mut trackers = HashMap::new();

        // Common analytics trackers
        trackers.insert("google-analytics.com".to_string(), TrackerCategory::Analytics);
        trackers.insert("googletagmanager.com".to_string(), TrackerCategory::Analytics);
        trackers.insert("facebook.net".to_string(), TrackerCategory::Analytics);
        trackers.insert("hotjar.com".to_string(), TrackerCategory::Analytics);
        trackers.insert("mixpanel.com".to_string(), TrackerCategory::Analytics);
        trackers.insert("amplitude.com".to_string(), TrackerCategory::Analytics);

        // Common ad networks
        trackers.insert("doubleclick.net".to_string(), TrackerCategory::Advertising);
        trackers.insert("googlesyndication.com".to_string(), TrackerCategory::Advertising);
        trackers.insert("adservice.google.com".to_string(), TrackerCategory::Advertising);
        trackers.insert("amazon-adsystem.com".to_string(), TrackerCategory::Advertising);

        // Fingerprinting
        trackers.insert("fingerprintjs.com".to_string(), TrackerCategory::Fingerprinting);

        // Cryptomining
        trackers.insert("coinhive.com".to_string(), TrackerCategory::Cryptomining);

        Self { trackers }
    }

    /// Check if a domain is a known tracker.
    pub fn is_tracker(&self, domain: &str) -> Option<TrackerCategory> {
        // Check exact match
        if let Some(category) = self.trackers.get(domain) {
            return Some(*category);
        }

        // Check if domain is a subdomain of a known tracker
        for (tracker_domain, category) in &self.trackers {
            if domain.ends_with(tracker_domain) || domain.ends_with(&format!(".{}", tracker_domain)) {
                return Some(*category);
            }
        }

        None
    }

    /// Add a tracker domain.
    pub fn add_tracker(&mut self, domain: impl Into<String>, category: TrackerCategory) {
        self.trackers.insert(domain.into(), category);
    }
}

impl Default for TrackerDatabase {
    fn default() -> Self {
        Self::new()
    }
}

// ── Helpers ────────────────────────────────────────────────────────

/// Simple regex-like pattern matching (supports * and ?).
fn regex_match(url: &str, pattern: &str) -> bool {
    // Convert simple glob to regex-like matching
    let pattern = pattern.replace(".", r"\.").replace("*", ".*").replace("?", ".");
    let re = format!("^{}$", pattern);
    // Simple matching for now - in production use regex crate
    url.contains(&pattern.replace(".*", "").replace("\\", ""))
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domain_blocking() {
        let mut blocker = RequestBlocker::new();
        blocker.block_global("ads.example.com");

        let decision = blocker.should_block(
            "https://ads.example.com/script.js",
            "https://example.com",
            ResourceType::Script,
        );

        assert!(matches!(decision, BlockDecision::Block { .. }));
    }

    #[test]
    fn site_allowlist() {
        let mut blocker = RequestBlocker::new();
        blocker.block_global("tracker.com");
        blocker.allow_site("example.com", "tracker.com");

        let decision = blocker.should_block(
            "https://tracker.com/pixel.gif",
            "https://example.com",
            ResourceType::Image,
        );

        assert_eq!(decision, BlockDecision::Allow);
    }

    #[test]
    fn filter_list_blocking() {
        let mut blocker = RequestBlocker::new();
        let mut list = FilterList::new("EasyList", "https://easylist.to/easylist/easylist.txt");
        list.rules.push(FilterRule {
            pattern: "ads.example.com".to_string(),
            rule_type: FilterRuleType::Domain,
            options: FilterOptions::default(),
            source: "EasyList".to_string(),
        });
        blocker.add_filter_list(list);

        let decision = blocker.should_block(
            "https://ads.example.com/banner.js",
            "https://example.com",
            ResourceType::Script,
        );

        assert!(matches!(decision, BlockDecision::Block { .. }));
    }

    #[test]
    fn statistics_tracking() {
        let mut blocker = RequestBlocker::new();
        blocker.block_global("tracker.com");

        blocker.should_block(
            "https://tracker.com/pixel.gif",
            "https://example.com",
            ResourceType::Image,
        );
        blocker.should_block(
            "https://safe.com/script.js",
            "https://example.com",
            ResourceType::Script,
        );

        assert_eq!(blocker.stats().requests_checked, 2);
        assert_eq!(blocker.stats().requests_blocked, 1);
    }

    #[test]
    fn tracker_detection() {
        let db = TrackerDatabase::new();
        assert_eq!(
            db.is_tracker("google-analytics.com"),
            Some(TrackerCategory::Analytics)
        );
        assert_eq!(
            db.is_tracker("cdn.google-analytics.com"),
            Some(TrackerCategory::Analytics)
        );
        assert_eq!(db.is_tracker("example.com"), None);
    }

    #[test]
    fn enable_disable_filter_list() {
        let mut blocker = RequestBlocker::new();
        let mut list = FilterList::new("EasyList", "https://easylist.to/easylist/easylist.txt");
        list.rules.push(FilterRule {
            pattern: "ads.example.com".to_string(),
            rule_type: FilterRuleType::Domain,
            options: FilterOptions::default(),
            source: "EasyList".to_string(),
        });
        blocker.add_filter_list(list);

        // Block when enabled
        let decision = blocker.should_block(
            "https://ads.example.com/banner.js",
            "https://example.com",
            ResourceType::Script,
        );
        assert!(matches!(decision, BlockDecision::Block { .. }));

        // Allow when disabled
        blocker.set_filter_list_enabled("EasyList", false);
        let decision = blocker.should_block(
            "https://ads.example.com/banner.js",
            "https://example.com",
            ResourceType::Script,
        );
        assert_eq!(decision, BlockDecision::Allow);
    }
}
