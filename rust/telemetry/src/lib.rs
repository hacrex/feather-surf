//! telemetry — Structured logging and privacy-safe diagnostics.
//!
//! Provides privacy-safe logging with redaction for sensitive data,
//! structured log entries, and diagnostic export.

pub mod logger;

pub use logger::{Logger, LogLevel, Redactor, LogEntry};
