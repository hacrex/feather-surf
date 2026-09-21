#import <Cocoa/Cocoa.h>
#include <include/cef_client.h>

/// App delegate for FeatherSurf macOS.
///
/// Initialises CEF, links to the Rust FFI library, and manages the app lifecycle.
@interface AppDelegate : NSObject <NSApplicationDelegate>
@end
