#import "BrowserWindowController.h"

#include <include/cef_browser.h>
#include <include/cef_client.h>

// ---------------------------------------------------------------------------
// Rust FFI declarations
// ---------------------------------------------------------------------------

extern "C" {
FfiTab*     feathersurf_tab_create(uint64_t id, const char* url);
void        feathersurf_tab_destroy(FfiTab* tab);
uint32_t    feathersurf_tab_state(const FfiTab* tab);
uint32_t    feathersurf_tab_transition(FfiTab* tab, uint32_t target_state);
const char* feathersurf_tab_url(const FfiTab* tab);
char*       feathersurf_tab_title(const FfiTab* tab);
void        feathersurf_string_free(char* s);
}

// ---------------------------------------------------------------------------
// CEF Client – provides the render handler and life-span handler
// ---------------------------------------------------------------------------

class BrowserClient : public CefClient,
                      public CefLifeSpanHandler,
                      public CefLoadHandler {
 public:
  BrowserClient() = default;

  // CefClient
  CefRefPtr<CefLifeSpanHandler> GetLifeSpanHandler() override { return this; }
  CefRefPtr<CefLoadHandler>     GetLoadHandler() override { return this; }

  // CefLifeSpanHandler – called when the browser is created or closed.
  void OnAfterCreated(CefRefPtr<CefBrowser> browser) override {}
  bool DoClose(CefRefPtr<CefBrowser> browser) override { return false; }
  void OnBeforeClose(CefRefPtr<CefBrowser> browser) override {}

  // CefLoadHandler – update title on page load.
  void OnTitleChange(CefRefPtr<CefBrowser> browser,
                     const CefString& title) override {
    NSString* titleStr =
        [NSString stringWithUTF8String:title.ToString().c_str()];
    dispatch_async(dispatch_get_main_queue(), ^{
      [[NSNotificationCenter defaultCenter]
          postNotificationName:@"FSTitleChanged"
                        object:nil
                      userInfo:@{@"title" : titleStr}];
    });
  }

  IMPLEMENT_REFCOUNTING(BrowserClient);
};

// ---------------------------------------------------------------------------
// Private interface
// ---------------------------------------------------------------------------

@interface BrowserWindowController () {
  CefRefPtr<BrowserClient> _client;
}
@property(nonatomic, strong) NSTextField* urlField;
@property(nonatomic, strong) NSButton* goButton;
@property(nonatomic, strong) NSView* browserContainer;
@end

@implementation BrowserWindowController

#pragma mark - Initialisation

- (instancetype)initWithURL:(NSString*)url {
  NSRect frame = NSMakeRect(300, 300, 1200, 800);
  NSWindowStyleMask style = NSWindowStyleMaskTitled |
                            NSWindowStyleMaskClosable |
                            NSWindowStyleMaskMiniaturizable |
                            NSWindowStyleMaskResizable;
  NSWindow* window = [[NSWindow alloc] initWithContentRect:frame
                                                 styleMask:style
                                                   backing:NSBackingStoreBuffered
                                                     defer:NO];
  window.title = @"FeatherSurf";
  window.delegate = self;
  [window center];

  self = [super initWithWindow:window];
  if (self) {
    _client = new BrowserClient();
    [self buildUI];
    [self loadURL:url];
  }
  return self;
}

- (void)dealloc {
  if (_rustTab) {
    feathersurf_tab_destroy(_rustTab);
    _rustTab = nullptr;
  }
}

#pragma mark - UI

- (void)buildUI {
  NSView* contentView = [[self window] contentView];

  // URL bar
  _urlField = [NSTextField textFieldWithString:@""];
  _urlField.translatesAutoresizingMaskIntoConstraints = NO;
  _urlField.placeholderString = @"Enter URL";
  _urlField.font = [NSFont systemFontOfSize:14];
  _urlField.target = self;
  _urlField.action = @selector(goClicked);
  [contentView addSubview:_urlField];

  _goButton = [NSButton buttonWithTitle:@"Go"
                                 target:self
                                 action:@selector(goClicked)];
  _goButton.translatesAutoresizingMaskIntoConstraints = NO;
  _goButton.bezelStyle = NSBezelStyleRounded;
  [contentView addSubview:_goButton];

  // Browser container
  _browserContainer = [[NSView alloc] initWithFrame:NSZeroRect];
  _browserContainer.translatesAutoresizingMaskIntoConstraints = NO;
  _browserContainer.wantsLayer = YES;
  _browserContainer.layer.backgroundColor = [[NSColor whiteColor] CGColor];
  [contentView addSubview:_browserContainer];

  [NSLayoutConstraint activateConstraints:@[
    [_urlField.topAnchor constraintEqualToAnchor:contentView.topAnchor constant:6],
    [_urlField.leadingAnchor constraintEqualToAnchor:contentView.leadingAnchor constant:8],
    [_urlField.trailingAnchor constraintEqualToAnchor:_goButton.leadingAnchor constant:-8],
    [_urlField.bottomAnchor constraintEqualToAnchor:contentView.topAnchor constant:-34],
    [_urlField.heightAnchor constraintEqualToConstant:28],

    [_goButton.centerYAnchor constraintEqualToAnchor:_urlField.centerYAnchor],
    [_goButton.trailingAnchor constraintEqualToAnchor:contentView.trailingAnchor constant:-8],
    [_goButton.widthAnchor constraintEqualToConstant:50],

    [_browserContainer.topAnchor constraintEqualToAnchor:_urlField.bottomAnchor constant:6],
    [_browserContainer.leadingAnchor constraintEqualToAnchor:contentView.leadingAnchor],
    [_browserContainer.trailingAnchor constraintEqualToAnchor:contentView.trailingAnchor],
    [_browserContainer.bottomAnchor constraintEqualToAnchor:contentView.bottomAnchor],
  ]];

  [[NSNotificationCenter defaultCenter] addObserver:self
                                           selector:@selector(titleChanged:)
                                               name:@"FSTitleChanged"
                                             object:nil];
}

#pragma mark - Navigation

- (void)loadURL:(NSString*)urlString {
  _urlField.stringValue = urlString;

  // Create Rust tab
  uint64_t tabId = (uint64_t)(uintptr_t)self;
  _rustTab = feathersurf_tab_create(tabId, [urlString UTF8String]);

  // Create CEF browser in the container
  NSRect bounds = [_browserContainer bounds];
  CefWindowInfo windowInfo;
  windowInfo.SetAsChild(
      (void*)(uintptr_t)[[_browserContainer window] windowNumber],
      CefRect(bounds.origin.x, bounds.origin.y, bounds.size.width, bounds.size.height));

  CefBrowserSettings settings;
  std::string urlStr([urlString UTF8String]);
  _cefBrowser = CefBrowserHost::CreateBrowser(
      windowInfo, _client, urlStr, settings, nullptr, nullptr);

  feathersurf_tab_transition(_rustTab, FSTabStateActive);
}

- (void)navigateTo:(NSString*)urlString {
  if (!urlString || urlString.length == 0) return;

  if (![urlString hasPrefix:@"http://"] && ![urlString hasPrefix:@"https://"]) {
    urlString = [@"https://" stringByAppendingString:urlString];
  }
  _urlField.stringValue = urlString;

  if (_cefBrowser) {
    std::string urlStr([urlString UTF8String]);
    _cefBrowser->GetMainFrame()->LoadURL(CefString(urlStr));
  }
  feathersurf_tab_transition(_rustTab, FSTabStateActive);
}

- (void)goClicked {
  [self navigateTo:[_urlField stringValue]];
}

#pragma mark - Tab state

- (void)setTabState:(FSTabState)state {
  if (_rustTab) {
    feathersurf_tab_transition(_rustTab, (uint32_t)state);
  }
}

#pragma mark - NSWindowDelegate

- (void)windowDidResize:(NSNotification*)notification {
  if (_cefBrowser) {
    NSView* handle = _cefBrowser->GetHost()->GetWindowHandle();
    if (handle) {
      NSRect bounds = [_browserContainer bounds];
      [handle setFrame:bounds];
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
  [[NSNotificationCenter defaultCenter] removeObserver:self];
}

#pragma mark - Notification handler

- (void)titleChanged:(NSNotification*)note {
  NSString* title = note.userInfo[@"title"];
  if (title) {
    [[self window] setTitle:title];
  }
}

@end
