// FeatherSurf Download Manager
//
// Manages file downloads with pause, resume, cancel, and retry support.
// Validates filenames and prevents path traversal attacks.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

// ── Download State ─────────────────────────────────────────────────

/// Unique download identifier.
pub type DownloadId = u64;

/// State of a download job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DownloadState {
    /// Download is being set up.
    Starting,
    /// Download is in progress.
    Downloading,
    /// Download is paused.
    Paused,
    /// Download completed successfully.
    Completed,
    /// Download failed.
    Failed,
    /// Download was cancelled by user.
    Cancelled,
    /// Download is being scanned for malware.
    Scanning,
}

/// A download job.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Download {
    /// Unique download ID.
    pub id: DownloadId,
    /// Source URL.
    pub url: String,
    /// Suggested filename from Content-Disposition.
    pub suggested_filename: String,
    /// Final filename (sanitized).
    pub filename: String,
    /// Destination directory.
    pub directory: String,
    /// MIME type.
    pub mime_type: String,
    /// Total bytes (-1 if unknown).
    pub total_bytes: i64,
    /// Bytes downloaded so far.
    pub downloaded_bytes: u64,
    /// Current state.
    pub state: DownloadState,
    /// Error message if failed.
    pub error: Option<String>,
    /// When the download started.
    pub started_at: u64,
    /// When the download was last updated.
    pub updated_at: u64,
    /// When the download completed (if it did).
    pub completed_at: Option<u64>,
    /// Download speed (bytes per second).
    pub speed: u64,
    /// Whether the file is a resume download.
    pub resumable: bool,
    /// SHA-256 hash if known.
    pub hash: Option<String>,
}

/// Download event for UI updates.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum DownloadEvent {
    /// Download started.
    Started(DownloadId),
    /// Download progress updated.
    Progress {
        id: DownloadId,
        downloaded: u64,
        total: i64,
        speed: u64,
    },
    /// Download paused.
    Paused(DownloadId),
    /// Download resumed.
    Resumed(DownloadId),
    /// Download completed.
    Completed(DownloadId),
    /// Download failed.
    Failed { id: DownloadId, error: String },
    /// Download cancelled.
    Cancelled(DownloadId),
}

// ── Download Manager ───────────────────────────────────────────────

/// Manages file downloads.
pub struct DownloadManager {
    /// Active downloads.
    downloads: HashMap<DownloadId, Download>,
    /// Next download ID.
    next_id: DownloadId,
    /// Default download directory.
    download_dir: PathBuf,
    /// Maximum concurrent downloads.
    max_concurrent: usize,
    /// Download history.
    history: Vec<Download>,
    /// Maximum history size.
    max_history: usize,
}

impl DownloadManager {
    pub fn new(download_dir: impl Into<PathBuf>) -> Self {
        Self {
            downloads: HashMap::new(),
            next_id: 1,
            download_dir: download_dir.into(),
            max_concurrent: 3,
            history: Vec::new(),
            max_history: 100,
        }
    }

    /// Create with custom settings.
    pub fn with_settings(
        download_dir: impl Into<PathBuf>,
        max_concurrent: usize,
        max_history: usize,
    ) -> Self {
        Self {
            download_dir: download_dir.into(),
            max_concurrent,
            max_history,
            ..Self::default()
        }
    }

    /// Start a new download.
    pub fn start_download(
        &mut self,
        url: impl Into<String>,
        suggested_filename: impl Into<String>,
        mime_type: impl Into<String>,
    ) -> Result<DownloadId, DownloadError> {
        // Check concurrent limit
        let active = self
            .downloads
            .values()
            .filter(|d| d.state == DownloadState::Downloading || d.state == DownloadState::Starting)
            .count();

        if active >= self.max_concurrent {
            return Err(DownloadError::TooManyConcurrent);
        }

        let url = url.into();
        let suggested = suggested_filename.into();
        let mime = mime_type.into();

        // Sanitize filename
        let filename = sanitize_filename(&suggested);
        let directory = self.download_dir.to_string_lossy().to_string();

        let id = self.next_id;
        self.next_id += 1;

        let now = now_secs();
        let download = Download {
            id,
            url,
            suggested_filename: suggested,
            filename: filename.clone(),
            directory: directory.clone(),
            mime_type: mime,
            total_bytes: -1,
            downloaded_bytes: 0,
            state: DownloadState::Starting,
            error: None,
            started_at: now,
            updated_at: now,
            completed_at: None,
            speed: 0,
            resumable: false,
            hash: None,
        };

        self.downloads.insert(id, download);
        Ok(id)
    }

    /// Pause a download.
    pub fn pause(&mut self, id: DownloadId) -> Result<(), DownloadError> {
        let download = self.downloads.get_mut(&id).ok_or(DownloadError::NotFound)?;

        if download.state != DownloadState::Downloading {
            return Err(DownloadError::InvalidState);
        }

        download.state = DownloadState::Paused;
        download.updated_at = now_secs();
        Ok(())
    }

    /// Resume a paused download.
    pub fn resume(&mut self, id: DownloadId) -> Result<(), DownloadError> {
        let download = self.downloads.get_mut(&id).ok_or(DownloadError::NotFound)?;

        if download.state != DownloadState::Paused {
            return Err(DownloadError::InvalidState);
        }

        download.state = DownloadState::Downloading;
        download.updated_at = now_secs();
        Ok(())
    }

    /// Cancel a download.
    pub fn cancel(&mut self, id: DownloadId) -> Result<(), DownloadError> {
        let download = self.downloads.get_mut(&id).ok_or(DownloadError::NotFound)?;

        if download.state == DownloadState::Completed {
            return Err(DownloadError::AlreadyCompleted);
        }

        download.state = DownloadState::Cancelled;
        download.updated_at = now_secs();

        // Move to history
        if let Some(dl) = self.downloads.remove(&id) {
            self.add_to_history(dl);
        }

        Ok(())
    }

    /// Retry a failed or cancelled download.
    pub fn retry(&mut self, id: DownloadId) -> Result<(), DownloadError> {
        // Check if it's in history
        if let Some(download) = self.history.iter_mut().find(|d| d.id == id) {
            if download.state == DownloadState::Failed || download.state == DownloadState::Cancelled
            {
                download.state = DownloadState::Starting;
                download.downloaded_bytes = 0;
                download.error = None;
                download.started_at = now_secs();
                download.updated_at = now_secs();
                download.completed_at = None;

                // Move back to active
                let dl = self
                    .history
                    .remove(self.history.iter().position(|d| d.id == id).unwrap());
                self.downloads.insert(id, dl);
                return Ok(());
            }
        }

        Err(DownloadError::NotFound)
    }

    /// Update download progress.
    pub fn update_progress(
        &mut self,
        id: DownloadId,
        downloaded: u64,
        total: i64,
        speed: u64,
    ) -> Result<(), DownloadError> {
        let download = self.downloads.get_mut(&id).ok_or(DownloadError::NotFound)?;

        download.downloaded_bytes = downloaded;
        download.total_bytes = total;
        download.speed = speed;
        download.updated_at = now_secs();

        if download.state == DownloadState::Starting {
            download.state = DownloadState::Downloading;
        }

        Ok(())
    }

    /// Mark a download as completed.
    pub fn complete(&mut self, id: DownloadId) -> Result<(), DownloadError> {
        let download = self.downloads.get_mut(&id).ok_or(DownloadError::NotFound)?;

        download.state = DownloadState::Completed;
        download.completed_at = Some(now_secs());
        download.updated_at = now_secs();
        download.speed = 0;

        // Move to history
        if let Some(dl) = self.downloads.remove(&id) {
            self.add_to_history(dl);
        }

        Ok(())
    }

    /// Mark a download as failed.
    pub fn fail(&mut self, id: DownloadId, error: impl Into<String>) -> Result<(), DownloadError> {
        let download = self.downloads.get_mut(&id).ok_or(DownloadError::NotFound)?;

        download.state = DownloadState::Failed;
        download.error = Some(error.into());
        download.updated_at = now_secs();
        download.speed = 0;

        // Move to history
        if let Some(dl) = self.downloads.remove(&id) {
            self.add_to_history(dl);
        }

        Ok(())
    }

    /// Get a download by ID.
    pub fn get(&self, id: DownloadId) -> Option<&Download> {
        self.downloads
            .get(&id)
            .or_else(|| self.history.iter().find(|d| d.id == id))
    }

    /// Get all active downloads.
    pub fn active(&self) -> Vec<&Download> {
        self.downloads.values().collect()
    }

    /// Get download history.
    pub fn history(&self) -> &[Download] {
        &self.history
    }

    /// Get total bytes downloaded.
    pub fn total_downloaded(&self) -> u64 {
        self.history
            .iter()
            .filter(|d| d.state == DownloadState::Completed)
            .map(|d| d.downloaded_bytes)
            .sum()
    }

    /// Clear download history.
    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// Set download directory.
    pub fn set_download_dir(&mut self, dir: impl Into<PathBuf>) {
        self.download_dir = dir.into();
    }

    /// Get download directory.
    pub fn download_dir(&self) -> &Path {
        &self.download_dir
    }

    /// Add a download to history.
    fn add_to_history(&mut self, download: Download) {
        self.history.push(download);
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
    }
}

impl Default for DownloadManager {
    fn default() -> Self {
        Self::new(dirs::download_dir().unwrap_or_else(|| PathBuf::from(".")))
    }
}

// ── Filename Sanitization ──────────────────────────────────────────

/// Sanitize a filename to prevent path traversal and invalid characters.
pub fn sanitize_filename(filename: &str) -> String {
    let mut sanitized = String::new();

    for c in filename.chars() {
        match c {
            // Allow alphanumeric, hyphen, underscore, dot
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' => {
                sanitized.push(c);
            }
            // Replace spaces with underscores
            ' ' => {
                sanitized.push('_');
            }
            // Skip path separators
            '/' | '\\' => {}
            // Skip other problematic characters
            _ => {}
        }
    }

    // Remove leading dots (hidden files)
    while sanitized.starts_with('.') {
        sanitized.remove(0);
    }

    // Remove trailing dots
    while sanitized.ends_with('.') {
        sanitized.pop();
    }

    // Ensure not empty
    if sanitized.is_empty() {
        sanitized = "download".to_string();
    }

    // Limit length
    if sanitized.len() > 255 {
        sanitized.truncate(255);
    }

    sanitized
}

/// Validate a file path to prevent directory traversal.
pub fn validate_path(base: &Path, path: &Path) -> bool {
    match path.canonicalize() {
        Ok(canonical) => match base.canonicalize() {
            Ok(base_canonical) => canonical.starts_with(base_canonical),
            Err(_) => false,
        },
        Err(_) => false,
    }
}

// ── Errors ─────────────────────────────────────────────────────────

/// Download manager errors.
#[derive(Debug, Clone, thiserror::Error)]
pub enum DownloadError {
    #[error("Download not found")]
    NotFound,
    #[error("Invalid state for this operation")]
    InvalidState,
    #[error("Too many concurrent downloads")]
    TooManyConcurrent,
    #[error("Download already completed")]
    AlreadyCompleted,
    #[error("File already exists")]
    FileExists,
    #[error("Permission denied")]
    PermissionDenied,
    #[error("Disk full")]
    DiskFull,
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
    fn start_download() {
        let mut manager = DownloadManager::new("/tmp/downloads");
        let id = manager
            .start_download(
                "https://example.com/file.pdf",
                "file.pdf",
                "application/pdf",
            )
            .unwrap();
        assert!(manager.get(id).is_some());
    }

    #[test]
    fn pause_resume() {
        let mut manager = DownloadManager::new("/tmp/downloads");
        let id = manager
            .start_download(
                "https://example.com/file.pdf",
                "file.pdf",
                "application/pdf",
            )
            .unwrap();

        manager.update_progress(id, 100, 1000, 50).unwrap();
        manager.pause(id).unwrap();
        assert_eq!(manager.get(id).unwrap().state, DownloadState::Paused);

        manager.resume(id).unwrap();
        assert_eq!(manager.get(id).unwrap().state, DownloadState::Downloading);
    }

    #[test]
    fn cancel_download() {
        let mut manager = DownloadManager::new("/tmp/downloads");
        let id = manager
            .start_download(
                "https://example.com/file.pdf",
                "file.pdf",
                "application/pdf",
            )
            .unwrap();

        manager.cancel(id).unwrap();
        assert!(manager.get(id).is_some()); // Still in history
        assert!(manager.active().is_empty()); // No longer active
        assert_eq!(manager.history().len(), 1);
    }

    #[test]
    fn retry_download() {
        let mut manager = DownloadManager::new("/tmp/downloads");
        let id = manager
            .start_download(
                "https://example.com/file.pdf",
                "file.pdf",
                "application/pdf",
            )
            .unwrap();

        manager.fail(id, "Network error").unwrap();
        assert_eq!(manager.history().len(), 1);

        manager.retry(id).unwrap();
        assert_eq!(manager.active().len(), 1);
    }

    #[test]
    fn concurrent_limit() {
        let mut manager = DownloadManager::with_settings("/tmp/downloads", 2, 10);
        let _1 = manager
            .start_download("https://a.com/1", "1.pdf", "application/pdf")
            .unwrap();
        let _2 = manager
            .start_download("https://a.com/2", "2.pdf", "application/pdf")
            .unwrap();

        let result = manager.start_download("https://a.com/3", "3.pdf", "application/pdf");
        assert!(matches!(result, Err(DownloadError::TooManyConcurrent)));
    }

    #[test]
    fn sanitize_filename_test() {
        assert_eq!(sanitize_filename("hello world.txt"), "hello_world.txt");
        assert_eq!(sanitize_filename("../../../etc/passwd"), "etcpasswd");
        assert_eq!(sanitize_filename("...hidden"), "hidden");
        assert_eq!(sanitize_filename(""), "download");
    }

    #[test]
    fn total_downloaded() {
        let mut manager = DownloadManager::new("/tmp/downloads");
        let id1 = manager
            .start_download("https://a.com/1", "1.pdf", "application/pdf")
            .unwrap();
        let id2 = manager
            .start_download("https://a.com/2", "2.pdf", "application/pdf")
            .unwrap();

        manager.update_progress(id1, 100, 100, 50).unwrap();
        manager.complete(id1).unwrap();

        manager.update_progress(id2, 50, 100, 25).unwrap();
        manager.complete(id2).unwrap();

        assert_eq!(manager.total_downloaded(), 150);
    }
}
