// FeatherSurf CEF Handler Implementation
//
// Connects CEF browser callbacks to the Rust FFI bridge for tab lifecycle
// management and memory scoring.

#include "cef_handler.h"
#include "feathersurf_ffi.h"

#include <include/cef_app.h>
#include <include/cef_browser.h>

#include <map>
#include <atomic>
#include <chrono>

namespace {

// Global state for FFI bridge
FfiEvictionManager* g_eviction_mgr = nullptr;
std::atomic<uint64_t> g_next_tab_id{1};

// Initialize the eviction manager (call once at startup)
void EnsureEvictionManager() {
    if (!g_eviction_mgr) {
        // mode: 1 = Balanced
        g_eviction_mgr = feathersurf_eviction_create(1);
    }
}

uint64_t NowSeconds() {
    return static_cast<uint64_t>(
        std::chrono::duration_cast<std::chrono::seconds>(
            std::chrono::system_clock::now().time_since_epoch())
            .count());
}

}  // namespace

// -- Lifecycle --------------------------------------------------------

bool CefHandler::OnBeforePopup(CefRefPtr<CefBrowser> browser,
                                CefRefPtr<CefFrame> frame,
                                const CefString& target_url,
                                const CefString& target_frame_name,
                                WindowOpenDisposition target_disposition,
                                bool user_gesture,
                                const CefPopupFeatures& popupFeatures,
                                CefWindowInfo& windowInfo,
                                CefRefPtr<CefClient>& client,
                                CefBrowserSettings& settings,
                                CefRefPtr<CefDictionaryValue>& extra_info,
                                bool* no_javascript_access) {
    // Open popups in a new tab instead of a separate window
    return false;
}

void CefHandler::OnAfterCreated(CefRefPtr<CefBrowser> browser) {
    EnsureEvictionManager();

    int64_t browser_id = browser->GetIdentifier();
    uint64_t tab_id = g_next_tab_id++;

    // Register with the Rust tab manager via FFI
    FfiTab* tab = feathersurf_tab_create(tab_id, "about:blank");
    if (tab) {
        feathersurf_eviction_add_tab(g_eviction_mgr, tab, NowSeconds());

        browser_to_tab_[browser_id] = tab_id;
        tab_to_browser_[tab_id] = browser;
        tab_handles_[tab_id] = tab;
    }
}

bool CefHandler::DoClose(CefRefPtr<CefBrowser> browser) {
    return false;
}

void CefHandler::OnBeforeClose(CefRefPtr<CefBrowser> browser) {
    int64_t browser_id = browser->GetIdentifier();

    auto it = browser_to_tab_.find(browser_id);
    if (it != browser_to_tab_.end()) {
        uint64_t tab_id = it->second;

        // Destroy the FFI tab handle
        auto handle_it = tab_handles_.find(tab_id);
        if (handle_it != tab_handles_.end()) {
            feathersurf_tab_destroy(handle_it->second);
            tab_handles_.erase(handle_it);
        }

        browser_to_tab_.erase(it);
        tab_to_browser_.erase(tab_id);
    }
}

// -- Load events ------------------------------------------------------

void CefHandler::OnLoadStart(CefRefPtr<CefBrowser> browser,
                              CefRefPtr<CefFrame> frame,
                              TransitionType transition_type) {
    if (!frame->IsMain()) return;

    int64_t browser_id = browser->GetIdentifier();
    auto it = browser_to_tab_.find(browser_id);
    if (it == browser_to_tab_.end()) return;

    uint64_t tab_id = it->second;

    // Transition tab to RecentlyActive on navigation
    auto handle_it = tab_handles_.find(tab_id);
    if (handle_it != tab_handles_.end()) {
        feathersurf_tab_transition(handle_it->second, 1);  // RecentlyActive
    }
}

void CefHandler::OnLoadEnd(CefRefPtr<CefBrowser> browser,
                            CefRefPtr<CefFrame> frame,
                            int httpStatusCode) {
    if (!frame->IsMain()) return;

    int64_t browser_id = browser->GetIdentifier();
    auto it = browser_to_tab_.find(browser_id);
    if (it == browser_to_tab_.end()) return;

    uint64_t tab_id = it->second;

    // Transition tab to Background after load completes
    auto handle_it = tab_handles_.find(tab_id);
    if (handle_it != tab_handles_.end()) {
        feathersurf_tab_transition(handle_it->second, 2);  // Background
    }
}

void CefHandler::OnLoadError(CefRefPtr<CefBrowser> browser,
                              CefRefPtr<CefFrame> frame,
                              ErrorCode errorCode,
                              const CefString& errorText,
                              const CefString& failedUrl) {
    if (!frame->IsMain()) return;

    // User navigated away - not an error
    if (errorCode == ERR_ABORTED) return;

    // Show error page
    std::wstring error_html = L"<html><head><style>"
        L"body { font-family: -apple-system, sans-serif; text-align: center; "
        L"padding: 50px; background: #f5f5f5; color: #333; }"
        L"h2 { color: #d32f2f; }"
        L".url { color: #666; font-size: 14px; word-break: break-all; }"
        L"</style></head><body>"
        L"<h2>Failed to load page</h2>"
        L"<p class='url'>" + failedUrl.ToString() + L"</p>"
        L"<p>" + errorText.ToString() + L"</p>"
        L"<p><small>Error code: " + std::to_wstring(errorCode) + L"</small></p>"
        L"</body></html>";

    frame->LoadURL(CefString(L"data:text/html," + error_html));
}

// -- Navigation -------------------------------------------------------

bool CefHandler::OnBeforeBrowse(CefRefPtr<CefBrowser> browser,
                                 CefRefPtr<CefFrame> frame,
                                 CefRefPtr<CefRequest> request,
                                 bool user_gesture,
                                 bool is_redirect) {
    if (!frame->IsMain()) return false;

    // TODO: Check privacy rules (tracker blocking, etc.)
    return false;  // Allow navigation
}

// -- Tab lookup -------------------------------------------------------

CefRefPtr<CefBrowser> CefHandler::GetBrowserForTab(int64_t tab_id) const {
    auto it = tab_to_browser_.find(tab_id);
    if (it != tab_to_browser_.end()) {
        return it->second;
    }
    return nullptr;
}

int64_t CefHandler::GetTabIdForBrowser(CefRefPtr<CefBrowser> browser) const {
    if (!browser) return -1;
    auto it = browser_to_tab_.find(browser->GetIdentifier());
    if (it != browser_to_tab_.end()) {
        return it->second;
    }
    return -1;
}

// -- Freeze/Suspend/Restore -------------------------------------------

bool CefHandler::FreezeTab(int64_t tab_id) {
    auto browser = GetBrowserForTab(tab_id);
    if (!browser) return false;

    // Freeze the browser by dispatching a freeze event via JS
    CefRefPtr<CefBrowserHost> host = browser->GetHost();
    if (!host) return false;

    std::string script =
        "{"
        "  const freezeEvent = new Event('freeze');"
        "  document.dispatchEvent(freezeEvent);"
        "}";
    host->ExecuteJavaScript(CefString(script), CefString("about:blank"), 0);

    // Mark as frozen in the Rust eviction manager (state 3 = Frozen)
    auto handle_it = tab_handles_.find(tab_id);
    if (handle_it != tab_handles_.end()) {
        feathersurf_tab_transition(handle_it->second, 3);
    }

    return true;
}

bool CefHandler::SuspendTab(int64_t tab_id) {
    auto browser = GetBrowserForTab(tab_id);
    if (!browser) return false;

    // Close the browser process, keeping tab data
    CefRefPtr<CefBrowserHost> host = browser->GetHost();
    if (host) {
        host->CloseBrowser(true);
    }

    // Mark as suspended (state 4 = Suspended)
    auto handle_it = tab_handles_.find(tab_id);
    if (handle_it != tab_handles_.end()) {
        feathersurf_tab_transition(handle_it->second, 4);
    }

    return true;
}

bool CefHandler::RestoreTab(int64_t tab_id, const std::string& url) {
    // Check if already has a browser
    auto browser = GetBrowserForTab(tab_id);
    if (browser) return false;

    // Mark as active in the Rust eviction manager (state 0 = Active)
    auto handle_it = tab_handles_.find(tab_id);
    if (handle_it != tab_handles_.end()) {
        feathersurf_tab_transition(handle_it->second, 0);
    }

    // TODO: Create a new CEF browser for the tab
    // This requires the parent HWND which should come from BrowserWindow

    return true;
}
