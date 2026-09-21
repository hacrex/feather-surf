// FeatherSurf CEF Handler Implementation

#include "cef_handler.h"

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
    // TODO: Create a new tab in the browser shell and navigate to target_url
    return false;  // Allow the popup
}

void CefHandler::OnAfterCreated(CefRefPtr<CefBrowser> browser) {
    int64_t browser_id = browser->GetIdentifier();
    // TODO: Register with the Rust tab manager via FFI
    // feathersurf_eviction_add_tab(manager, tab_id, now);
}

bool CefHandler::DoClose(CefRefPtr<CefBrowser> browser) {
    // Allow the close
    return false;
}

void CefHandler::OnBeforeClose(CefRefPtr<CefBrowser> browser) {
    int64_t browser_id = browser->GetIdentifier();
    // TODO: Unregister from the Rust tab manager
    browser_to_tab_.erase(browser_id);
    tab_to_browser_.erase(browser_id);
}

void CefHandler::OnLoadStart(CefRefPtr<CefBrowser> browser,
                              CefRefPtr<CefFrame> frame,
                              TransitionType transition_type) {
    if (!frame->IsMain()) return;

    // TODO: Send navigation start event to Rust tab manager
    // Update the tab state to indicate loading
}

void CefHandler::OnLoadEnd(CefRefPtr<CefBrowser> browser,
                            CefRefPtr<CefFrame> frame,
                            int httpStatusCode) {
    if (!frame->IsMain()) return;

    // TODO: Send navigation finish event to Rust tab manager
    // Update the snapshot with the page title and URL
    CefString title = frame->GetBrowser()->GetMainFrame()->GetURL();

    // Update status bar
    // SendMessageW(status_bar_, SB_SETTEXT, 0, L"Done");
}

void CefHandler::OnLoadError(CefRefPtr<CefBrowser> browser,
                              CefRefPtr<CefFrame> frame,
                              ErrorCode errorCode,
                              const CefString& errorText,
                              const CefString& failedUrl) {
    if (!frame->IsMain()) return;

    // TODO: Send navigation error event to Rust tab manager
    // Display error in the browser view (like Chrome's error pages)
    if (errorCode == ERR_ABORTED) return;  // User navigated away

    // Show error page
    std::wstring error_html = L"<html><body style='font-family:sans-serif;"
        L"text-align:center;padding:50px;'>"
        L"<h2>Page failed to load</h2>"
        L"<p>" + errorText.ToString() + L"</p>"
        L"<p>" + failedUrl.ToString() + L"</p>"
        L"</body></html>";

    frame->LoadURL(CefString(L"data:text/html," + error_html));
}

bool CefHandler::OnBeforeBrowse(CefRefPtr<CefBrowser> browser,
                                 CefRefPtr<CefFrame> frame,
                                 CefRefPtr<CefRequest> request,
                                 bool user_gesture,
                                 bool is_redirect) {
    if (!frame->IsMain()) return false;

    // TODO: Notify Rust tab manager of navigation
    // Check privacy rules (tracker blocking, etc.)
    // Return true to block the request, false to allow

    return false;  // Allow navigation
}

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
