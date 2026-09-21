// FeatherSurf Browser Window Implementation (GTK4)

#include "browser_window.h"
#include <string.h>
#include <stdlib.h>

#define INITIAL_TAB_CAPACITY 8
#define TAB_HEIGHT 36
#define TOOLBAR_HEIGHT 40
#define STATUS_HEIGHT 24

// ── Callbacks ──────────────────────────────────────────────────────

static void on_back_clicked(GtkButton *button, gpointer user_data) {
    (void)button;
    BrowserWindow *browser = user_data;
    browser_go_back(browser);
}

static void on_forward_clicked(GtkButton *button, gpointer user_data) {
    (void)button;
    BrowserWindow *browser = user_data;
    browser_go_forward(browser);
}

static void on_reload_clicked(GtkButton *button, gpointer user_data) {
    (void)button;
    BrowserWindow *browser = user_data;
    browser_reload(browser);
}

static void on_home_clicked(GtkButton *button, gpointer user_data) {
    (void)button;
    BrowserWindow *browser = user_data;
    browser_go_home(browser);
}

static void on_address_bar_activate(GtkEntry *entry, gpointer user_data) {
    BrowserWindow *browser = user_data;
    const char *url = gtk_editable_get_text(GTK_EDITABLE(entry));
    if (url && url[0] != '\0') {
        browser_navigate(browser, url);
    }
}

static void on_tab_close_clicked(GtkButton *button, gpointer user_data) {
    (void)button;
    // TODO: Find which tab this button belongs to and close it
    BrowserWindow *browser = user_data;
    if (browser->tab_count > 1) {
        browser_close_tab(browser, browser->current_tab);
    }
}

static void on_new_tab_clicked(GtkButton *button, gpointer user_data) {
    (void)button;
    BrowserWindow *browser = user_data;
    browser_add_tab(browser, "New Tab", "about:blank");
}

static gboolean on_window_close(GtkWindow *window, gpointer user_data) {
    (void)window;
    BrowserWindow *browser = user_data;
    // TODO: Save session before closing
    (void)browser;
    return FALSE;  // Allow close
}

static gboolean on_key_press(GtkEventControllerKey *controller,
                              guint keyval, guint keycode, GdkModifierType state,
                              gpointer user_data) {
    (void)controller;
    (void)keycode;
    BrowserWindow *browser = user_data;

    gboolean ctrl = (state & GDK_CONTROL_MASK) != 0;

    if (ctrl) {
        switch (keyval) {
            case GDK_KEY_t:
                browser_add_tab(browser, "New Tab", "about:blank");
                return TRUE;
            case GDK_KEY_w:
                if (browser->tab_count > 1) {
                    browser_close_tab(browser, browser->current_tab);
                }
                return TRUE;
            case GDK_KEY_l:
                gtk_widget_grab_focus(browser->address_bar);
                gtk_editable_select_region(GTK_EDITABLE(browser->address_bar),
                                           0, -1);
                return TRUE;
            case GDK_KEY_r:
                browser_reload(browser);
                return TRUE;
            case GDK_KEY_Tab:
                if (browser->tab_count > 1) {
                    int next = browser->current_tab + 1;
                    if (next >= browser->tab_count) next = 0;
                    browser_select_tab(browser, next);
                }
                return TRUE;
        }
    }

    return FALSE;
}

// ── Tab strip helpers ──────────────────────────────────────────────

static void rebuild_tab_strip(BrowserWindow *browser) {
    // Remove all children from tab bar
    GtkWidget *child;
    while ((child = gtk_widget_get_first_child(browser->tab_bar)) != NULL) {
        gtk_box_remove(GTK_BOX(browser->tab_bar), child);
    }

    // Add tab buttons
    for (int i = 0; i < browser->tab_count; i++) {
        GtkWidget *box = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 4);
        gtk_widget_set_size_request(box, 120, TAB_HEIGHT);

        // Tab label
        GtkWidget *label = gtk_label_new(browser->tabs[i].title);
        gtk_label_set_ellipsize(GTK_LABEL(label), PANGO_ELLIPSIZE_END);
        gtk_label_set_max_width_chars(GTK_LABEL(label), 12);
        gtk_widget_set_hexpand(label, TRUE);
        gtk_box_append(GTK_BOX(box), label);

        // Close button
        if (browser->tab_count > 1) {
            GtkWidget *close_btn = gtk_button_new_from_icon_name("window-close-symbolic");
            gtk_widget_set_size_request(close_btn, 20, 20);
            g_signal_connect(close_btn, "clicked", G_CALLBACK(on_tab_close_clicked), browser);
            gtk_box_append(GTK_BOX(box), close_btn);
        }

        // Style active tab differently
        if (i == browser->current_tab) {
            gtk_widget_add_css_class(box, "active-tab");
        } else {
            gtk_widget_add_css_class(box, "tab");
        }

        gtk_box_append(GTK_BOX(browser->tab_bar), box);
    }
}

// ── Public API ─────────────────────────────────────────────────────

BrowserWindow *browser_window_new(void) {
    BrowserWindow *browser = calloc(1, sizeof(BrowserWindow));
    if (!browser) return NULL;

    browser->tab_capacity = INITIAL_TAB_CAPACITY;
    browser->tabs = calloc(browser->tab_capacity, sizeof(BrowserTab));
    if (!browser->tabs) {
        free(browser);
        return NULL;
    }

    browser->home_url = strdup("about:blank");
    browser->current_tab = 0;
    browser->tab_count = 0;

    // ── Main window ──
    browser->window = gtk_window_new();
    gtk_window_set_title(GTK_WINDOW(browser->window), "FeatherSurf");
    gtk_window_set_default_size(GTK_WINDOW(browser->window), 1280, 800);
    g_signal_connect(browser->window, "close-request",
                     G_CALLBACK(on_window_close), browser);

    // ── Main vertical box ──
    GtkWidget *main_box = gtk_box_new(GTK_ORIENTATION_VERTICAL, 0);
    gtk_window_set_child(GTK_WINDOW(browser->window), main_box);

    // ── Toolbar (horizontal box) ──
    GtkWidget *toolbar = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 4);
    gtk_widget_set_size_request(toolbar, -1, TOOLBAR_HEIGHT);
    gtk_widget_set_margin_start(toolbar, 4);
    gtk_widget_set_margin_end(toolbar, 4);
    gtk_widget_set_margin_top(toolbar, 4);
    gtk_widget_set_margin_bottom(toolbar, 4);
    gtk_box_append(GTK_BOX(main_box), toolbar);

    // Navigation buttons
    browser->nav_back = gtk_button_new_from_icon_name("go-previous-symbolic");
    g_signal_connect(browser->nav_back, "clicked", G_CALLBACK(on_back_clicked), browser);
    gtk_box_append(GTK_BOX(toolbar), browser->nav_back);

    browser->nav_forward = gtk_button_new_from_icon_name("go-next-symbolic");
    g_signal_connect(browser->nav_forward, "clicked", G_CALLBACK(on_forward_clicked), browser);
    gtk_box_append(GTK_BOX(toolbar), browser->nav_forward);

    browser->nav_reload = gtk_button_new_from_icon_name("view-refresh-symbolic");
    g_signal_connect(browser->nav_reload, "clicked", G_CALLBACK(on_reload_clicked), browser);
    gtk_box_append(GTK_BOX(toolbar), browser->nav_reload);

    browser->nav_home = gtk_button_new_from_icon_name("go-home-symbolic");
    g_signal_connect(browser->nav_home, "clicked", G_CALLBACK(on_home_clicked), browser);
    gtk_box_append(GTK_BOX(toolbar), browser->nav_home);

    // Address bar
    browser->address_bar = gtk_entry_new();
    gtk_entry_set_placeholder_text(GTK_ENTRY(browser->address_bar), "Enter URL...");
    gtk_widget_set_hexpand(browser->address_bar, TRUE);
    g_signal_connect(browser->address_bar, "activate",
                     G_CALLBACK(on_address_bar_activate), browser);
    gtk_box_append(GTK_BOX(toolbar), browser->address_bar);

    // ── Tab strip ──
    browser->tab_strip = gtk_box_new(GTK_ORIENTATION_VERTICAL, 0);
    gtk_box_append(GTK_BOX(main_box), browser->tab_strip);

    // Tab bar (horizontal scrollable)
    browser->tab_bar = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 2);
    gtk_widget_set_size_request(browser->tab_bar, -1, TAB_HEIGHT);
    gtk_widget_set_margin_start(browser->tab_bar, 4);
    gtk_widget_set_margin_end(browser->tab_bar, 4);
    gtk_box_append(GTK_BOX(browser->tab_strip), browser->tab_bar);

    // New tab button
    GtkWidget *new_tab_btn = gtk_button_new_from_icon_name("list-add-symbolic");
    gtk_widget_set_size_request(new_tab_btn, 28, TAB_HEIGHT);
    g_signal_connect(new_tab_btn, "clicked", G_CALLBACK(on_new_tab_clicked), browser);
    gtk_box_append(GTK_BOX(browser->tab_bar), new_tab_btn);

    // ── Browser view (placeholder) ──
    browser->browser_view = gtk_label_new("Browser view — connect CEF or WebKit here");
    gtk_widget_set_vexpand(browser->browser_view, TRUE);
    gtk_widget_set_hexpand(browser->browser_view, TRUE);
    gtk_widget_add_css_class(browser->browser_view, "browser-view");
    gtk_box_append(GTK_BOX(main_box), browser->browser_view);

    // ── Status bar ──
    browser->status_bar = gtk_label_new("Ready");
    gtk_widget_set_size_request(browser->status_bar, -1, STATUS_HEIGHT);
    gtk_widget_set_halign(browser->status_bar, GTK_ALIGN_START);
    gtk_widget_set_margin_start(browser->status_bar, 8);
    gtk_widget_add_css_class(browser->status_bar, "status-bar");
    gtk_box_append(GTK_BOX(main_box), browser->status_bar);

    // ── Keyboard shortcuts ──
    GtkEventController *key_controller = gtk_event_controller_key_new();
    g_signal_connect(key_controller, "key-pressed",
                     G_CALLBACK(on_key_press), browser);
    gtk_widget_add_controller(browser->window, key_controller);

    // ── CSS styling ──
    GtkCssProvider *css = gtk_css_provider_new();
    gtk_css_provider_load_from_string(css,
        ".active-tab { "
        "  background: alpha(@theme_selected_bg_color, 0.2); "
        "  border-bottom: 2px solid @theme_selected_bg_color; "
        "  padding: 4px 8px; "
        "} "
        ".tab { "
        "  padding: 4px 8px; "
        "} "
        ".tab:hover { "
        "  background: alpha(@theme_selected_bg_color, 0.1); "
        "} "
        ".browser-view { "
        "  background: @theme_bg_color; "
        "  color: @theme_fg_color; "
        "} "
        ".status-bar { "
        "  background: alpha(@theme_fg_color, 0.05); "
        "  font-size: 12px; "
        "}"
    );
    gtk_style_context_add_provider_for_display(
        gdk_display_get_default(),
        GTK_STYLE_PROVIDER(css),
        GTK_STYLE_PROVIDER_PRIORITY_APPLICATION);

    return browser;
}

void browser_window_show(BrowserWindow *browser) {
    if (!browser) return;
    gtk_widget_present(browser->window);
}

void browser_window_free(BrowserWindow *browser) {
    if (!browser) return;
    for (int i = 0; i < browser->tab_count; i++) {
        free(browser->tabs[i].title);
        free(browser->tabs[i].url);
    }
    free(browser->tabs);
    free(browser->home_url);
    free(browser);
}

void browser_navigate(BrowserWindow *browser, const char *url) {
    if (!browser || !url) return;

    // Update address bar
    gtk_editable_set_text(GTK_EDITABLE(browser->address_bar), url);

    // Update current tab
    if (browser->current_tab >= 0 && browser->current_tab < browser->tab_count) {
        free(browser->tabs[browser->current_tab].url);
        browser->tabs[browser->current_tab].url = strdup(url);

        // Update tab label
        char display[64];
        const char *title = strrchr(url, '/');
        if (title && strlen(title) > 1) {
            snprintf(display, sizeof(display), "%s", title + 1);
        } else {
            snprintf(display, sizeof(display), "%s", url);
        }
        free(browser->tabs[browser->current_tab].title);
        browser->tabs[browser->current_tab].title = strdup(display);
        rebuild_tab_strip(browser);
    }

    // Update status bar
    gtk_label_set_text(GTK_LABEL(browser->status_bar), "Loading...");

    // TODO: Connect to CEF/WebKit for actual navigation
}

void browser_go_back(BrowserWindow *browser) {
    (void)browser;
    // TODO: CEF/WebKit back
}

void browser_go_forward(BrowserWindow *browser) {
    (void)browser;
    // TODO: CEF/WebKit forward
}

void browser_reload(BrowserWindow *browser) {
    if (!browser) return;
    if (browser->current_tab >= 0 && browser->current_tab < browser->tab_count) {
        browser_navigate(browser, browser->tabs[browser->current_tab].url);
    }
}

void browser_go_home(BrowserWindow *browser) {
    if (!browser) return;
    browser_navigate(browser, browser->home_url);
}

void browser_add_tab(BrowserWindow *browser, const char *title, const char *url) {
    if (!browser) return;

    // Grow array if needed
    if (browser->tab_count >= browser->tab_capacity) {
        browser->tab_capacity *= 2;
        browser->tabs = realloc(browser->tabs,
                                browser->tab_capacity * sizeof(BrowserTab));
    }

    int idx = browser->tab_count++;
    browser->tabs[idx].title = strdup(title);
    browser->tabs[idx].url = strdup(url);
    browser->tabs[idx].page = NULL;

    rebuild_tab_strip(browser);
    browser_select_tab(browser, idx);
}

void browser_close_tab(BrowserWindow *browser, int index) {
    if (!browser || index < 0 || index >= browser->tab_count) return;
    if (browser->tab_count <= 1) return;

    free(browser->tabs[index].title);
    free(browser->tabs[index].url);

    // Shift remaining tabs
    for (int i = index; i < browser->tab_count - 1; i++) {
        browser->tabs[i] = browser->tabs[i + 1];
    }
    browser->tab_count--;

    if (browser->current_tab >= browser->tab_count) {
        browser->current_tab = browser->tab_count - 1;
    }

    rebuild_tab_strip(browser);
    browser_select_tab(browser, browser->current_tab);
}

void browser_select_tab(BrowserWindow *browser, int index) {
    if (!browser || index < 0 || index >= browser->tab_count) return;

    browser->current_tab = index;

    // Update address bar
    gtk_editable_set_text(GTK_EDITABLE(browser->address_bar),
                          browser->tabs[index].url);

    rebuild_tab_strip(browser);

    // TODO: Show the selected tab's web view
}
