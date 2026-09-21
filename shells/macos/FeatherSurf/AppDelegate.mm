#import "AppDelegate.h"
#include <include/cef_app.h>
#include <include/cef_browser.h>

// ---------------------------------------------------------------------------
// Rust FFI declarations (libfeathersurf_ffi.dylib)
// ---------------------------------------------------------------------------

typedef struct FfiTab FfiTab;

extern "C" {
FfiTab*     feathersurf_tab_create(uint64_t id, const char* url);
void        feathersurf_tab_destroy(FfiTab* tab);
uint32_t    feathersurf_tab_state(const FfiTab* tab);
uint32_t    feathersurf_tab_transition(FfiTab* tab, uint32_t target_state);
const char* feathersurf_tab_url(const FfiTab* tab);
char*       feathersurf_tab_title(const FfiTab* tab);
void        feathersurf_string_free(char* s);
}

// Tab state constants matching the Rust TabState enum
static const uint32_t TAB_ACTIVE          = 0;
static const uint32_t TAB_RECENTLY_ACTIVE = 1;
static const uint32_t TAB_BACKGROUND      = 2;
static const uint32_t TAB_FROZEN          = 3;
static const uint32_t TAB_SUSPENDED       = 4;
static const uint32_t TAB_DISCARDABLE     = 5;

// ---------------------------------------------------------------------------
// CEF App – browser-process hooks
// ---------------------------------------------------------------------------

class FeatherSurfApp : public CefApp, public CefBrowserProcessHandler {
 public:
  CefRefPtr<CefBrowserProcessHandler> GetBrowserProcessHandler() override {
    return this;
  }
  void OnBeforeCommandLineProcessing(
      const CefString& process_type,
      CefRefPtr<CefCommandLine> command_line) override {
    command_line->AppendSwitch("enable-gpu");
    command_line->AppendSwitch("disable-gpu-compositing");
  }
 private:
  IMPLEMENT_REFCOUNTING(FeatherSurfApp);
};

// ---------------------------------------------------------------------------
// AppDelegate
// ---------------------------------------------------------------------------

@interface AppDelegate ()
@property(nonatomic, strong) NSWindow* mainWindow;
@property(nonatomic, strong) NSTextField* urlField;
@property(nonatomic, strong) NSButton* goButton;
@property(nonatomic, strong) NSView* browserContainer;
@property(nonatomic, assign) FfiTab* rustTab;
@end

@implementation AppDelegate {
  CefRefPtr<CefBrowser> _cefBrowser;
}

#pragma mark - App lifecycle

- (void)applicationDidFinishLaunching:(NSNotification*)notification {
  [self initCEF];
  [self initRustCore];
  [self buildUI];
  [self loadInitialURL];
}

- (void)applicationWillTerminate:(NSNotification*)notification {
  if (_cefBrowser) {
    _cefBrowser->GetHost()->CloseBrowser(true);
    _cefBrowser = nullptr;
  }
  [self shutdownCEF];
}

- (BOOL)applicationShouldTerminateAfterLastWindowClosed:(NSApplication*)app {
  return YES;
}

- (void)applicationDidBecomeActive:(NSNotification*)notification {
  if (_rustTab) {
    feathersurf_tab_transition(_rustTab, TAB_ACTIVE);
  }
}

- (void)applicationDidResignActive:(NSNotification*)notification {
  if (_rustTab) {
    feathersurf_tab_transition(_rustTab, TAB_BACKGROUND);
  }
}

#pragma mark - CEF initialisation

- (void)initCEF {
  CefMainArgs main_args(0, nullptr);
  CefRefPtr<FeatherSurfApp> app(new FeatherSurfApp());
  if (!CefInitialize(main_args, CefSettings(), app.get(), nullptr)) {
    NSLog(@"CefInitialize failed");
  }
  // Start the CEF message loop on the main thread
  [self pumpCEFMessages];
}

- (void)pumpCEFMessages {
  CefDoMessageLoopWork();
  dispatch_after(
      dispatch_time(DISPATCH_TIME_NOW, (int64_t)(16 * NSEC_PER_MSEC)),
      dispatch_get_main_queue(), ^{
        [self pumpCEFMessages];
      });
}

- (void)shutdownCEF {
  CefShutdown();
}

#pragma mark - Rust core

- (void)initRustCore {
  NSString* homeDir = NSHomeDirectory();
  const char* url = "https://www.google.com";
  _rustTab = feathersurf_tab_create(1, url);
  if (!_rustTab) {
    NSLog(@"Failed to create Rust tab");
  }
}

#pragma mark - UI construction

- (void)buildUI {
  // Main window (NSWindow with title bar, URL bar, and browser container)
  NSRect frame = NSMakeRect(200, 200, 1200, 800);
  NSWindowStyleMask style = NSWindowStyleMaskTitled |
                            NSWindowStyleMaskClosable |
                            NSWindowStyleMaskMiniaturizable |
                            NSWindowStyleMaskResizable;
  _mainWindow = [[NSWindow alloc] initWithContentRect:frame
                                            styleMask:style
                                              backing:NSBackingStoreBuffered
                                                defer:NO];
  _mainWindow.title = @"FeatherSurf";
  _mainWindow.delegate = (id<NSWindowDelegate>)self;
  [_mainWindow center];

  NSView* contentView = [_mainWindow contentView];

  // --- URL bar row ---------------------------------------------------------
  NSView* urlBarContainer = [[NSView alloc] initWithFrame:NSMakeRect(0, 0, 0, 0)];
  urlBarContainer.translatesAutoresizingMaskIntoConstraints = NO;
  [contentView addSubview:urlBarContainer];

  _urlField = [NSTextField textFieldWithString:@"https://www.google.com"];
  _urlField.translatesAutoresizingMaskIntoConstraints = NO;
  _urlField.placeholderString = @"Enter URL";
  _urlField.font = [NSFont systemFontOfSize:14];
  _urlField.target = self;
  _urlField.action = @selector(navigateToURL);
  [urlBarContainer addSubview:_urlField];

  _goButton = [NSButton buttonWithTitle:@"Go" target:self action:@selector(navigateToURL)];
  _goButton.translatesAutoresizingMaskIntoConstraints = NO;
  _goButton.bezelStyle = NSBezelStyleRounded;
  [urlBarContainer addSubview:_goButton];

  // --- Browser container ---------------------------------------------------
  _browserContainer = [[NSView alloc] initWithFrame:NSMakeRect(0, 0, 0, 0)];
  _browserContainer.translatesAutoresizingMaskIntoConstraints = NO;
  _browserContainer.wantsLayer = YES;
  _browserContainer.layer.backgroundColor = [[NSColor whiteColor] CGColor];
  [contentView addSubview:_browserContainer];

  // --- Layout constraints --------------------------------------------------
  [NSLayoutConstraint activateConstraints:@[
    [urlBarContainer.topAnchor constraintEqualToAnchor:contentView.topAnchor],
    [urlBarContainer.leadingAnchor constraintEqualToAnchor:contentView.leadingAnchor],
    [urlBarContainer.trailingAnchor constraintEqualToAnchor:contentView.trailingAnchor],
    [urlBarContainer.heightAnchor constraintEqualToConstant:40],

    [_urlField.topAnchor constraintEqualToAnchor:urlBarContainer.topAnchor constant:6],
    [_urlField.leadingAnchor constraintEqualToAnchor:urlBarContainer.leadingAnchor constant:8],
    [_urlField.trailingAnchor constraintEqualToAnchor:_goButton.leadingAnchor constant:-8],
    [_urlField.bottomAnchor constraintEqualToAnchor:urlBarContainer.bottomAnchor constant:-6],

    [_goButton.centerYAnchor constraintEqualToAnchor:urlBarContainer.centerYAnchor],
    [_goButton.trailingAnchor constraintEqualToAnchor:urlBarContainer.trailingAnchor constant:-8],
    [_goButton.widthAnchor constraintEqualToConstant:50],

    [_browserContainer.topAnchor constraintEqualToAnchor:urlBarContainer.bottomAnchor],
    [_browserContainer.leadingAnchor constraintEqualToAnchor:contentView.leadingAnchor],
    [_browserContainer.trailingAnchor constraintEqualToAnchor:contentView.trailingAnchor],
    [_browserContainer.bottomAnchor constraintEqualToAnchor:contentView.bottomAnchor],
  ]];

  [_mainWindow makeKeyAndOrderFront:nil];
}

#pragma mark - Navigation

- (void)loadInitialURL {
  NSString* initialURL = @"https://www.google.com";
  [self navigateTo:initialURL];
}

- (void)navigateToURL {
  NSString* text = [_urlField stringValue];
  if (text.length == 0) return;

  if (![text hasPrefix:@"http://"] && ![text hasPrefix:@"https://"]) {
    text = [@"https://" stringByAppendingString:text];
  }
  [self navigateTo:text];
}

- (void)navigateTo:(NSString*)urlString {
  // Update the URL field
  _urlField.stringValue = urlString;

  // Create or navigate CEF browser
  std::string urlStr([urlString UTF8String]);

  CefWindowInfo windowInfo;
  NSView* contentView = [_browserContainer window].contentView;
  NSRect bounds = [_browserContainer bounds];
  windowInfo.SetAsChild(
      (void*)(uintptr_t)[_browserContainer window].windowNumber,
      CefRect(bounds.origin.x, bounds.origin.y, bounds.size.width, bounds.size.height));

  CefBrowserSettings settings;
  if (!_cefBrowser) {
    _cefBrowser = CefBrowserHost::CreateBrowser(
        windowInfo, nullptr, urlStr, settings, nullptr, nullptr);
  } else {
    _cefBrowser->GetMainFrame()->LoadURL(CefString(urlStr));
  }

  // Sync URL to Rust core
  if (_rustTab) {
    feathersurf_tab_transition(_rustTab, TAB_ACTIVE);
  }
}

#pragma mark - NSWindowDelegate

- (void)windowDidResize:(NSNotification*)notification {
  if (_cefBrowser) {
    NSRect bounds = [_browserContainer bounds];
    NSView* contentView = _cefBrowser->GetHost()->GetWindowHandle();
    if (contentView) {
      [contentView setFrame:bounds];
    }
  }
}

- (void)windowWillClose:(NSNotification*)notification {
  if (_cefBrowser) {
    _cefBrowser->GetHost()->CloseBrowser(true);
    _cefBrowser = nullptr;
  }
  if (_rustTab) {
    feathersurf_tab_destroy(_rustTab);
    _rustTab = nullptr;
  }
}

@end
