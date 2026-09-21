//! Window management abstraction.
//!
//! Each platform implements window creation and management differently.
//! This trait provides a uniform interface for the browser shell.

/// Unique identifier for a window.
pub type WindowId = u64;

/// Position and size of a window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// Trait for platform-specific window management.
///
/// # Platform Implementations
///
/// - **Windows:** Win32 API (`CreateWindowExW`)
/// - **Linux:** GTK4 or raw Wayland/X11
/// - **macOS:** AppKit (`NSWindow`)
/// - **Android:** Android Activity / SurfaceView
/// - **iOS:** UIKit (`UIWindow` + `UIViewController`)
pub trait Window {
    /// Create a new browser window.
    fn create_window(&mut self, rect: WindowRect, title: &str) -> Result<WindowId, WindowError>;

    /// Show or hide a window.
    fn set_visible(&mut self, id: WindowId, visible: bool) -> Result<(), WindowError>;

    /// Set the window title.
    fn set_title(&mut self, id: WindowId, title: &str) -> Result<(), WindowError>;

    /// Set the window bounds.
    fn set_rect(&mut self, id: WindowId, rect: WindowRect) -> Result<(), WindowError>;

    /// Get the current window bounds.
    fn get_rect(&self, id: WindowId) -> Option<WindowRect>;

    /// Minimize the window.
    fn minimize(&mut self, id: WindowId) -> Result<(), WindowError>;

    /// Maximize or restore the window.
    fn maximize_or_restore(&mut self, id: WindowId) -> Result<(), WindowError>;

    /// Close and destroy a window.
    fn close_window(&mut self, id: WindowId) -> Result<(), WindowError>;

    /// Enter fullscreen mode.
    fn enter_fullscreen(&mut self, id: WindowId) -> Result<(), WindowError>;

    /// Exit fullscreen mode.
    fn exit_fullscreen(&mut self, id: WindowId) -> Result<(), WindowError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WindowError {
    NotFound(WindowId),
    CreationFailed(String),
    PlatformError(String),
}

impl std::fmt::Display for WindowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(id) => write!(f, "window {id} not found"),
            Self::CreationFailed(msg) => write!(f, "window creation failed: {msg}"),
            Self::PlatformError(msg) => write!(f, "platform error: {msg}"),
        }
    }
}

impl std::error::Error for WindowError {}
