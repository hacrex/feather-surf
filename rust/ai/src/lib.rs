// FeatherSurf AI Gateway
//
// Optional AI features with privacy-first design.
// All AI is disabled by default and requires explicit user consent.

use std::collections::HashMap;
use std::time::{Duration, SystemTime};

// ── AI Configuration ───────────────────────────────────────────────

/// AI feature toggle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum AiFeature {
    /// Page summarization.
    PageSummarization,
    /// Selection summarization.
    SelectionSummarization,
    /// Developer assistance.
    DeveloperAssistance,
    /// Smart suggestions.
    SmartSuggestions,
}

/// AI provider configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AiConfig {
    /// Whether AI features are enabled.
    pub enabled: bool,
    /// Enabled features.
    pub features: Vec<AiFeature>,
    /// Selected provider.
    pub provider: AiProvider,
    /// API key (encrypted).
    pub api_key: Option<Vec<u8>>,
    /// Whether to use local models.
    pub use_local: bool,
    /// Maximum tokens per request.
    pub max_tokens: usize,
    /// Request timeout in seconds.
    pub timeout_secs: u64,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            features: Vec::new(),
            provider: AiProvider::None,
            api_key: None,
            use_local: false,
            max_tokens: 1000,
            timeout_secs: 30,
        }
    }
}

/// AI provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AiProvider {
    /// No AI (disabled).
    None,
    /// OpenAI.
    OpenAi,
    /// Anthropic.
    Anthropic,
    /// Local model (Ollama/llama.cpp).
    Local,
}

// ── AI Gateway ─────────────────────────────────────────────────────

/// Provider-neutral AI gateway.
pub struct AiGateway {
    /// Configuration.
    config: AiConfig,
    /// Request history.
    requests: Vec<AiRequest>,
    /// Maximum request history.
    max_requests: usize,
    /// Consent given for specific sites.
    site_consents: HashMap<String, bool>,
}

/// An AI request.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AiRequest {
    /// Request ID.
    pub id: String,
    /// Feature that made the request.
    pub feature: AiFeature,
    /// Provider used.
    pub provider: AiProvider,
    /// Input data (page content, selection, etc.).
    pub input: String,
    /// Output from AI.
    pub output: Option<String>,
    /// When the request was made.
    pub timestamp: u64,
    /// Request duration in ms.
    pub duration_ms: Option<u64>,
    /// Whether the request succeeded.
    pub success: bool,
    /// Error message if failed.
    pub error: Option<String>,
    /// Tokens used.
    pub tokens_used: Option<usize>,
}

impl AiGateway {
    pub fn new() -> Self {
        Self {
            config: AiConfig::default(),
            requests: Vec::new(),
            max_requests: 100,
            site_consents: HashMap::new(),
        }
    }

    /// Enable AI with a specific provider.
    pub fn enable(&mut self, provider: AiProvider) {
        self.config.enabled = true;
        self.config.provider = provider;
    }

    /// Disable AI.
    pub fn disable(&mut self) {
        self.config.enabled = false;
        self.config.provider = AiProvider::None;
        self.config.features.clear();
    }

    /// Check if AI is enabled.
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Enable a specific feature.
    pub fn enable_feature(&mut self, feature: AiFeature) {
        if !self.config.features.contains(&feature) {
            self.config.features.push(feature);
        }
    }

    /// Disable a specific feature.
    pub fn disable_feature(&mut self, feature: AiFeature) {
        self.config.features.retain(|f| *f != feature);
    }

    /// Check if a feature is enabled.
    pub fn is_feature_enabled(&self, feature: AiFeature) -> bool {
        self.config.enabled && self.config.features.contains(&feature)
    }

    /// Set API key.
    pub fn set_api_key(&mut self, key: Vec<u8>) {
        self.config.api_key = Some(key);
    }

    /// Check if consent was given for a site.
    pub fn has_site_consent(&self, site: &str) -> bool {
        self.site_consents.get(site).copied().unwrap_or(false)
    }

    /// Grant consent for a site.
    pub fn grant_site_consent(&mut self, site: impl Into<String>) {
        self.site_consents.insert(site.into(), true);
    }

    /// Revoke consent for a site.
    pub fn revoke_site_consent(&mut self, site: &str) {
        self.site_consents.remove(site);
    }

    /// Make an AI request (simulated).
    pub fn request(
        &mut self,
        feature: AiFeature,
        input: &str,
        site: Option<&str>,
    ) -> Result<String, AiError> {
        // Check if enabled
        if !self.config.enabled {
            return Err(AiError::Disabled);
        }

        // Check if feature is enabled
        if !self.config.features.contains(&feature) {
            return Err(AiError::FeatureNotEnabled);
        }

        // Check provider
        if self.config.provider == AiProvider::None {
            return Err(AiError::NoProvider);
        }

        // Check API key for cloud providers
        if self.config.provider != AiProvider::Local && self.config.api_key.is_none() {
            return Err(AiError::NoApiKey);
        }

        // Check site consent
        if let Some(site) = site {
            if !self.has_site_consent(site) {
                return Err(AiError::ConsentRequired);
            }
        }

        // Simulate AI response
        let output = simulate_ai_response(feature, input);
        let tokens = input.len() / 4 + output.len() / 4; // Rough estimate

        let request = AiRequest {
            id: format!("ai-{}", now_secs()),
            feature,
            provider: self.config.provider,
            input: input.to_string(),
            output: Some(output.clone()),
            timestamp: now_secs(),
            duration_ms: Some(100),
            success: true,
            error: None,
            tokens_used: Some(tokens),
        };

        self.requests.push(request);
        if self.requests.len() > self.max_requests {
            self.requests.remove(0);
        }

        Ok(output)
    }

    /// Get request history.
    pub fn requests(&self) -> &[AiRequest] {
        &self.requests
    }

    /// Get total tokens used.
    pub fn total_tokens(&self) -> usize {
        self.requests.iter().filter_map(|r| r.tokens_used).sum()
    }

    /// Get configuration.
    pub fn config(&self) -> &AiConfig {
        &self.config
    }

    /// Clear request history.
    pub fn clear_history(&mut self) {
        self.requests.clear();
    }
}

impl Default for AiGateway {
    fn default() -> Self {
        Self::new()
    }
}

// ── Simulated AI Responses ─────────────────────────────────────────

fn simulate_ai_response(feature: AiFeature, input: &str) -> String {
    match feature {
        AiFeature::PageSummarization => {
            format!("Summary of page content ({} chars): This page contains information about...", input.len())
        }
        AiFeature::SelectionSummarization => {
            format!("Selection summary ({} chars): The selected text discusses...", input.len())
        }
        AiFeature::DeveloperAssistance => {
            format!("Developer assistance for ({} chars): I can help with that code...", input.len())
        }
        AiFeature::SmartSuggestions => {
            format!("Suggestions for ({} chars): Based on your browsing pattern...", input.len())
        }
    }
}

// ── AI Errors ──────────────────────────────────────────────────────

/// AI gateway errors.
#[derive(Debug, Clone, thiserror::Error)]
pub enum AiError {
    #[error("AI features are disabled")]
    Disabled,
    #[error("Feature not enabled")]
    FeatureNotEnabled,
    #[error("No AI provider selected")]
    NoProvider,
    #[error("No API key configured")]
    NoApiKey,
    #[error("Site consent required")]
    ConsentRequired,
    #[error("Request failed: {0}")]
    RequestFailed(String),
    #[error("Rate limited")]
    RateLimited,
    #[error("Token limit exceeded")]
    TokenLimitExceeded,
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
    fn ai_disabled_by_default() {
        let gateway = AiGateway::new();
        assert!(!gateway.is_enabled());
        assert!(gateway.config().features.is_empty());
    }

    #[test]
    fn enable_ai() {
        let mut gateway = AiGateway::new();
        gateway.enable(AiProvider::Local);
        assert!(gateway.is_enabled());
        assert_eq!(gateway.config().provider, AiProvider::Local);
    }

    #[test]
    fn enable_feature() {
        let mut gateway = AiGateway::new();
        gateway.enable(AiProvider::Local);
        gateway.enable_feature(AiFeature::PageSummarization);
        
        assert!(gateway.is_feature_enabled(AiFeature::PageSummarization));
        assert!(!gateway.is_feature_enabled(AiFeature::SelectionSummarization));
    }

    #[test]
    fn site_consent() {
        let mut gateway = AiGateway::new();
        assert!(!gateway.has_site_consent("example.com"));
        
        gateway.grant_site_consent("example.com");
        assert!(gateway.has_site_consent("example.com"));
        
        gateway.revoke_site_consent("example.com");
        assert!(!gateway.has_site_consent("example.com"));
    }

    #[test]
    fn request_without_consent_fails() {
        let mut gateway = AiGateway::new();
        gateway.enable(AiProvider::Local);
        gateway.enable_feature(AiFeature::PageSummarization);
        gateway.set_api_key(vec![1, 2, 3]);
        
        let result = gateway.request(
            AiFeature::PageSummarization,
            "Test content",
            Some("example.com"),
        );
        
        assert!(matches!(result, Err(AiError::ConsentRequired)));
    }

    #[test]
    fn request_with_consent_succeeds() {
        let mut gateway = AiGateway::new();
        gateway.enable(AiProvider::Local);
        gateway.enable_feature(AiFeature::PageSummarization);
        gateway.set_api_key(vec![1, 2, 3]);
        gateway.grant_site_consent("example.com");
        
        let result = gateway.request(
            AiFeature::PageSummarization,
            "Test content",
            Some("example.com"),
        );
        
        assert!(result.is_ok());
        assert_eq!(gateway.requests().len(), 1);
    }
}
