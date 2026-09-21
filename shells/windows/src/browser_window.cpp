// FeatherSurf Browser Window Implementation

#include "browser_window.h"

#include <commctrl.h>
#include <shlwapi.h>
#include <shellscalingapi.h>

#pragma comment(lib, "shcore.lib")

namespace {

constexpr int WINDOW_MIN_WIDTH = 800;
constexpr int WINDOW_MIN_HEIGHT = 600;
constexpr int TOOLBAR_HEIGHT = 40;
constexpr int TAB_HEIGHT = 32;
constexpr int STATUS_HEIGHT = 22;
constexpr int NAV_BUTTON_WIDTH = 30;

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

void BrowserWindow::NavigateTo(const std::wstring& url) {
    // TODO: Connect to CEF browser
    // For now, update the address bar and status bar
    SetWindowTextW(address_bar_, url.c_str());
    SendMessageW(status_bar_, SB_SETTEXT, 0,
                 reinterpret_cast<LPARAM>(L"Navigating..."));

    if (current_tab_ >= 0 && current_tab_ < static_cast<int>(tabs_.size())) {
        tabs_[current_tab_].url = url;
        // Update tab title
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
    }
}

void BrowserWindow::GoBack() {
    // TODO: CEF GoBack
}

void BrowserWindow::GoForward() {
    // TODO: CEF GoForward
}

void BrowserWindow::Reload() {
    // TODO: CEF Reload
    if (current_tab_ >= 0 && current_tab_ < static_cast<int>(tabs_.size())) {
        NavigateTo(tabs_[current_tab_].url);
    }
}

void BrowserWindow::GoHome() {
    NavigateTo(home_url_);
}

void BrowserWindow::AddTab(const std::wstring& title,
                            const std::wstring& url) {
    tabs_.push_back({title, url});

    TCITEMW tie = {};
    tie.mask = TCIF_TEXT;
    tie.pszText = const_cast<wchar_t*>(title.c_str());
    TabCtrl_InsertItem(tab_strip_,
                       static_cast<int>(tabs_.size()) - 1, &tie);

    SelectTab(static_cast<int>(tabs_.size()) - 1);
}

void BrowserWindow::CloseTab(int index) {
    if (index < 0 || index >= static_cast<int>(tabs_.size())) return;
    if (tabs_.size() == 1) {
        // Last tab - close window
        DestroyWindow(hwnd_);
        return;
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
    NavigateTo(tabs_[index].url);
}
