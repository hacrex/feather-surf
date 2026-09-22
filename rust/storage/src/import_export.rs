// FeatherSurf Import/Export
//
// Import and export bookmarks and history in HTML and JSON formats.

use crate::{Bookmark, BookmarkFolder, BookmarkStore, HistoryStore, TransitionType};

// ── HTML Bookmark Export ───────────────────────────────────────────

/// Export bookmarks to Netscape Bookmark Format (HTML).
pub fn export_bookmarks_html(store: &BookmarkStore) -> String {
    let mut html = String::from(
        "<!DOCTYPE NETSCAPE-Bookmark-file-1>\n\
         <!-- This is an automatically generated file. -->\n\
         <META HTTP-EQUIV=\"Content-Type\" CONTENT=\"text/html; charset=UTF-8\">\n\
         <TITLE>Bookmarks</TITLE>\n\
         <H1>Bookmarks</H1>\n\
         <DL><p>\n",
    );

    // Export root-level folders
    for folder in store.folders_in_parent(None) {
        html.push_str(&export_folder_html(store, folder, 1));
    }

    // Export root-level bookmarks
    for bookmark in store.bookmarks_in_folder(None) {
        html.push_str(&export_bookmark_html(bookmark, 1));
    }

    html.push_str("</DL><p>\n");
    html
}

fn export_folder_html(store: &BookmarkStore, folder: &BookmarkFolder, depth: usize) -> String {
    let indent = "    ".repeat(depth);
    let mut html = format!(
        "{}<DT><H3>{}</H3>\n{}<DL><p>\n",
        indent, folder.name, indent
    );

    // Subfolders
    for subfolder in store.folders_in_parent(Some(folder.id)) {
        html.push_str(&export_folder_html(store, subfolder, depth + 1));
    }

    // Bookmarks in this folder
    for bookmark in store.bookmarks_in_folder(Some(folder.id)) {
        html.push_str(&export_bookmark_html(bookmark, depth + 1));
    }

    html.push_str(&format!("{}</DL><p>\n", indent));
    html
}

fn export_bookmark_html(bookmark: &Bookmark, depth: usize) -> String {
    let indent = "    ".repeat(depth);
    format!(
        "{}<DT><A HREF=\"{}\" ADD_DATE=\"{}\">{}</A>\n",
        indent, bookmark.url, bookmark.created_at, bookmark.title
    )
}

// ── HTML Bookmark Import ───────────────────────────────────────────

/// Import bookmarks from Netscape Bookmark Format (HTML).
pub fn import_bookmarks_html(store: &mut BookmarkStore, html: &str) -> Result<usize, String> {
    let mut count = 0;
    let mut current_folder: Option<u64> = None;
    let mut folder_stack: Vec<u64> = Vec::new();

    for line in html.lines() {
        let line = line.trim();

        if line.starts_with("<DT><H3") {
            // Parse folder
            if let Some(name) = extract_tag_content(line, "H3") {
                let folder_id = store.add_folder(&name, current_folder);
                folder_stack.push(folder_id);
                current_folder = Some(folder_id);
                count += 1;
            }
        } else if line == "</DL>" {
            // End of folder
            folder_stack.pop();
            current_folder = folder_stack.last().copied();
        } else if line.starts_with("<DT><A") {
            // Parse bookmark
            if let Some((url, title)) = extract_anchor(line) {
                store.add_bookmark(&url, &title, current_folder);
                count += 1;
            }
        }
    }

    Ok(count)
}

// ── JSON Export ────────────────────────────────────────────────────

/// Export bookmarks to JSON.
pub fn export_bookmarks_json(store: &BookmarkStore) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&store.export_all())
}

/// Export history to JSON.
pub fn export_history_json(store: &HistoryStore) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&store.export_all())
}

/// Export all data to JSON.
pub fn export_all_json(
    bookmarks: &BookmarkStore,
    history: &HistoryStore,
) -> Result<String, serde_json::Error> {
    let data = serde_json::json!({
        "bookmarks": bookmarks.export_all(),
        "history": history.export_all(),
    });
    serde_json::to_string_pretty(&data)
}

// ── JSON Import ────────────────────────────────────────────────────

/// Import bookmarks from JSON.
pub fn import_bookmarks_json(store: &mut BookmarkStore, json: &str) -> Result<usize, String> {
    let entries: Vec<BookmarkEntry> =
        serde_json::from_str(json).map_err(|e| format!("Invalid JSON: {}", e))?;

    let mut count = 0;
    for entry in entries {
        store.add_bookmark(&entry.url, &entry.title, entry.folder_id);
        count += 1;
    }

    Ok(count)
}

/// Import history from JSON.
pub fn import_history_json(store: &mut HistoryStore, json: &str) -> Result<usize, String> {
    let entries: Vec<HistoryEntryData> =
        serde_json::from_str(json).map_err(|e| format!("Invalid JSON: {}", e))?;

    let mut count = 0;
    for entry in entries {
        store.record(&entry.url, &entry.title, entry.transition_type);
        count += 1;
    }

    Ok(count)
}

// ── Types ──────────────────────────────────────────────────────────

#[derive(serde::Serialize, serde::Deserialize)]
struct BookmarkEntry {
    url: String,
    title: String,
    folder_id: Option<u64>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct HistoryEntryData {
    url: String,
    title: String,
    transition_type: TransitionType,
}

// ── Helpers ────────────────────────────────────────────────────────

fn extract_tag_content(line: &str, tag: &str) -> Option<String> {
    let start = format!("<{}>", tag);
    let end = format!("</{}>", tag);

    if let Some(s) = line.find(&start) {
        let content_start = s + start.len();
        if let Some(e) = line[content_start..].find(&end) {
            return Some(line[content_start..content_start + e].to_string());
        }
    }

    // Try with attributes: <H3 ...>content</H3>
    let start = format!("<{} ", tag);
    if let Some(s) = line.find(&start) {
        if let Some(close) = line[s..].find('>') {
            let content_start = s + close + 1;
            if let Some(e) = line[content_start..].find(&end) {
                return Some(line[content_start..content_start + e].to_string());
            }
        }
    }

    None
}

fn extract_anchor(line: &str) -> Option<(String, String)> {
    // Extract HREF="..."
    let href = if let Some(s) = line.find("HREF=\"") {
        let start = s + 6;
        if let Some(e) = line[start..].find('"') {
            line[start..start + e].to_string()
        } else {
            return None;
        }
    } else {
        return None;
    };

    // Extract title (content after the > that closes the opening <A ...> tag)
    let title = if let Some(after_href) = line.find("HREF=\"") {
        let after_href_quote = after_href + 6;
        if let Some(quote_end) = line[after_href_quote..].find('"') {
            let after_quote = after_href_quote + quote_end + 1;
            if let Some(gt) = line[after_quote..].find('>') {
                let content_start = after_quote + gt + 1;
                if let Some(e) = line[content_start..].find('<') {
                    line[content_start..content_start + e].to_string()
                } else {
                    line[content_start..].to_string()
                }
            } else {
                href.clone()
            }
        } else {
            href.clone()
        }
    } else {
        href.clone()
    };

    Some((href, title))
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_import_html_roundtrip() {
        let mut store = BookmarkStore::new();
        let folder = store.add_folder("Work", None);
        store.add_bookmark("https://example.com", "Example", Some(folder));
        store.add_bookmark("https://rust-lang.org", "Rust", None);

        let html = export_bookmarks_html(&store);
        assert!(html.contains("Example"));
        assert!(html.contains("Rust"));

        let mut new_store = BookmarkStore::new();
        let count = import_bookmarks_html(&mut new_store, &html).unwrap();
        assert!(count >= 2);
    }

    #[test]
    fn export_import_json_roundtrip() {
        let mut store = BookmarkStore::new();
        store.add_bookmark("https://example.com", "Example", None);
        store.add_bookmark("https://rust-lang.org", "Rust", None);

        let json = export_bookmarks_json(&store).unwrap();
        let mut new_store = BookmarkStore::new();
        let count = import_bookmarks_json(&mut new_store, &json).unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn history_export_import() {
        let mut store = HistoryStore::new();
        store.record("https://example.com", "Example", TransitionType::Typed);
        store.record("https://rust-lang.org", "Rust", TransitionType::Link);

        let json = export_history_json(&store).unwrap();
        let mut new_store = HistoryStore::new();
        let count = import_history_json(&mut new_store, &json).unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn extract_tag_content_test() {
        assert_eq!(
            extract_tag_content("<H3>Work</H3>", "H3"),
            Some("Work".to_string())
        );
        assert_eq!(
            extract_tag_content("<H3 ATTR=\"val\">Work</H3>", "H3"),
            Some("Work".to_string())
        );
    }

    #[test]
    fn extract_anchor_test() {
        let line = "<DT><A HREF=\"https://example.com\" ADD_DATE=\"123\">Example</A>";
        let (url, title) = extract_anchor(line).unwrap();
        assert_eq!(url, "https://example.com");
        assert_eq!(title, "Example");
    }
}
