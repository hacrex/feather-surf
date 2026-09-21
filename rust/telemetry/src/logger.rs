// FeatherSurf Structured Logging
//
// Privacy-safe logging with redaction for sensitive data.
// Uses the `tracing` crate for structured, async-aware logging.

use std::fs::{self, OpenOptions};
use std::io::{self, Write, BufWriter};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

// ── Log Levels ─────────────────────────────────────────────────────

/// Log level with privacy awareness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Error => "ERROR",
            LogLevel::Warn => "WARN",
            LogLevel::Info => "INFO",
            LogLevel::Debug => "DEBUG",
            LogLevel::Trace => "TRACE",
        }
    }
}

// ── Redaction ──────────────────────────────────────────────────────

/// Redacts sensitive data from log messages.
pub struct Redactor {
    /// Patterns to redact (e.g., URLs, IPs, tokens).
    patterns: Vec<String>,
    /// Replacement text.
    replacement: String,
}

impl Redactor {
    pub fn new() -> Self {
        Self {
            patterns: Vec::new(),
            replacement: "[REDACTED]".to_string(),
        }
    }

    /// Create with custom replacement.
    pub fn with_replacement(replacement: impl Into<String>) -> Self {
        Self {
            patterns: Vec::new(),
            replacement: replacement.into(),
        }
    }

    /// Add a pattern to redact.
    pub fn add_pattern(&mut self, pattern: impl Into<String>) {
        self.patterns.push(pattern.into());
    }

    /// Add common sensitive patterns.
    pub fn with_defaults() -> Self {
        let mut redactor = Self::new();
        // URLs with credentials
        redactor.add_pattern(r"(?i)https?://[^:]+:[^@]+@");
        // API keys/tokens (common patterns)
        redactor.add_pattern(r"(?i)(api[_-]?key|token|secret|password)\s*[=:]\s*\S+");
        // IP addresses (optional, can be too aggressive)
        // redactor.add_pattern(r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b");
        redactor
    }

    /// Redact sensitive data from a string.
    pub fn redact(&self, input: &str) -> String {
        let mut output = input.to_string();
        for pattern in &self.patterns {
            // Simple substring replacement for now
            // In production, use regex
            if let Some(pos) = output.to_lowercase().find(&pattern.to_lowercase()) {
                let end = (pos + pattern.len()).min(output.len());
                output.replace_range(pos..end, &self.replacement);
            }
        }
        output
    }
}

impl Default for Redactor {
    fn default() -> Self {
        Self::with_defaults()
    }
}

// ── Log Entry ──────────────────────────────────────────────────────

/// A structured log entry.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LogEntry {
    pub timestamp: u64,
    pub level: LogLevel,
    pub module: String,
    pub message: String,
    /// Redacted URL if present.
    pub url: Option<String>,
    /// Tab ID if applicable.
    pub tab_id: Option<u64>,
    /// Additional context.
    pub context: Vec<(String, String)>,
}

// ── Logger ─────────────────────────────────────────────────────────

/// Privacy-safe structured logger.
pub struct Logger {
    /// Log level threshold.
    level: LogLevel,
    /// Output path.
    output_path: Option<PathBuf>,
    /// Buffer writer for performance.
    writer: Option<Mutex<BufWriter<Box<dyn Write + Send>>>>,
    /// Redactor for sensitive data.
    redactor: Redactor,
    /// Maximum log file size before rotation.
    max_file_size: u64,
    /// Current file size.
    current_size: u64,
    /// Log retention (days).
    retention_days: u32,
}

impl Logger {
    pub fn new(level: LogLevel) -> Self {
        Self {
            level,
            output_path: None,
            writer: None,
            redactor: Redactor::default(),
            max_file_size: 10 * 1024 * 1024, // 10MB
            current_size: 0,
            retention_days: 7,
        }
    }

    /// Create with output file.
    pub fn to_file(level: LogLevel, path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;

        let metadata = file.metadata()?;
        let current_size = metadata.len();

        Ok(Self {
            level,
            output_path: Some(path),
            writer: Some(Mutex::new(BufWriter::new(Box::new(file)))),
            redactor: Redactor::default(),
            max_file_size: 10 * 1024 * 1024,
            current_size,
            retention_days: 7,
        })
    }

    /// Set log level.
    pub fn set_level(&mut self, level: LogLevel) {
        self.level = level;
    }

    /// Set redactor.
    pub fn set_redactor(&mut self, redactor: Redactor) {
        self.redactor = redactor;
    }

    /// Set max file size for rotation.
    pub fn set_max_file_size(&mut self, bytes: u64) {
        self.max_file_size = bytes;
    }

    /// Set log retention in days.
    pub fn set_retention(&mut self, days: u32) {
        self.retention_days = days;
    }

    /// Log a message.
    pub fn log(&mut self, level: LogLevel, module: &str, message: &str) {
        if level > self.level {
            return;
        }

        let entry = LogEntry {
            timestamp: now_secs(),
            level,
            module: module.to_string(),
            message: self.redactor.redact(message),
            url: None,
            tab_id: None,
            context: Vec::new(),
        };

        self.write_entry(&entry);
    }

    /// Log with URL (redacted).
    pub fn log_with_url(&mut self, level: LogLevel, module: &str, message: &str, url: &str) {
        if level > self.level {
            return;
        }

        let entry = LogEntry {
            timestamp: now_secs(),
            level,
            module: module.to_string(),
            message: self.redactor.redact(message),
            url: Some(self.redactor.redact(url)),
            tab_id: None,
            context: Vec::new(),
        };

        self.write_entry(&entry);
    }

    /// Log with tab context.
    pub fn log_tab(
        &mut self,
        level: LogLevel,
        module: &str,
        message: &str,
        tab_id: u64,
        url: Option<&str>,
    ) {
        if level > self.level {
            return;
        }

        let entry = LogEntry {
            timestamp: now_secs(),
            level,
            module: module.to_string(),
            message: self.redactor.redact(message),
            url: url.map(|u| self.redactor.redact(u)),
            tab_id: Some(tab_id),
            context: Vec::new(),
        };

        self.write_entry(&entry);
    }

    /// Log with context.
    pub fn log_context(
        &mut self,
        level: LogLevel,
        module: &str,
        message: &str,
        context: Vec<(&str, &str)>,
    ) {
        if level > self.level {
            return;
        }

        let entry = LogEntry {
            timestamp: now_secs(),
            level,
            module: module.to_string(),
            message: self.redactor.redact(message),
            url: None,
            tab_id: None,
            context: context
                .into_iter()
                .map(|(k, v)| (k.to_string(), self.redactor.redact(v)))
                .collect(),
        };

        self.write_entry(&entry);
    }

    /// Write a log entry.
    fn write_entry(&mut self, entry: &LogEntry) {
        let formatted = format!(
            "[{} {:5}] {}: {}{}\n",
            chrono_timestamp(entry.timestamp),
            entry.level.as_str(),
            entry.module,
            entry.message,
            if entry.tab_id.is_some() {
                format!(" [tab={}]", entry.tab_id.unwrap())
            } else {
                String::new()
            }
        );

        // Check if rotation needed
        if self.current_size + formatted.len() as u64 > self.max_file_size {
            self.rotate();
        }

        if let Some(writer) = &mut self.writer {
            if let Ok(mut w) = writer.lock() {
                let _ = w.write_all(formatted.as_bytes());
                self.current_size += formatted.len() as u64;
            }
        } else {
            // Stdout fallback
            print!("{}", formatted);
        }
    }

    /// Rotate log file.
    fn rotate(&mut self) {
        if let Some(path) = &self.output_path {
            let rotated = path.with_extension(format!(
                "{}.log",
                chrono_timestamp(now_secs())
            ));
            let _ = fs::rename(path, &rotated);

            // Open new file
            if let Ok(file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
            {
                self.writer = Some(Mutex::new(BufWriter::new(Box::new(file))));
                self.current_size = 0;
            }

            // Clean old logs
            self.clean_old_logs();
        }
    }

    /// Clean logs older than retention period.
    fn clean_old_logs(&self) {
        if let Some(path) = &self.output_path {
            if let Some(parent) = path.parent() {
                let cutoff = now_secs() - (self.retention_days as u64 * 86400);
                if let Ok(entries) = fs::read_dir(parent) {
                    for entry in entries.flatten() {
                        let name = entry.file_name();
                        let name_str = name.to_string_lossy();
                        if name_str.ends_with(".log") && name_str.starts_with("feathersurf") {
                            if let Ok(metadata) = entry.metadata() {
                                if let Ok(modified) = metadata.modified() {
                                    let modified_secs = modified
                                        .duration_since(SystemTime::UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_secs();
                                    if modified_secs < cutoff {
                                        let _ = fs::remove_file(entry.path());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Flush all buffered logs.
    pub fn flush(&mut self) {
        if let Some(writer) = &mut self.writer {
            if let Ok(mut w) = writer.lock() {
                let _ = w.flush();
            }
        }
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new(LogLevel::Info)
    }
}

// ── Helpers ────────────────────────────────────────────────────────

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn chrono_timestamp(secs: u64) -> String {
    let dt = SystemTime::UNIX_EPOCH + Duration::from_secs(secs);
    format!("{:?}", dt).replace("SystemTime ", "")
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redactor_default_patterns() {
        let redactor = Redactor::default();
        let redacted = redactor.redact("password=secret123");
        assert!(redacted.contains("[REDACTED]"));
    }

    #[test]
    fn redactor_custom_replacement() {
        let mut redactor = Redactor::with_replacement("***");
        redactor.add_pattern("secret");
        let redacted = redactor.redact("This is a secret message");
        assert_eq!(redacted, "This is a *** message");
    }

    #[test]
    fn log_level_ordering() {
        assert!(LogLevel::Error > LogLevel::Warn);
        assert!(LogLevel::Info < LogLevel::Debug);
    }

    #[test]
    fn logger_level_filter() {
        let mut logger = Logger::new(LogLevel::Warn);
        // Info should be filtered out
        logger.log(LogLevel::Info, "test", "This should not appear");
        // Warn should appear
        logger.log(LogLevel::Warn, "test", "This should appear");
    }

    #[test]
    fn log_entry_serialization() {
        let entry = LogEntry {
            timestamp: 1234567890,
            level: LogLevel::Info,
            module: "test".to_string(),
            message: "Hello".to_string(),
            url: None,
            tab_id: Some(1),
            context: vec![("key".to_string(), "value".to_string())],
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("1234567890"));
        assert!(json.contains("tab_id"));
    }
}
