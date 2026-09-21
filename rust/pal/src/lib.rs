//! Platform Abstraction Layer (PAL)
//!
//! Defines the trait interface that each platform shell must implement.
//! The Rust core (tab-manager, memory-manager, etc.) is platform-agnostic.
//! Platform-specific code lives in the shell crates and implements these traits.

pub mod memory;
pub mod process;
pub mod renderer;
pub mod window;

pub use memory::MemoryProvider;
pub use process::ProcessMonitor;
pub use renderer::Renderer;
pub use window::Window;
