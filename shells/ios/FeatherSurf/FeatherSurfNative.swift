import Foundation

/// C FFI bridge to the FeatherSurf Rust core.
///
/// The static library (libfeathersurf_ffi.a) is linked at build time.
/// This module provides Swift-friendly wrappers around the C functions.
class FeatherSurfNative {

    // MARK: - C Function Declarations

    /// Opaque handle to a Tab instance (from Rust).
    typealias FfiTab = OpaquePointer

    /// Create a new tab.
    /// - Parameter url: The initial URL string.
    /// - Returns: An opaque handle to the tab, or nil on failure.
    static func tabCreate(url: String) -> FfiTab? {
        return url.withCString { cUrl in
            feathersurf_tab_create(0, cUrl)
        }
    }

    /// Destroy a tab and free its memory.
    /// - Parameter tab: The tab handle.
    static func tabDestroy(_ tab: FfiTab) {
        feathersurf_tab_destroy(tab)
    }

    /// Get the current state of a tab.
    /// - Parameter tab: The tab handle.
    /// - Returns: A TabState value.
    static func tabState(_ tab: FfiTab) -> TabState {
        let raw = feathersurf_tab_state(tab)
        return TabState(rawValue: raw) ?? .active
    }

    /// Transition a tab to a new state.
    /// - Parameters:
    ///   - tab: The tab handle.
    ///   - state: The target state.
    /// - Returns: true on success, false on failure.
    static func tabTransition(_ tab: FfiTab, to state: TabState) -> Bool {
        return feathersurf_tab_transition(tab, state.rawValue) == 0
    }

    /// Get the current URL of a tab.
    /// - Parameter tab: The tab handle.
    /// - Returns: The URL string, or nil.
    static func tabUrl(_ tab: FfiTab) -> String? {
        guard let cStr = feathersurf_tab_url(tab) else { return nil }
        return String(cString: cStr)
    }

    /// Get the current title of a tab.
    /// - Parameter tab: The tab handle.
    /// - Returns: The title string, or nil.
    static func tabTitle(_ tab: FfiTab) -> String? {
        guard let cStr = feathersurf_tab_title(tab) else { return nil }
        let result = String(cString: cStr)
        feathersurf_string_free(cStr)
        return result
    }
}
