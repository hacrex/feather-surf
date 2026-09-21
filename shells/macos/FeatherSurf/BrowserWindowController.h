#import <Cocoa/Cocoa.h>
#include <include/cef_browser.h>

/// Forward-declare the Rust tab handle.
typedef struct FfiTab FfiTab;

/// Tab state constants matching the Rust TabState enum.
typedef NS_ENUM(uint32_t, FSTabState) {
  FSTabStateActive         = 0,
  FSTabStateRecentlyActive = 1,
  FSTabStateBackground     = 2,
  FSTabStateFrozen         = 3,
  FSTabStateSuspended      = 4,
  FSTabStateDiscardable    = 5,
};

/// Window controller that hosts a single CEF browser instance.
///
/// Each BrowserWindowController owns one tab (FfiTab*) from the Rust core and
/// renders it inside a Cocoa NSWindow via CEF's native windowed mode.
@interface BrowserWindowController : NSWindowController <NSWindowDelegate>

/// The Rust tab handle managed by this window.
@property(nonatomic, readonly, nullable) FfiTab* rustTab;

/// The underlying CEF browser (exposed for testing).
@property(nonatomic, readonly, nullable) CefRefPtr<CefBrowser> cefBrowser;

/// Designated initializer.
/// @param url  The initial URL to load.
- (instancetype)initWithURL:(NSString*)url;

/// Navigate the browser to a new URL.
- (void)navigateTo:(NSString*)urlString;

/// Synchronize the Rust tab state with the given value.
- (void)setTabState:(FSTabState)state;

@end
