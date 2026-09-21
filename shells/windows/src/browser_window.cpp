// FeatherSurf Browser Window Implementation
//
// Manages the main browser window, address bar, tab strip, and navigation
// controls. Connects to CEF for web rendering and the Rust FFI bridge
// for tab lifecycle management.

#include "browser_window.h"
#include "cef_handler.h"
#include "feathersurf_ffi.h"

#include <commctrl.h>
#include <shlwapi.h>
#include <shellscalingapi.h>

#include <include/cef_app.h>
#include <include/cef_browser.h>

#pragma comment(lib, "shcore.lib")

namespace {

constexpr int WINDOW_MIN_WIDTH = 800;
constexpr int WINDOW_MIN_HEIGHT = 600;
constexpr int TOOLBAR_HEIGHT = 40;
constexpr int TAB_HEIGHT = 32;
constexpr int STATUS_HEIGHT = 22;
constexpr int NAV_BUTTON_WIDTH = 30;

// Global CEF handler (shared across tabs)
CefRefPtr<CefHandler> g_cef_handler;

}  // namespace

LRESULT CALLBACK BrowserWindow::WndProc(HWND hwnd, UINT msg, WPARAM wp,
                                         LPARAM lp) {
    BrowserWindow* self = nullptr;

    if (msg == WM_NCCREATE) {
        auto cs = reinterpret_cast<CREATESTRUCT*>(lp);
        self = static_cast<BrowserWindow*>(cs->lpCreateParams);
        SetWindowLongPtr(hwnd, GWLP_USERDATA, reinterpret_cast<LONG_PTR>(self));
        self->hwnd_ = hwnd;
    } else {
        self = reinterpret_cast<BrowserWindow*>(GetWindowLongPtr(hwnd, GWLP_USERDATA));
    }

    if (self) {
        switch (msg) {
            case WM_CREATE:
                return self->OnCreate(hwnd);
            case WM_SIZE:
                return self->OnSize(wp, lp);
            case WM_COMMAND:
                return self->OnCommand(wp, lp);
            case WM_NOTIFY:
                return self->OnNotify(wp, lp);
            case WM_KEYDOWN:
                return self->OnKeyDown(wp, lp);
            case WM_DESTROY:
                PostQuitMessage(0);
                return 0;
            case WM_GETMINMAXINFO: {
                auto mmi = reinterpret_cast<MINMAXINFO*>(lp);
                mmi->ptMinTrackSize.x = WINDOW_MIN_WIDTH;
                mmi->ptMinTrackSize.y = WINDOW_MIN_HEIGHT;
                return 0;
            }
        }
    }
    return DefWindowProcW(hwnd, msg, wp, lp);
}

bool BrowserWindow::Create(HINSTANCE hInstance, const wchar_t* title,
                            int nCmdShow) {
    WNDCLASSEXW wc = {};
    wc.cbSize = sizeof(WNDCLASSEXW);
    wc.style = CS_HREDRAW | CS_VREDRAW;
    wc.lpfnWndProc = WndProc;
    wc.hInstance = hInstance;
    wc.hCursor = LoadCursorW(nullptr, IDC_ARROW);
    wc.hbrBackground = reinterpret_cast<HBRUSH>(GetSysColorBrush(COLOR_WINDOW));
    wc.lpszClassName = L"FeatherSurfMain";

    if (!RegisterClassExW(&wc)) {
        return false;
    }

    // Center the window on screen
    int screen_w = GetSystemMetrics(SM_CXSCREEN);
    int screen_h = GetSystemMetrics(SM_CYSCREEN);
    int window_w = screen_w * 3 / 4;
    int window_h = screen_h * 3 / 4;
    int window_x = (screen_w - window_w) / 2;
    int window_y = (screen_h - window_h) / 2;

    hwnd_ = CreateWindowExW(
        0, wc.lpszClassName, title,
        WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN,
        window_x, window_y, window_w, window_h,
        nullptr, nullptr, hInstance, this);

    if (!hwnd_) {
        return false;
    }

    ShowWindow(hwnd_, nCmdShow);
    UpdateWindow(hwnd_);
    return true;
}

LRESULT BrowserWindow::OnCreate(HWND hwnd) {
    CreateNavigationButtons(hwnd);
    CreateAddressBar(hwnd);
    CreateTabStrip(hwnd);
    CreateStatusBar(hwnd);

    // Create the browser view container
    browser_view_ = CreateWindowExW(
        0, L"STATIC", L"",
        WS_CHILD | WS_VISIBLE | SS_BLACKRECT,
        0, 0, 0, 0,
        hwnd, nullptr,
        GetModuleHandleW(nullptr), nullptr);

    // Initialize CEF handler
    g_cef_handler = new CefHandler();

    // Add a default tab
    AddTab(L"New Tab", L"about:blank");

    return 0;
}

LRESULT BrowserWindow::OnSize(WPARAM wp, LPARAM lp) {
    if (wp == SIZE_MINIMIZED) return 0;

    int width = LOWORD(lp);
    int height = HIWORD(lp);

    int y = 0;

    // Navigation buttons and address bar (top row)
    int nav_y = y;
    int nav_x = 4;

    if (btn_back_) {
        MoveWindow(btn_back_, nav_x, nav_y + 4, NAV_BUTTON_WIDTH,
                   TOOLBAR_HEIGHT - 8, TRUE);
        nav_x += NAV_BUTTON_WIDTH + 2;
    }
    if (btn_forward_) {
        MoveWindow(btn_forward_, nav_x, nav_y + 4, NAV_BUTTON_WIDTH,
                   TOOLBAR_HEIGHT - 8, TRUE);
        nav_x += NAV_BUTTON_WIDTH + 2;
    }
    if (btn_reload_) {
        MoveWindow(btn_reload_, nav_x, nav_y + 4, NAV_BUTTON_WIDTH,
                   TOOLBAR_HEIGHT - 8, TRUE);
        nav_x += NAV_BUTTON_WIDTH + 2;
    }
    if (btn_home_) {
        MoveWindow(btn_home_, nav_x, nav_y + 4, NAV_BUTTON_WIDTH,
                   TOOLBAR_HEIGHT - 8, TRUE);
        nav_x += NAV_BUTTON_WIDTH + 8;
    }
    if (address_bar_) {
        int bar_width = width - nav_x - 8;
        MoveWindow(address_bar_, nav_x, nav_y + 2, bar_width,
                   TOOLBAR_HEIGHT - 4, TRUE);
    }
    y += TOOLBAR_HEIGHT;

    // Tab strip
    if (tab_strip_) {
        MoveWindow(tab_strip_, 0, y, width, TAB_HEIGHT, TRUE);
        y += TAB_HEIGHT;
    }

    // Status bar
    int browser_height = height - y - STATUS_HEIGHT;
    if (status_bar_) {
        MoveWindow(status_bar_, 0, height - STATUS_HEIGHT, width,
                   STATUS_HEIGHT, TRUE);
        browser_height = height - y - STATUS_HEIGHT;
    }

    // Browser view (CEF)
    if (browser_view_) {
        MoveWindow(browser_view_, 0, y, width, browser_height, TRUE);
    }

    // Resize all CEF browsers
    ResizeCefBrowsers();

    return 0;
}

LRESULT BrowserWindow::OnCommand(WPARAM wp, LPARAM lp) {
    int id = LOWORD(wp);
    switch (id) {
        case ID_BTN_BACK:
            GoBack();
            return 0;
        case ID_BTN_FORWARD:
            GoForward();
            return 0;
        case ID_BTN_RELOAD:
            Reload();
            return 0;
        case ID_BTN_HOME:
            GoHome();
            return 0;
        case ID_ADDRESS_BAR: {
            if (HIWORD(wp) == EN_CHANGE) {
                // Address bar text changed - could update UI
            }
            return 0;
        }
    }
    return DefWindowProcW(hwnd_, WM_COMMAND, wp, lp);
}

LRESULT BrowserWindow::OnNotify(WPARAM wp, LPARAM lp) {
    auto nmh = reinterpret_cast<NMHDR*>(lp);
    if (nmh->idFrom == ID_TAB_STRIP && nmh->code == TCN_SELCHANGE) {
        int sel = TabCtrl_GetCurSel(tab_strip_);
        if (sel >= 0) {
            SelectTab(sel);
        }
    }
    return DefWindowProcW(hwnd_, WM_NOTIFY, wp, lp);
}

LRESULT BrowserWindow::OnKeyDown(WPARAM wp, LPARAM lp) {
    bool ctrl = (GetKeyState(VK_CONTROL) & 0x8000) != 0;
    if (ctrl) {
        switch (wp) {
            case 'T':
                AddTab(L"New Tab", L"about:blank");
                return 0;
            case 'W':
                CloseTab(current_tab_);
                return 0;
            case 'L':
                SetFocus(address_bar_);
                SendMessageW(address_bar_, EM_SETSEL, 0, -1);
                return 0;
            case 'R':
                Reload();
                return 0;
            case VK_TAB:
                // Cycle tabs
                if (tabs_.size() > 1) {
                    int next = current_tab_ + ((GetKeyState(VK_SHIFT) & 0x8000) ? -1 : 1);
                    if (next < 0) next = static_cast<int>(tabs_.size()) - 1;
                    if (next >= static_cast<int>(tabs_.size())) next = 0;
                    SelectTab(next);
                }
                return 0;
        }
    }
    return DefWindowProcW(hwnd_, WM_KEYDOWN, wp, lp);
}

// ── UI Creation ────────────────────────────────────────────────────

void BrowserWindow::CreateAddressBar(HWND parent) {
    address_bar_ = CreateWindowExW(
        0, L"EDIT", L"",
        WS_CHILD | WS_VISIBLE | WS_BORDER | ES_AUTOHSCROLL,
        0, 0, 400, 24,
        parent, reinterpret_cast<HMENU>(ID_ADDRESS_BAR),
        GetModuleHandleW(nullptr), nullptr);

    if (address_bar_) {
        HFONT hfont = CreateFontW(
            16, 0, 0, 0, FW_NORMAL, FALSE, FALSE, FALSE,
            DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS,
            CLEARTYPE_QUALITY, DEFAULT_PITCH | FF_SWISS, L"Segoe UI");
        SendMessageW(address_bar_, WM_SETFONT, reinterpret_cast<WPARAM>(hfont),
                     TRUE);
    }
}

void BrowserWindow::CreateNavigationButtons(HWND parent) {
    auto create_button = [&](const wchar_t* text, int id, int x) {
        HWND btn = CreateWindowExW(
            0, L"BUTTON", text,
            WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON | BS_FLAT,
            x, 4, NAV_BUTTON_WIDTH, TOOLBAR_HEIGHT - 8,
            parent, reinterpret_cast<HMENU>(id),
            GetModuleHandleW(nullptr), nullptr);

        HFONT hfont = CreateFontW(
            14, 0, 0, 0, FW_NORMAL, FALSE, FALSE, FALSE,
            DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS,
            CLEARTYPE_QUALITY, DEFAULT_PITCH | FF_SWISS, L"Segoe UI");
        SendMessageW(btn, WM_SETFONT, reinterpret_cast<WPARAM>(hfont), TRUE);
        return btn;
    };

    btn_back_ = create_button(L"\u25C0", ID_BTN_BACK, 4);
    btn_forward_ = create_button(L"\u25B6", ID_BTN_FORWARD, 36);
    btn_reload_ = create_button(L"\u21BB", ID_BTN_RELOAD, 68);
    btn_home_ = create_button(L"\u2302", ID_BTN_HOME, 100);
}

void BrowserWindow::CreateTabStrip(HWND parent) {
    tab_strip_ = CreateWindowExW(
        0, WC_TABCONTROLW, L"",
        WS_CHILD | WS_VISIBLE | TCS_FIXEDWIDTH,
        0, TOOLBAR_HEIGHT, 800, TAB_HEIGHT,
        parent, reinterpret_cast<HMENU>(ID_TAB_STRIP),
        GetModuleHandleW(nullptr), nullptr);

    if (tab_strip_) {
        HFONT hfont = CreateFontW(
            13, 0, 0, 0, FW_NORMAL, FALSE, FALSE, FALSE,
            DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS,
            CLEARTYPE_QUALITY, DEFAULT_PITCH | FF_SWISS, L"Segoe UI");
        SendMessageW(tab_strip_, WM_SETFONT, reinterpret_cast<WPARAM>(hfont),
                     TRUE);
    }
}

void BrowserWindow::CreateStatusBar(HWND parent) {
    status_bar_ = CreateWindowExW(
        0, STATUSCLASSNAMEW, L"",
        WS_CHILD | WS_VISIBLE | SBARS_SIZEGRIP,
        0, 0, 0, 0,
        parent, reinterpret_cast<HMENU>(ID_STATUS_BAR),
        GetModuleHandleW(nullptr), nullptr);

    if (status_bar_) {
        int parts[] = {300, 500, -1};
        SendMessageW(status_bar_, SB_SETPARTS, 3,
                     reinterpret_cast<LPARAM>(parts));
        SendMessageW(status_bar_, SB_SETTEXT, 0,
                     reinterpret_cast<LPARAM>(L"Ready"));
    }
}

// ── Navigation ─────────────────────────────────────────────────────

void BrowserWindow::NavigateTo(const std::wstring& url) {
    SetWindowTextW(address_bar_, url.c_str());
    SendMessageW(status_bar_, SB_SETTEXT, 0,
                 reinterpret_cast<LPARAM>(L"Navigating..."));

    if (current_tab_ >= 0 && current_tab_ < static_cast<int>(tabs_.size())) {
        tabs_[current_tab_].url = url;

        // Update tab title in tab strip
        wchar_t display[128];
        if (url.length() > 30) {
            wcscpy_s(display, L"");
            wcsncat_s(display, url.c_str(), 27);
            wcscat_s(display, L"...");
        } else {
            wcscpy_s(display, url.c_str());
        }
        TCITEMW tie = {};
        tie.mask = TCIF_TEXT;
        tie.pszText = display;
        TabCtrl_SetItem(tab_strip_, current_tab_, &tie);

        // Navigate CEF browser
        auto browser = g_cef_handler->GetBrowserForTab(current_tab_);
        if (browser) {
            browser->GetMainFrame()->LoadURL(CefString(url));
        }
    }
}

void BrowserWindow::GoBack() {
    auto browser = g_cef_handler->GetBrowserForTab(current_tab_);
    if (browser) {
        browser->GoBack();
    }
}

void BrowserWindow::GoForward() {
    auto browser = g_cef_handler->GetBrowserForTab(current_tab_);
    if (browser) {
        browser->GoForward();
    }
}

void BrowserWindow::Reload() {
    auto browser = g_cef_handler->GetBrowserForTab(current_tab_);
    if (browser) {
        browser->Reload();
    }
}

void BrowserWindow::GoHome() {
    NavigateTo(home_url_);
}

// ── Tab Management ─────────────────────────────────────────────────

void BrowserWindow::AddTab(const std::wstring& title, const std::wstring& url) {
    tabs_.push_back({title, url});

    TCITEMW tie = {};
    tie.mask = TCIF_TEXT;
    tie.pszText = const_cast<wchar_t*>(title.c_str());
    TabCtrl_InsertItem(tab_strip_,
                       static_cast<int>(tabs_.size()) - 1, &tie);

    // Create CEF browser for the new tab
    CreateCefBrowserForTab(static_cast<int>(tabs_.size()) - 1, url);

    SelectTab(static_cast<int>(tabs_.size()) - 1);
}

void BrowserWindow::CloseTab(int index) {
    if (index < 0 || index >= static_cast<int>(tabs_.size())) return;
    if (tabs_.size() == 1) {
        // Last tab - close window
        DestroyWindow(hwnd_);
        return;
    }

    // Close CEF browser for this tab
    auto browser = g_cef_handler->GetBrowserForTab(index);
    if (browser) {
        browser->GetHost()->CloseBrowser(true);
    }

    tabs_.erase(tabs_.begin() + index);
    TabCtrl_DeleteItem(tab_strip_, index);

    if (current_tab_ >= static_cast<int>(tabs_.size())) {
        current_tab_ = static_cast<int>(tabs_.size()) - 1;
    }
    SelectTab(current_tab_);
}

void BrowserWindow::SelectTab(int index) {
    if (index < 0 || index >= static_cast<int>(tabs_.size())) return;
    current_tab_ = index;
    TabCtrl_SetCurSel(tab_strip_, index);

    // Show/hide CEF browsers
    ShowCefBrowserForTab(index);

    // Update address bar
    SetWindowTextW(address_bar_, tabs_[index].url.c_str());
}

// ── CEF Browser Management ─────────────────────────────────────────

void BrowserWindow::CreateCefBrowserForTab(int index, const std::wstring& url) {
    if (!g_cef_handler || !browser_view_) return;

    CefWindowInfo windowInfo;
    CefBrowserSettings settings;

    // Set the parent window to our browser view container
    windowInfo.SetAsChild(browser_view_, CefRect(0, 0, 0, 0));

    // Create the browser
    CefRefPtr<CefBrowser> browser;
    bool result = CefBrowserHost::CreateBrowser(
        windowInfo,
        g_cef_handler,
        CefString(url),
        settings,
        nullptr,
        nullptr);

    if (result) {
        SendMessageW(status_bar_, SB_SETTEXT, 0,
                     reinterpret_cast<LPARAM>(L"Browser created"));
    }
}

void BrowserWindow::ShowCefBrowserForTab(int index) {
    // Hide all CEF browsers, show only the selected one
    // This is handled by the CEF window parenting
}

void BrowserWindow::ResizeCefBrowsers() {
    if (!browser_view_) return;

    // Get the size of the browser view container
    RECT rect;
    GetClientRect(browser_view_, &rect);
    int width = rect.right - rect.left;
    int height = rect.bottom - rect.top;

    // Resize all CEF browsers to fill the container
    // Only the active browser should be visible
    for (size_t i = 0; i < tabs_.size(); ++i) {
        auto browser = g_cef_handler->GetBrowserForTab(static_cast<int64_t>(i));
        if (browser) {
            HWND hwnd = browser->GetHost()->GetWindowHandle();
            if (hwnd) {
                // Position all browsers at (0,0) but only show the active one
                MoveWindow(hwnd, 0, 0, width, height, TRUE);
                ShowWindow(hwnd, (i == static_cast<size_t>(current_tab_))
                    ? SW_SHOW : SW_HIDE);
            }
        }
    }
}
