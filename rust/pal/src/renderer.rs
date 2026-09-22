//! Rendering engine abstraction.
//!
//! Each platform provides a renderer that manages web content.
//! The renderer is responsible for:
//! - Loading and displaying web pages
//! - Managing per-tab renderer processes
//! - Communicating tab state back to the core

use tab_manager::{TabSnapshot, TabState};

/// Unique identifier for a renderer instance (maps to a tab).
pub type RendererId = u64;

/// Trait for platform-specific rendering engines.
///
/// # Implementations by Platform
///
/// - **Windows/Linux/macOS:** CEF (Chromium Embedded Framework)
/// - **Android:** Android WebView
/// - **iOS:** WKWebView (Apple WebKit)
///
/// # Key Difference: iOS
///
/// iOS mandates WebKit, so it cannot use Chromium/CEF. The `Renderer` trait
/// abstracts this difference. The tab-manager state machine works identically
/// regardless of rendering backend.
pub trait Renderer {
    /// Create a new renderer instance for a tab.
    fn create_renderer(&mut self, tab_id: RendererId, url: &str) -> Result<(), RendererError>;

    /// Navigate an existing renderer to a new URL.
    fn navigate(&mut self, id: RendererId, url: &str) -> Result<(), RendererError>;

    /// Get the current URL of a renderer.
    fn current_url(&self, id: RendererId) -> Option<String>;

    /// Get the page title of a renderer.
    fn current_title(&self, id: RendererId) -> Option<String>;

    /// Capture the current state for tab suspension/restoration.
    fn capture_state(&self, id: RendererId) -> Option<TabSnapshot>;

    /// Restore a renderer from a captured state (for suspended tabs).
    fn restore_state(
        &mut self,
        id: RendererId,
        snapshot: &TabSnapshot,
    ) -> Result<(), RendererError>;

    /// Notify the renderer of a lifecycle state change.
    ///
    /// The renderer should:
    /// - `Active`: Full rendering, network access
    /// - `RecentlyActive`: Keep alive but deprioritize
    /// - `Background`: Throttle animations/timers
    /// - `Frozen`: Pause JavaScript execution
    /// - `Suspended`: Serialize state and release renderer process
    /// - `Discardable`: Mark as discardable under memory pressure
    fn notify_state_change(&mut self, id: RendererId, state: TabState)
        -> Result<(), RendererError>;

    /// Destroy a renderer and release its resources.
    fn destroy_renderer(&mut self, id: RendererId) -> Result<(), RendererError>;

    /// Get the OS PID of the renderer process for a given tab.
    fn renderer_pid(&self, id: RendererId) -> Option<u32>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RendererError {
    NotFound(RendererId),
    CreationFailed(String),
    NavigationFailed(String),
    PlatformError(String),
}

impl std::fmt::Display for RendererError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(id) => write!(f, "renderer {id} not found"),
            Self::CreationFailed(msg) => write!(f, "renderer creation failed: {msg}"),
            Self::NavigationFailed(msg) => write!(f, "navigation failed: {msg}"),
            Self::PlatformError(msg) => write!(f, "platform error: {msg}"),
        }
    }
}

impl std::error::Error for RendererError {}
