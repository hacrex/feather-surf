// FeatherSurf Linux Browser Shell
//
// GTK4-based entry point for the FeatherSurf browser on Linux.
// Connects to the Rust FFI bridge (feathersurf-ffi) and optionally CEF.

#include <gtk/gtk.h>
#include "browser_window.h"

int main(int argc, char *argv[]) {
    // Initialize GTK
    gtk_init();

    // Create the main browser window
    BrowserWindow *browser = browser_window_new();
    browser_window_show(browser);

    // Run the main loop
    gtk_main();

    // Cleanup
    browser_window_free(browser);
    return 0;
}
