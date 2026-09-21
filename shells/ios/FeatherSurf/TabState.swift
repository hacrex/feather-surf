import Foundation

/// Tab state constants matching the Rust TabState enum.
enum TabState: UInt32 {
    case active = 0
    case recentlyActive = 1
    case background = 2
    case frozen = 3
    case suspended = 4
    case discardable = 5
}
