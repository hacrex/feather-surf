// FeatherSurf Storage
//
// Bookmarks and history storage with in-memory backend.
// Designed to be backed by SQLite in production.

pub mod import_export;
pub mod preferences;
pub mod profiles;

use std::collections::HashMap;
use std::time::SystemTime;

// ── Bookmarks ──────────────────────────────────────────────────────

/// A bookmark folder.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct BookmarkFolder {
    pub id: u64,
    pub name: String,
    pub parent_id: Option<u64>,
    pub created_at: u64,
}

/// A bookmark entry.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct Bookmark {
    pub id: u64,
    pub url: String,
    pub title: String,
    pub folder_id: Option<u64>,
    pub created_at: u64,
    pub favicon_url: Option<String>,
}

/// Bookmarks storage.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct BookmarkStore {
    folders: HashMap<u64, BookmarkFolder>,
    bookmarks: HashMap<u64, Bookmark>,
    next_id: u64,
}

impl BookmarkStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a bookmark. Returns the bookmark ID.
    pub fn add_bookmark(
        &mut self,
        url: impl Into<String>,
        title: impl Into<String>,
        folder_id: Option<u64>,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.bookmarks.insert(
            id,
            Bookmark {
                id,
                url: url.into(),
                title: title.into(),
                folder_id,
                created_at: now_secs(),
                favicon_url: None,
            },
        );
        id
    }

    /// Remove a bookmark. Returns true if it existed.
    pub fn remove_bookmark(&mut self, id: u64) -> bool {
        self.bookmarks.remove(&id).is_some()
    }

    /// Update a bookmark's title.
    pub fn update_title(&mut self, id: u64, title: impl Into<String>) -> bool {
        if let Some(bm) = self.bookmarks.get_mut(&id) {
            bm.title = title.into();
            true
        } else {
            false
        }
    }

    /// Move a bookmark to a different folder.
    pub fn move_bookmark(&mut self, id: u64, folder_id: Option<u64>) -> bool {
        if let Some(bm) = self.bookmarks.get_mut(&id) {
            bm.folder_id = folder_id;
            true
        } else {
            false
        }
    }

    /// Get a bookmark by ID.
    pub fn get_bookmark(&self, id: u64) -> Option<&Bookmark> {
        self.bookmarks.get(&id)
    }

    /// Get all bookmarks in a folder (or root if None).
    pub fn bookmarks_in_folder(&self, folder_id: Option<u64>) -> Vec<&Bookmark> {
        self.bookmarks
            .values()
            .filter(|bm| bm.folder_id == folder_id)
            .collect()
    }

    /// Search bookmarks by title or URL (case-insensitive substring).
    pub fn search_bookmarks(&self, query: &str) -> Vec<&Bookmark> {
        let q = query.to_lowercase();
        self.bookmarks
            .values()
            .filter(|bm| bm.title.to_lowercase().contains(&q) || bm.url.to_lowercase().contains(&q))
            .collect()
    }

    /// Create a folder. Returns the folder ID.
    pub fn add_folder(&mut self, name: impl Into<String>, parent_id: Option<u64>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.folders.insert(
            id,
            BookmarkFolder {
                id,
                name: name.into(),
                parent_id,
                created_at: now_secs(),
            },
        );
        id
    }

    /// Remove a folder and all its bookmarks.
    pub fn remove_folder(&mut self, id: u64) -> bool {
        if self.folders.remove(&id).is_some() {
            // Remove bookmarks in this folder
            self.bookmarks.retain(|_, bm| bm.folder_id != Some(id));
            // Remove child folders recursively
            let children: Vec<u64> = self
                .folders
                .values()
                .filter(|f| f.parent_id == Some(id))
                .map(|f| f.id)
                .collect();
            for child in children {
                self.remove_folder(child);
            }
            true
        } else {
            false
        }
    }

    /// Get a folder by ID.
    pub fn get_folder(&self, id: u64) -> Option<&BookmarkFolder> {
        self.folders.get(&id)
    }

    /// Get all folders in a parent (or root if None).
    pub fn folders_in_parent(&self, parent_id: Option<u64>) -> Vec<&BookmarkFolder> {
        self.folders
            .values()
            .filter(|f| f.parent_id == parent_id)
            .collect()
    }

    /// Get total bookmark count.
    pub fn bookmark_count(&self) -> usize {
        self.bookmarks.len()
    }

    /// Get total folder count.
    pub fn folder_count(&self) -> usize {
        self.folders.len()
    }

    /// Export all bookmarks as a flat list.
    pub fn export_all(&self) -> Vec<&Bookmark> {
        self.bookmarks.values().collect()
    }

    /// Clear all bookmarks and folders.
    pub fn clear(&mut self) {
        self.bookmarks.clear();
        self.folders.clear();
        self.next_id = 0;
    }
}

// ── History ────────────────────────────────────────────────────────

/// A history entry.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct HistoryEntry {
    pub id: u64,
    pub url: String,
    pub title: String,
    pub visited_at: u64,
    pub transition_type: TransitionType,
}

/// How the navigation happened.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TransitionType {
    /// User clicked a link.
    Link,
    /// User typed the URL.
    Typed,
    /// User navigated via bookmark.
    Bookmark,
    /// User navigated via back/forward.
    BackForward,
    /// Redirect from another page.
    Redirect,
    /// Reload.
    Reload,
}

/// History storage with retention policy.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HistoryStore {
    entries: Vec<HistoryEntry>,
    next_id: u64,
    max_entries: usize,
}

impl Default for HistoryStore {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            next_id: 0,
            max_entries: 10_000,
        }
    }
}

impl HistoryStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a history store with a custom retention limit.
    pub fn with_max_entries(max_entries: usize) -> Self {
        Self {
            max_entries,
            ..Self::default()
        }
    }

    /// Record a navigation.
    pub fn record(
        &mut self,
        url: impl Into<String>,
        title: impl Into<String>,
        transition: TransitionType,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        self.entries.push(HistoryEntry {
            id,
            url: url.into(),
            title: title.into(),
            visited_at: now_secs(),
            transition_type: transition,
        });

        // Trim oldest entries if over limit
        if self.entries.len() > self.max_entries {
            let excess = self.entries.len() - self.max_entries;
            self.entries.drain(..excess);
        }

        id
    }

    /// Search history by URL or title (case-insensitive substring).
    pub fn search(&self, query: &str) -> Vec<&HistoryEntry> {
        let q = query.to_lowercase();
        self.entries
            .iter()
            .rev() // Most recent first
            .filter(|e| e.url.to_lowercase().contains(&q) || e.title.to_lowercase().contains(&q))
            .collect()
    }

    /// Get the most recent N entries.
    pub fn recent(&self, n: usize) -> Vec<&HistoryEntry> {
        self.entries.iter().rev().take(n).collect()
    }

    /// Get entries for a specific URL.
    pub fn for_url(&self, url: &str) -> Vec<&HistoryEntry> {
        self.entries.iter().filter(|e| e.url == url).collect()
    }

    /// Get the last entry for a URL (most recent visit).
    pub fn last_visit(&self, url: &str) -> Option<&HistoryEntry> {
        self.entries.iter().rev().find(|e| e.url == url)
    }

    /// Remove entries older than N days.
    pub fn prune_old_entries(&mut self, days: u64) {
        let cutoff = now_secs().saturating_sub(days * 86400);
        self.entries.retain(|e| e.visited_at >= cutoff);
    }

    /// Remove all entries for a URL.
    pub fn remove_url(&mut self, url: &str) -> usize {
        let before = self.entries.len();
        self.entries.retain(|e| e.url != url);
        before - self.entries.len()
    }

    /// Clear all history.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.next_id = 0;
    }

    /// Get total entry count.
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Export all entries.
    pub fn export_all(&self) -> Vec<&HistoryEntry> {
        self.entries.iter().collect()
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

    // ── Bookmark tests ─────────────────────────────────────────────

    #[test]
    fn add_and_get_bookmark() {
        let mut store = BookmarkStore::new();
        let id = store.add_bookmark("https://example.com", "Example", None);
        let bm = store.get_bookmark(id).unwrap();
        assert_eq!(bm.url, "https://example.com");
        assert_eq!(bm.title, "Example");
        assert!(bm.folder_id.is_none());
    }

    #[test]
    fn remove_bookmark() {
        let mut store = BookmarkStore::new();
        let id = store.add_bookmark("https://example.com", "Example", None);
        assert!(store.remove_bookmark(id));
        assert!(store.get_bookmark(id).is_none());
    }

    #[test]
    fn update_bookmark_title() {
        let mut store = BookmarkStore::new();
        let id = store.add_bookmark("https://example.com", "Old", None);
        assert!(store.update_title(id, "New"));
        assert_eq!(store.get_bookmark(id).unwrap().title, "New");
    }

    #[test]
    fn move_bookmark_to_folder() {
        let mut store = BookmarkStore::new();
        let folder = store.add_folder("Work", None);
        let bm = store.add_bookmark("https://example.com", "Example", None);
        assert!(store.move_bookmark(bm, Some(folder)));
        let in_folder = store.bookmarks_in_folder(Some(folder));
        assert_eq!(in_folder.len(), 1);
        assert_eq!(in_folder[0].id, bm);
    }

    #[test]
    fn search_bookmarks() {
        let mut store = BookmarkStore::new();
        store.add_bookmark("https://rust-lang.org", "Rust", None);
        store.add_bookmark("https://github.com", "GitHub", None);
        store.add_bookmark("https://docs.rs", "Docs.rs", None);

        let results = store.search_bookmarks("rust");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].url, "https://rust-lang.org");

        let results = store.search_bookmarks("rs");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn add_and_remove_folder() {
        let mut store = BookmarkStore::new();
        let folder = store.add_folder("Work", None);
        store.add_bookmark("https://example.com", "Example", Some(folder));
        assert_eq!(store.bookmark_count(), 1);
        assert!(store.remove_folder(folder));
        assert_eq!(store.bookmark_count(), 0);
    }

    #[test]
    fn nested_folders() {
        let mut store = BookmarkStore::new();
        let work = store.add_folder("Work", None);
        let projects = store.add_folder("Projects", Some(work));
        store.add_bookmark("https://example.com", "Example", Some(projects));

        let in_work = store.folders_in_parent(None);
        assert_eq!(in_work.len(), 1);
        assert_eq!(in_work[0].name, "Work");

        let in_projects = store.folders_in_parent(Some(work));
        assert_eq!(in_projects.len(), 1);
    }

    // ── History tests ──────────────────────────────────────────────

    #[test]
    fn record_and_search_history() {
        let mut store = HistoryStore::new();
        store.record("https://example.com", "Example", TransitionType::Typed);
        store.record("https://rust-lang.org", "Rust", TransitionType::Link);

        let results = store.search("rust");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].url, "https://rust-lang.org");
    }

    #[test]
    fn recent_entries() {
        let mut store = HistoryStore::new();
        for i in 0..10 {
            store.record(
                format!("https://example{i}.com"),
                format!("Page {i}"),
                TransitionType::Typed,
            );
        }
        let recent = store.recent(3);
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].url, "https://example9.com");
    }

    #[test]
    fn history_pruning() {
        let mut store = HistoryStore::with_max_entries(5);
        for i in 0..10 {
            store.record(
                format!("https://example{i}.com"),
                format!("Page {i}"),
                TransitionType::Typed,
            );
        }
        assert_eq!(store.entry_count(), 5);
        assert_eq!(store.entries[0].url, "https://example5.com");
    }

    #[test]
    fn remove_url_from_history() {
        let mut store = HistoryStore::new();
        store.record("https://example.com", "Example", TransitionType::Typed);
        store.record(
            "https://example.com",
            "Example Again",
            TransitionType::Reload,
        );
        store.record("https://other.com", "Other", TransitionType::Typed);

        let removed = store.remove_url("https://example.com");
        assert_eq!(removed, 2);
        assert_eq!(store.entry_count(), 1);
    }

    #[test]
    fn last_visit() {
        let mut store = HistoryStore::new();
        store.record("https://example.com", "First", TransitionType::Typed);
        store.record("https://other.com", "Other", TransitionType::Typed);
        store.record("https://example.com", "Second", TransitionType::Reload);

        let last = store.last_visit("https://example.com").unwrap();
        assert_eq!(last.title, "Second");
    }

    #[test]
    fn prune_old_entries() {
        let mut store = HistoryStore::new();
        // All entries are recent (now), so pruning 0 days keeps nothing
        store.record("https://example.com", "Example", TransitionType::Typed);
        store.prune_old_entries(0);
        // Entry is brand new, so it should survive (visited_at == now)
        assert_eq!(store.entry_count(), 1);
    }

    #[test]
    fn clear_history() {
        let mut store = HistoryStore::new();
        store.record("https://example.com", "Example", TransitionType::Typed);
        store.clear();
        assert_eq!(store.entry_count(), 0);
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn bookmark_roundtrip(
            count in 0usize..100,
            title_len in 0usize..50,
            url_len in 1usize..100
        ) {
            let mut store = BookmarkStore::new();
            let mut ids = Vec::new();
            for i in 0..count {
                let title = format!("t{}", i % title_len.max(1));
                let url = format!("https://example{}.com", i % url_len.max(1));
                ids.push(store.add_bookmark(url, title, None));
            }
            // All IDs should be unique
            let mut unique = ids.clone();
            unique.sort();
            unique.dedup();
            prop_assert_eq!(ids.len(), unique.len());
            prop_assert_eq!(store.bookmark_count(), count);
        }

        #[test]
        fn history_pruning_respects_limit(max in 1usize..1000) {
            let mut store = HistoryStore::with_max_entries(max);
            for i in 0..(max * 2) {
                store.record(
                    format!("https://example{i}.com"),
                    format!("Page {i}"),
                    TransitionType::Typed,
                );
            }
            prop_assert!(store.entry_count() <= max);
        }
    }
}
