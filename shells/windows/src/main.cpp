// FeatherSurf Windows Browser Shell
//
// Entry point and Win32 window creation. This shell connects to the Rust
// FFI bridge (feathersurf-ffi) and CEF for web rendering.

#ifndef UNICODE
#define UNICODE
#endif

#include <windows.h>
#include <commctrl.h>
#include <shellapi.h>

#include "browser_window.h"
#include "cef_handler.h"

#pragma comment(lib, "comctl32.lib")
#pragma comment(lib, "shlwapi.lib")

int WINAPI wWinMain(HINSTANCE hInstance, HINSTANCE hPrevInstance,
                    LPWSTR lpCmdLine, int nCmdShow) {
    // Initialize common controls (for tab strip, buttons, etc.)
    INITCOMMONCONTROLSEX icex = {};
    icex.dwSize = sizeof(INITCOMMONCONTROLSEX);
    icex.dwICC = ICC_TAB_CLASSES | ICC_BAR_CLASSES | ICC_WIN95_CLASSES;
    InitCommonControlsEx(&icex);

    // Initialize CEF
    CefSettings settings;
    settings.no_sandbox = true;  // Sandbox configured separately in production
    settings.multi_threaded_message_loop = false;

    CefMainArgs main_args(hInstance);
    if (!CefInitialize(main_args, settings, nullptr, nullptr)) {
        MessageBoxW(nullptr, L"Failed to initialize CEF", L"FeatherSurf",
                    MB_OK | MB_ICONERROR);
        return 1;
    }

    // Create the main browser window
    BrowserWindow browser;
    if (!browser.Create(hInstance, L"FeatherSurf", nCmdShow)) {
        MessageBoxW(nullptr, L"Failed to create browser window", L"FeatherSurf",
                    MB_OK | MB_ICONERROR);
        CefShutdown();
        return 1;
    }

    // Message loop
    MSG msg;
    while (GetMessageW(&msg, nullptr, 0, 0)) {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);

        // Process CEF messages on the browser UI thread
        CefDoMessageLoopWork();
    }

    CefShutdown();
    return static_cast<int>(msg.wParam);
}
