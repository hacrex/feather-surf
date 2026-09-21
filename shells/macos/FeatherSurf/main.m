#import <Cocoa/Cocoa.h>
#include <include/cef_app.h>

/// CEF subprocess entry point.
///
/// On macOS, the CEF helper subprocess is a separate executable embedded in the
/// app bundle. When launched with `--type=...`, CEF runs the subprocess instead
/// of the main app. This function handles both cases:
///   - Subprocess: passes through to CefExecuteProcess and exits.
///   - Main app: starts the Cocoa run loop via NSApplicationMain.
int main(int argc, const char* argv[]) {
    @autoreleasepool {
        CefMainArgs main_args(argc, const_cast<char**>(argv));
        int exit_code = CefExecuteProcess(main_args, nullptr, nullptr);
        if (exit_code >= 0) {
            return exit_code;
        }
        return NSApplicationMain(argc, argv);
    }
}
