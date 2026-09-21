// FeatherSurf Browser Window
//
// Manages the main browser window, address bar, tab strip, and navigation
// controls. Connects to CEF for web rendering.

#pragma once

#ifndef UNICODE
#define UNICODE
#endif

#include <windows.h>
#include <string>
#include <vector>

// Window control IDs
constexpr int ID_ADDRESS_BAR     = 1001;
constexpr int ID_BTN_BACK        = 1002;
constexpr int ID_BTN_FORWARD     = 1003;
constexpr int ID_BTN_RELOAD      = 1004;
constexpr int ID_BTN_HOME        = 1005;
constexpr int ID_TAB_STRIP       = 1006;
constexpr int ID_STATUS_BAR      = 1007;

class BrowserWindow {
public:
    BrowserWindow() = default;
    ~BrowserWindow() = default;

    // Non-copyable
    BrowserWindow(const BrowserWindow&) = delete;
    BrowserWindow& operator=(const BrowserWindow&) = delete;

    // Create and show the browser window.
    bool Create(HINSTANCE hInstance, const wchar_t* title, int nCmdShow);

    // Get the HWND of the main window.
    HWND GetHWND() const { return hwnd_; }

private:
    // Window procedure
    static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp);

    // Message handlers
    LRESULT OnCreate(HWND hwnd);
    LRESULT OnSize(WPARAM wp, LPARAM lp);
    LRESULT OnCommand(WPARAM wp, LPARAM lp);
    LRESULT OnNotify(WPARAM wp, LPARAM lp);
    LRESULT OnKeyDown(WPARAM wp, LPARAM lp);

    // UI creation helpers
    void CreateAddressBar(HWND parent);
    void CreateNavigationButtons(HWND parent);
    void CreateTabStrip(HWND parent);
    void CreateStatusBar(HWND parent);

    // Navigation
    void NavigateTo(const std::wstring& url);
    void GoBack();
    void GoForward();
    void Reload();
    void GoHome();

    // Tab management
    void AddTab(const std::wstring& title, const std::wstring& url);
    void CloseTab(int index);
    void SelectTab(int index);

    HWND hwnd_ = nullptr;
    HWND address_bar_ = nullptr;
    HWND btn_back_ = nullptr;
    HWND btn_forward_ = nullptr;
    HWND btn_reload_ = nullptr;
    HWND btn_home_ = nullptr;
    HWND tab_strip_ = nullptr;
    HWND status_bar_ = nullptr;
    HWND browser_view_ = nullptr;  // CEF browser host window

    std::wstring home_url_ = L"about:blank";
    int current_tab_ = 0;

    struct Tab {
        std::wstring title;
        std::wstring url;
    };
    std::vector<Tab> tabs_;
};
