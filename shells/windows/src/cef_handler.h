// FeatherSurf CEF Handler
//
// Handles CEF browser process callbacks and connects to the Rust FFI bridge
// for tab lifecycle management.

#pragma once

#include <include/cef_client.h>
#include <include/cef_browser.h>

#include <string>
#include <map>

class CefHandler : public CefClient,
                   public CefLifeSpanHandler,
                   public CefLoadHandler,
                   public CefRequestHandler {
public:
    CefHandler() = default;
    ~CefHandler() override = default;

    // CefClient
    CefRefPtr<CefLifeSpanHandler> GetLifeSpanHandler() override { return this; }
    CefRefPtr<CefLoadHandler> GetLoadHandler() override { return this; }
    CefRefPtr<CefRequestHandler> GetRequestHandler() override { return this; }

    // CefLifeSpanHandler
    bool OnBeforePopup(CefRefPtr<CefBrowser> browser,
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
                       bool* no_javascript_access) override;

    void OnAfterCreated(CefRefPtr<CefBrowser> browser) override;
    bool DoClose(CefRefPtr<CefBrowser> browser) override;
    void OnBeforeClose(CefRefPtr<CefBrowser> browser) override;

    // CefLoadHandler
    void OnLoadStart(CefRefPtr<CefBrowser> browser,
                     CefRefPtr<CefFrame> frame,
                     TransitionType transition_type) override;
    void OnLoadEnd(CefRefPtr<CefBrowser> browser,
                   CefRefPtr<CefFrame> frame,
                   int httpStatusCode) override;
    void OnLoadError(CefRefPtr<CefBrowser> browser,
                     CefRefPtr<CefFrame> frame,
                     ErrorCode errorCode,
                     const CefString& errorText,
                     const CefString& failedUrl) override;

    // CefRequestHandler
    bool OnBeforeBrowse(CefRefPtr<CefBrowser> browser,
                        CefRefPtr<CefFrame> frame,
                        CefRefPtr<CefRequest> request,
                        bool user_gesture,
                        bool is_redirect) override;

    // Tab management via FFI
    CefRefPtr<CefBrowser> GetBrowserForTab(int64_t tab_id) const;
    int64_t GetTabIdForBrowser(CefRefPtr<CefBrowser> browser) const;

    // Freeze/Suspend/Restore commands
    bool FreezeTab(int64_t tab_id);
    bool SuspendTab(int64_t tab_id);
    bool RestoreTab(int64_t tab_id, const std::string& url);

private:
    // Map of browser IDs to tab IDs
    std::map<int64_t, int64_t> browser_to_tab_;
    std::map<int64_t, CefRefPtr<CefBrowser>> tab_to_browser_;

    IMPLEMENT_REFCOUNTING(CefHandler);
    DISALLOW_COPY_AND_ASSIGN(CefHandler);
};
