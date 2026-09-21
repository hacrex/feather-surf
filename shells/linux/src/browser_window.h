// FeatherSurf Browser Window (GTK4)
//
// Manages the main browser window, address bar, tab strip, and navigation
// controls for the Linux shell.

#pragma once

#include <gtk/gtk.h>

// Tab structure
typedef struct {
    char *title;
    char *url;
    GtkWidget *page;  // Web view widget (CEF or WebKit)
} BrowserTab;

// Browser window structure
typedef struct {
    GtkWidget *window;
    GtkWidget *header_bar;
    GtkWidget *nav_back;
    GtkWidget *nav_forward;
    GtkWidget *nav_reload;
    GtkWidget *nav_home;
    GtkWidget *address_bar;
    GtkWidget *tab_strip;
    GtkWidget *tab_bar;
    GtkWidget *browser_view;
    GtkWidget *status_bar;

    BrowserTab *tabs;
    int tab_count;
    int tab_capacity;
    int current_tab;

    char *home_url;
} BrowserWindow;

// Create a new browser window
BrowserWindow *browser_window_new(void);

// Show the browser window
void browser_window_show(BrowserWindow *browser);

// Free the browser window and all resources
void browser_window_free(BrowserWindow *browser);

// Navigation
void browser_navigate(BrowserWindow *browser, const char *url);
void browser_go_back(BrowserWindow *browser);
void browser_go_forward(BrowserWindow *browser);
void browser_reload(BrowserWindow *browser);
void browser_go_home(BrowserWindow *browser);

// Tab management
void browser_add_tab(BrowserWindow *browser, const char *title, const char *url);
void browser_close_tab(BrowserWindow *browser, int index);
void browser_select_tab(BrowserWindow *browser, int index);
